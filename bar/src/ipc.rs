use std::io;
use std::path::PathBuf;
use std::rc::Rc;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::App;

#[derive(Debug)]
pub struct Ipc {
    path: PathBuf,
}

impl Ipc {
    pub fn new() -> Self {
        let socket_file = std::env::var("XDG_RUNTIME_DIR")
            .map_or_else(|_| PathBuf::from("/tmp"), PathBuf::from)
            .join("bar_ipc.sock");
        
        Self {
            path: socket_file,
        }
    }

    pub async fn send(&self, text: String) -> io::Result<String> {
        let mut stream = match UnixStream::connect(&self.path).await {
            Ok(stream) => Ok(stream),
            Err(err) => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Failed to connect to IPC socket: {}", err),
            )),
        }?;

        stream.write_all(text.as_bytes()).await?;
        stream.write_all(b"\n").await?;

        let mut response = String::new();
        stream.read_to_string(&mut response).await?;
        Ok(response)
    }

    /// Starts the IPC server on its socket.
    pub fn start(&self, app: Rc<App>) {
        let (tx_input, mut rx_input) = mpsc::channel(10);
        let (tx_output, mut rx_output) = mpsc::channel(10);

        let path = self.path.clone();

        // If the socket file already exists, remove it
        if path.exists() {
            if let Err(err) = std::fs::remove_file(&path) {
                eprintln!("Failed to remove existing IPC socket file: {}", err);
            }
        }

        App::spawn(async move {
            let listener = match UnixListener::bind(&path) {
                Ok(listener) => listener,
                Err(err) => {
                    panic!("Failed to bind IPC socket at {}: {}", path.display(), err);
                }
            };

            println!("IPC server listening on {}", path.display());

            loop {
                match listener.accept().await {
                    Ok((stream, _addr)) => {
                        if let Err(err) = Self::handle_connection(stream, &tx_input, &mut rx_output).await {
                            println!("Failed to handle connection: {}", err);
                            continue;
                        }
                    }
                    Err(err) => {
                        println!("Failed to accept connection: {}", err);
                        continue;
                    }
                }
            }
        });

        let app_clone = app.clone();
        glib::spawn_future_local(async move {
            while let Some(val) = rx_input.recv().await {
                let res = Self::handle_command(val, app_clone.clone());
                tx_output.send(res).await.unwrap_or_else(|_| {
                    eprintln!("Failed to send response back to client");
                });
            }
        });
    }

    /// Handles a connection from a IPC client by reading its message and closing the connection.
    async fn handle_connection(
        mut stream: UnixStream,
        tx_command: &Sender<String>,
        rx_response: &mut Receiver<String>,
    ) -> io::Result<()> {
        let (stream_read, mut stream_write) = stream.split();

        let mut bytes = Vec::new();
        let mut stream_read = BufReader::new(stream_read);
        stream_read.read_until(b'\n', &mut bytes).await.map_err(|err| {
            io::Error::new(io::ErrorKind::Other, format!("Failed to read from stream: {}", err))
        })?;

        let buffer = String::from_utf8(bytes).map_err(|err| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse UTF-8: {}", err))
        })?;

        let command = buffer.trim().to_string();
        if command.is_empty() {
            return Ok(());
        }

        tx_command.send(command).await.map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "Failed to send command to handler")
        })?;
        let res = rx_response.recv().await.unwrap_or("Error: No response".to_string());

        stream_write.write_all(res.as_bytes()).await?;
        stream_write.shutdown().await?;

        Ok(())
    }

    /// Handles a command recieved over IPC.
    fn handle_command(
        command: String,
        app: Rc<App>,
    ) -> String {
        if command == "inspect" {
            gtk4::Window::set_interactive_debugging(true);
            return "ok".to_string();
        }
        if let Some(response) = app.modules.borrow_mut().handle_command(&command, app.clone()) {
            return response;
        }
        return format!("Error: Unknown command '{}'", command);
    }
}