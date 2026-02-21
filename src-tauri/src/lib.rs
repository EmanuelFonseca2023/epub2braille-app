mod braille;

#[tauri::command]
fn convertir(path: String) -> Result<String, String> {
    braille::convertir_epub_a_bin(&path)
}

#[tauri::command]
fn extraer_texto(path: String) -> Result<String, String> {
    braille::extraer_texto_epub(&path)
}

#[tauri::command]
fn convertir_texto(texto: String, bin_path: String) -> Result<String, String> {
    braille::convertir_texto_a_bin(&texto, &bin_path)
}

#[tauri::command]
fn obtener_tamano(path: String) -> Result<u64, String> {
    std::fs::metadata(&path)
        .map(|m| m.len())
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![convertir, extraer_texto, convertir_texto, obtener_tamano])
        .run(tauri::generate_context!())
        .expect("error al iniciar Tauri");
}