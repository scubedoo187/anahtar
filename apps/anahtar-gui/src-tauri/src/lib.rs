use serde::Serialize;

#[derive(Debug, Serialize)]
struct BackendStatus {
    app: &'static str,
    version: &'static str,
    service: &'static str,
}

#[tauri::command]
fn backend_status() -> BackendStatus {
    BackendStatus {
        app: "Anahtar",
        version: env!("CARGO_PKG_VERSION"),
        service: "anahtar-app ready",
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![backend_status])
        .run(tauri::generate_context!())
        .expect("failed to run Anahtar GUI");
}
