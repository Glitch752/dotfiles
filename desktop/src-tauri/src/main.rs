// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // If there are any arguments, we assume they are for the IPC server
    if std::env::args().len() > 1 {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");
        runtime.block_on(async {
            let args: Vec<String> = std::env::args().collect();
            let ipc = desktop_lib::Ipc::new();
            let response = ipc
                .send(args[1..].join(" "))
                .await
                .expect("Failed to send IPC message");

            println!("{}", response);
        });
        return;
    }

    desktop_lib::run()
}
