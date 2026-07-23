use crate::startup;
use tauri::Manager;

/// 获取开机自启状态
#[tauri::command]
pub fn get_startup() -> Result<bool, String> {
    startup::is_enabled()
}

/// 设置开机自启
#[tauri::command]
pub fn set_startup(enabled: bool) -> Result<(), String> {
    startup::set_enabled(enabled)
}

/// 设置窗口主题（深色/浅色标题栏）
#[tauri::command]
pub fn set_window_theme(app_handle: tauri::AppHandle, theme: String) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let t = match theme.as_str() {
            "dark" => tauri::Theme::Dark,
            _ => tauri::Theme::Light,
        };
        window.set_theme(Some(t)).map_err(|e| format!("设置窗口主题失败: {}", e))?;
    }
    Ok(())
}
