use auto_launch::AutoLaunch;
use auto_launch::AutoLaunchBuilder;

/// 获取开机自启管理器实例
fn get_auto_launch() -> Result<AutoLaunch, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("获取程序路径失败: {}", e))?;
    let exe_str = exe_path.to_string_lossy();

    AutoLaunchBuilder::new()
        .set_app_name("CrabProxy")
        .set_app_path(&exe_str)
        .build()
        .map_err(|e| format!("创建开机自启管理器失败: {}", e))
}

/// 检查开机自启是否已启用
pub fn is_enabled() -> Result<bool, String> {
    let auto = get_auto_launch()?;
    auto.is_enabled()
        .map_err(|e| format!("检查开机自启状态失败: {}", e))
}

/// 设置开机自启
pub fn set_enabled(enabled: bool) -> Result<(), String> {
    let auto = get_auto_launch()?;
    if enabled {
        auto.enable()
            .map_err(|e| format!("启用开机自启失败: {}", e))?;
    } else {
        auto.disable()
            .map_err(|e| format!("禁用开机自启失败: {}", e))?;
    }
    log::info!("开机自启: {}", if enabled { "已启用" } else { "已禁用" });
    Ok(())
}
