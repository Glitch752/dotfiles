use std::io;
use std::path::PathBuf;

use tauri::{AppHandle, Emitter, Listener};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Ipc {
    path: PathBuf,
}

impl Ipc {
    pub fn new() -> Self {
        let socket_file = std::env::var("XDG_RUNTIME_DIR")
            .map_or_else(|_| PathBuf::from("/tmp"), PathBuf::from)
            .join("bar_ipc.sock");

        Self { path: socket_file }
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
    pub fn start(&self, app: AppHandle) {
        let path = self.path.clone();

        // If the socket file already exists, remove it
        if path.exists() {
            if let Err(err) = std::fs::remove_file(&path) {
                eprintln!("Failed to remove existing IPC socket file: {}", err);
            }
        }

        tauri::async_runtime::spawn(async move {
            let listener = match UnixListener::bind(&path) {
                Ok(listener) => listener,
                Err(err) => {
                    panic!("Failed to bind IPC socket at {}: {}", path.display(), err);
                }
            };

            println!("IPC server listening on {}", path.display());

            loop {
                match listener.accept().await {
                    Ok((stream, _addr)) => match Ipc::handle_connection(stream, &app).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error handling connection: {}", e);
                        }
                    },
                    Err(err) => {
                        println!("Failed to accept connection: {}", err);
                        continue;
                    }
                }
            }
        });
    }

    async fn handle_connection(mut stream: UnixStream, app: &AppHandle) -> io::Result<()> {
        let (stream_read, mut stream_write) = stream.split();

        let mut bytes = Vec::new();
        let mut stream_read = BufReader::new(stream_read);
        stream_read
            .read_until(b'\n', &mut bytes)
            .await
            .map_err(|err| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to read from stream: {}", err),
                )
            })?;

        let buffer = String::from_utf8(bytes).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse UTF-8: {}", err),
            )
        })?;

        let command = buffer.trim().to_string();
        if command.is_empty() {
            // Shouldnt't happen, probably
            eprintln!("Got empty IPC message");
            return Ok(());
        }

        app.emit("ipc_call", command)
            .expect("Failed to emit IPC call");

        let (tx_res, mut rx_res) = mpsc::channel::<String>(1);

        let handler = app.listen("ipc_response", move |res: tauri::Event| {
            // Seems unnecessary, but this unescapes the escaped payload
            let payload = serde_json::from_str::<String>(res.payload())
                .expect("Failed to unserialize IPC response");
            let tx_res = tx_res.clone();
            tauri::async_runtime::spawn(async move {
                tx_res
                    .send(payload)
                    .await
                    .expect("Failed to send IPC repsonse");
            });
        });

        let payload = rx_res.recv().await.ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Failed to receive IPC response")
        })?;
        stream_write.write_all(payload.as_bytes()).await?;
        stream_write.shutdown().await?;

        app.unlisten(handler);

        Ok(())
    }
}
