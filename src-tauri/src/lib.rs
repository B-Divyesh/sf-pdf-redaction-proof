mod pdf;

use pdf::{inspect, sanitize, AuditReport};

#[tauri::command]
fn inspect_pdf(path: String) -> Result<AuditReport, String> { inspect(&path).map_err(|e| e.to_string()) }

#[tauri::command]
fn sanitize_pdf(path: String) -> Result<AuditReport, String> { sanitize(&path).map_err(|e| e.to_string()) }

#[tauri::command]
fn write_report(path: String, contents: String) -> Result<(), String> {
    if contents.len() > 2_000_000 { return Err("Report exceeds the 2 MB safety limit.".into()); }
    std::fs::write(path, contents).map_err(|e| format!("Could not write report: {e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![inspect_pdf, sanitize_pdf, write_report])
        .run(tauri::generate_context!())
        .expect("error while running Redaction Proof");
}
