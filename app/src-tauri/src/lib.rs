/// Versions of the app and of the core library, shown in Settings › About.
#[tauri::command]
fn versions() -> (String, String) {
    (
        env!("CARGO_PKG_VERSION").to_string(),
        divelens_core::VERSION.to_string(),
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![versions])
        .run(tauri::generate_context!())
        .expect("error while running DiveLens");
}
