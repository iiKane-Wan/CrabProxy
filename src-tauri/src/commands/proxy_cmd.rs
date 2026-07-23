use crate::config::manager;
use crate::config::model::PortRule;
use crate::proxy::engine::SharedProxyEngine;
use crate::proxy::state::ProxyState;
use tauri::State;

/// 切换到指定配置
#[tauri::command]
pub async fn switch_config(
    name: String,
    engine: State<'_, SharedProxyEngine>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // 加载配置
    let config = manager::load_config(&name).map_err(|e| e.to_string())?;

    // 切换引擎
    let mut engine = engine.lock().await;
    engine.load_config(&config, &app_handle).await?;

    // 持久化：记住当前激活的配置，下次启动自动恢复
    let _ = manager::save_active_state(&name);

    log::info!("已切换到配置: {}", name);
    Ok(())
}

/// 批量启用/禁用所有端口
#[tauri::command]
pub async fn toggle_all_ports(
    config_name: String,
    enabled: bool,
    engine: State<'_, SharedProxyEngine>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // 如果需要启用，加载配置以获取端口列表
    let config = if enabled {
        Some(manager::load_config(&config_name).map_err(|e| e.to_string())?)
    } else {
        None
    };

    let mut engine = engine.lock().await;
    engine.toggle_all(enabled, config.as_ref(), &app_handle).await?;

    // 同步更新配置文件中的端口启用状态
    if let Some(mut config) = config {
        for port in &mut config.ports {
            port.enabled = enabled;
        }
        manager::save_config(&config).map_err(|e| e.to_string())?;
    }

    log::info!("批量切换端口: config={}, enabled={}", config_name, enabled);
    Ok(())
}

/// 更新单个端口规则
#[tauri::command]
pub async fn update_port(
    config_name: String,
    port: PortRule,
    engine: State<'_, SharedProxyEngine>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // 更新配置文件
    let mut config = manager::load_config(&config_name).map_err(|e| e.to_string())?;

    let target_ip = config.resolve_target_ip(&port);
    let target_port = config.resolve_target_port(&port);
    let existing = config
        .ports
        .iter_mut()
        .find(|p| p.local_port == port.local_port)
        .ok_or_else(|| format!("端口 {} 不存在", port.local_port))?;

    existing.target_ip = port.target_ip;
    existing.target_port = port.target_port;
    existing.enabled = port.enabled;

    manager::save_config(&config).map_err(|e| e.to_string())?;

    // 动态调整代理引擎
    let mut engine = engine.lock().await;
    engine
        .update_port(port.local_port, target_ip, target_port, port.enabled, &app_handle)
        .await?;

    log::info!("端口已更新: config={}, port={}", config_name, port.local_port);
    Ok(())
}

/// 添加新端口
#[tauri::command]
pub async fn add_port(
    config_name: String,
    port: PortRule,
    engine: State<'_, SharedProxyEngine>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // 更新配置文件
    let mut config = manager::load_config(&config_name).map_err(|e| e.to_string())?;

    // 检查端口是否已存在
    if config.ports.iter().any(|p| p.local_port == port.local_port) {
        return Err(format!("端口 {} 已存在", port.local_port));
    }

    let target_ip = config.resolve_target_ip(&port);
    let target_port = config.resolve_target_port(&port);
    config.ports.push(port);

    manager::save_config(&config).map_err(|e| e.to_string())?;

    // 动态添加代理
    let last = config.ports.last().unwrap();
    let mut engine = engine.lock().await;
    engine
        .add_port(last.local_port, target_ip, target_port, last.enabled, &app_handle)
        .await?;

    log::info!("端口已添加: config={}, port={}", config_name, config.ports.last().unwrap().local_port);
    Ok(())
}

/// 删除端口
#[tauri::command]
pub async fn remove_port(
    config_name: String,
    local_port: u16,
    engine: State<'_, SharedProxyEngine>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // 更新配置文件
    let mut config = manager::load_config(&config_name).map_err(|e| e.to_string())?;

    config.ports.retain(|p| p.local_port != local_port);

    manager::save_config(&config).map_err(|e| e.to_string())?;

    // 动态移除代理
    let mut engine = engine.lock().await;
    engine.remove_port(local_port, &app_handle).await?;

    log::info!("端口已删除: config={}, port={}", config_name, local_port);
    Ok(())
}

/// 获取代理运行状态
#[tauri::command]
pub async fn get_proxy_status(
    engine: State<'_, SharedProxyEngine>,
) -> Result<ProxyState, String> {
    let engine = engine.lock().await;
    Ok(engine.snapshot())
}

/// 启动时自动恢复上次激活的配置
/// 前端在初始化时调用此命令，若存在上次保存的状态则自动加载
#[tauri::command]
pub async fn restore_last_session(
    engine: State<'_, SharedProxyEngine>,
    app_handle: tauri::AppHandle,
) -> Result<Option<String>, String> {
    let active_name = manager::get_active_state().map_err(|e| e.to_string())?;
    if let Some(ref name) = active_name {
        log::info!("正在恢复上次会话的配置: {}", name);
        let config = manager::load_config(name).map_err(|e| e.to_string())?;
        let mut engine = engine.lock().await;
        engine.load_config(&config, &app_handle).await?;
    }
    Ok(active_name)
}
