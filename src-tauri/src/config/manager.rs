use crate::config::model::{ConfigMeta, ProxyConfig};
use crate::error::{AppError, AppResult};
use std::fs;
use std::path::PathBuf;

/// 获取配置文件的存储目录
/// Windows: %APPDATA%/port-proxy/configs/
/// Linux:   ~/.config/port-proxy/configs/
/// macOS:   ~/Library/Application Support/port-proxy/configs/
pub fn get_config_dir() -> AppResult<PathBuf> {
    // 使用 Tauri 提供的 app_data_dir 作为备选
    // 这里直接使用标准目录，与 Tauri 解耦便于测试
    let base = if cfg!(target_os = "windows") {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| {
            let home = std::env::var("USERPROFILE").unwrap_or_default();
            format!("{}\\AppData\\Roaming", home)
        });
        PathBuf::from(appdata).join("port-proxy")
    } else if cfg!(target_os = "macos") {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home).join("Library/Application Support/port-proxy")
    } else {
        let home = std::env::var("HOME").unwrap_or_default();
        let xdg = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(&home).join(".config"));
        xdg.join("port-proxy")
    };

    let config_dir = base.join("configs");
    fs::create_dir_all(&config_dir)?;
    Ok(config_dir)
}

/// 获取单个配置文件的路径
fn config_path(config_dir: &PathBuf, name: &str) -> PathBuf {
    config_dir.join(format!("{}.json", sanitize_filename(name)))
}

/// 将配置名称中的不安全字符替换为下划线
fn sanitize_filename(name: &str) -> String {
    name.replace(
        |c: char| c == '/' || c == '\\' || c == ':' || c == '*' || c == '?' || c == '"' || c == '<' || c == '>' || c == '|',
        "_",
    )
}

/// 列出所有配置（返回摘要列表）
pub fn list_configs() -> AppResult<Vec<ConfigMeta>> {
    let config_dir = get_config_dir()?;
    let mut metas = Vec::new();

    let entries = fs::read_dir(&config_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // 只处理 .json 文件，跳过 _index.json 等元文件
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if stem.starts_with('_') {
            continue;
        }

        match load_config_by_name(&config_dir, stem) {
            Ok(config) => {
                metas.push(config.to_meta());
            }
            Err(e) => {
                log::warn!("跳过损坏的配置文件 '{}': {}", stem, e);
                // 为损坏的配置文件生成一个错误标记
                metas.push(ConfigMeta {
                    name: format!("{} [错误]", stem),
                    global_ip: "—".into(),
                    port_count: 0,
                    enabled_count: 0,
                });
            }
        }
    }

    // 按名称排序
    metas.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(metas)
}

/// 加载指定名称的配置
fn load_config_by_name(config_dir: &PathBuf, name: &str) -> AppResult<ProxyConfig> {
    let path = config_path(config_dir, name);
    let content = fs::read_to_string(&path)?;
    let mut config: ProxyConfig = serde_json::from_str(&content)?;

    // 确保配置名称与文件名一致
    config.name = name.to_string();

    // 校验配置有效性
    config.validate().map_err(AppError::Config)?;

    Ok(config)
}

/// 加载指定配置（公开接口）
pub fn load_config(name: &str) -> AppResult<ProxyConfig> {
    let config_dir = get_config_dir()?;
    load_config_by_name(&config_dir, name)
}

/// 保存配置（新建或更新）
pub fn save_config(config: &ProxyConfig) -> AppResult<()> {
    // 先校验
    config.validate().map_err(AppError::Config)?;

    let config_dir = get_config_dir()?;
    let path = config_path(&config_dir, &config.name);

    let json = serde_json::to_string_pretty(config)?;
    fs::write(&path, json)?;

    log::info!("配置已保存: {}", config.name);
    Ok(())
}

/// 删除配置
pub fn delete_config(name: &str) -> AppResult<()> {
    let config_dir = get_config_dir()?;
    let path = config_path(&config_dir, name);

    if !path.exists() {
        return Err(AppError::Config(format!("配置 '{}' 不存在", name)));
    }

    fs::remove_file(&path)?;
    log::info!("配置已删除: {}", name);
    Ok(())
}

/// 重命名配置
pub fn rename_config(old_name: &str, new_name: &str) -> AppResult<()> {
    let config_dir = get_config_dir()?;
    let old_path = config_path(&config_dir, old_name);
    let new_path = config_path(&config_dir, new_name);

    if !old_path.exists() {
        return Err(AppError::Config(format!("配置 '{}' 不存在", old_name)));
    }
    if new_path.exists() {
        return Err(AppError::Config(format!("配置 '{}' 已存在", new_name)));
    }

    // 读取旧配置，更新名称后保存到新文件
    let mut config = load_config_by_name(&config_dir, old_name)?;
    config.name = new_name.to_string();
    let json = serde_json::to_string_pretty(&config)?;
    fs::write(&new_path, json)?;
    fs::remove_file(&old_path)?;

    log::info!("配置已重命名: {} -> {}", old_name, new_name);
    Ok(())
}

/// 保存当前激活的配置名（用于下次启动恢复）
pub fn save_active_state(name: &str) -> AppResult<()> {
    let config_dir = get_config_dir()?;
    let path = config_dir.join("_active.json");
    let json = serde_json::json!({ "active_config": name });
    fs::write(&path, serde_json::to_string_pretty(&json)?)?;
    Ok(())
}

/// 读取上次激活的配置名
pub fn get_active_state() -> AppResult<Option<String>> {
    let config_dir = get_config_dir()?;
    let path = config_dir.join("_active.json");
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)?;
    Ok(parsed
        .get("active_config")
        .and_then(|v| v.as_str())
        .map(String::from))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::PortRule;

    fn test_config(name: &str) -> ProxyConfig {
        ProxyConfig {
            name: name.into(),
            global_ip: "192.168.1.1".into(),
            ports: vec![
                PortRule { name: None, local_port: 8080, target_ip: None, target_port: None, enabled: true },
            ],
        }
    }

    #[test]
    fn test_save_and_load() {
        let name = "test-save-load";
        let config = test_config(name);
        save_config(&config).unwrap();
        let loaded = load_config(name).unwrap();
        assert_eq!(loaded.name, name);
        assert_eq!(loaded.global_ip, "192.168.1.1");
        assert_eq!(loaded.ports.len(), 1);

        // 清理
        delete_config(name).unwrap();
    }

    #[test]
    fn test_list_configs() {
        let name = "test-list-configs";
        let config = test_config(name);
        save_config(&config).unwrap();
        let metas = list_configs().unwrap();
        assert!(metas.iter().any(|m| m.name == name));

        delete_config(name).unwrap();
    }

    #[test]
    fn test_rename() {
        let config = test_config("test-rename-old");
        save_config(&config).unwrap();

        rename_config("test-rename-old", "test-rename-new").unwrap();
        let loaded = load_config("test-rename-new").unwrap();
        assert_eq!(loaded.name, "test-rename-new");

        delete_config("test-rename-new").unwrap();
    }
}
