mod diff;

use diff::{compute_diff_files, DiffResult};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
fn pick_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let path = app
        .dialog()
        .file()
        .set_title("Seleziona file")
        .blocking_pick_file();

    match path {
        None => Ok(None),
        Some(file) => match file.into_path() {
            Ok(p) => Ok(Some(p.to_string_lossy().into_owned())),
            Err(e) => Err(format!("Percorso file non valido: {e}")),
        },
    }
}

#[tauri::command]
fn compute_diff(left_path: String, right_path: String) -> Result<DiffResult, String> {
    compute_diff_files(&left_path, &right_path).map_err(|e| e.message())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![pick_file, compute_diff])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
