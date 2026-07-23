use crate::config::manager;
use crate::config::model::{ConfigMeta, ProxyConfig};

/// 获取所有配置的摘要列表
#[tauri::command]
pub fn get_all_configs() -> Result<Vec<ConfigMeta>, String> {
    manager::list_configs().map_err(|e| e.to_string())
}

/// 加载指定配置的完整内容
#[tauri::command]
pub fn load_config(name: String) -> Result<ProxyConfig, String> {
    manager::load_config(&name).map_err(|e| e.to_string())
}

/// 保存配置（新建或更新）
#[tauri::command]
pub fn save_config(config: ProxyConfig) -> Result<(), String> {
    manager::save_config(&config).map_err(|e| e.to_string())
}

/// 删除配置
#[tauri::command]
pub fn delete_config(name: String) -> Result<(), String> {
    manager::delete_config(&name).map_err(|e| e.to_string())
}

/// 重命名配置
#[tauri::command]
pub fn rename_config(old_name: String, new_name: String) -> Result<(), String> {
    manager::rename_config(&old_name, &new_name).map_err(|e| e.to_string())
}
