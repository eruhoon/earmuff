// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn open_paint() {
    std::process::Command::new("mspaint")
        .spawn()
        .expect("failed to open paint");
}

#[tauri::command]
fn execute_vbs() {
    std::process::Command::new("wscript.exe")
        .arg("C:\\Users\\path\\Desktop\\path\\file.vbs")
        .spawn()
        .expect("failed to execute VBScript");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, open_paint, execute_vbs])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
