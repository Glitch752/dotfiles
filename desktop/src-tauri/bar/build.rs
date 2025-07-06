use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use schemars::schema_for;

const COMMANDS: &[&str] = &[
    "debug_log",
    "niri_request",
    "get_upower_properties",
    "get_networkmanager_state",
    "get_systray_items"
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();

    // Generate Typescript definitions for niri_ipc types with schemars and json-schema-to-typescript

    // Never rerun unless build.rs changes
    println!("cargo:rerun-if-changed=build.rs");

    let build_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get cargo directory");
    let main_bindings_dir = Path::new(&build_dir).join("../bindings");

    fs::create_dir_all("bindings").expect("Failed to create bindings directory");
    fn generate_typescript_schema<T: schemars::JsonSchema>(
        type_name: &str,
        schema_file: PathBuf,
        ts_file: PathBuf,
    ) {
        let schema = schema_for!(T);
        let schema_json =
            serde_json::to_string_pretty(&schema).expect("Failed to serialize schema");
        fs::write(&schema_file, schema_json).unwrap();

        Command::new("pnpx")
            .arg("json-schema-to-typescript")
            .arg("--input")
            .arg(&schema_file)
            .arg("--output")
            .arg(ts_file)
            .status()
            .expect(&format!(
                "Failed to run json-schema-to-typescript for {}",
                type_name
            ));

        fs::remove_file(schema_file).expect("Failed to remove schema file");
    }

    generate_typescript_schema::<niri_ipc::Request>(
        "Request",
        main_bindings_dir.join("niri_ipc_request_schema.json"),
        main_bindings_dir.join("NiriIpcRequest.ts"),
    );
    generate_typescript_schema::<niri_ipc::Response>(
        "Response",
        main_bindings_dir.join("niri_ipc_response_schema.json"),
        main_bindings_dir.join("NiriIpcResponse.ts"),
    );
    generate_typescript_schema::<niri_ipc::Event>(
        "Event",
        main_bindings_dir.join("niri_ipc_event_schema.json"),
        main_bindings_dir.join("NiriIpcEvent.ts"),
    );
}
