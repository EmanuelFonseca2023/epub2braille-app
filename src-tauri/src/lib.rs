mod braille;

#[tauri::command]
fn convertir(path: String) -> Result<String, String> {
    braille::convertir_epub_a_bin(&path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![convertir])
        .run(tauri::generate_context!())
        .expect("error al iniciar Tauri");
}