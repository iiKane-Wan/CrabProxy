mod commands;
mod config;
mod error;
mod proxy;
mod startup;

use fs2::FileExt;
use proxy::engine::create_shared_engine;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

/// 获取实例锁文件路径
fn get_lock_path() -> Result<PathBuf, String> {
    let base = if cfg!(target_os = "windows") {
        let appdata = std::env::var("APPDATA")
            .unwrap_or_else(|_| std::env::var("USERPROFILE").unwrap_or_default());
        PathBuf::from(appdata)
    } else {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home)
    };
    Ok(base.join(".crabproxy.lock"))
}

/// 单实例检测：尝试获取文件锁，失败则说明已有实例在运行
fn lock_instance() -> Result<fs::File, String> {
    let lock_path = get_lock_path()?;
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_path)
        .map_err(|e| format!("无法创建实例锁文件: {}", e))?;

    // 尝试获取排他锁（非阻塞）
    file.try_lock_exclusive()
        .map_err(|_| "应用已在运行中，不能重复启动".to_string())?;

    Ok(file)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 检查单实例锁
    let _lock = match lock_instance() {
        Ok(lock) => lock,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    // 检查是否为静默启动（开机自启）
    let is_silent = std::env::args().any(|a| a == "--silent");

    tauri::Builder::default()
        .setup(move |app| {
            // 开发模式下启用日志插件
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // 注册共享代理引擎为托管状态
            app.manage(create_shared_engine());

            // 构建系统托盘菜单
            let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let tray_icon = app
                .default_window_icon()
                .cloned()
                .unwrap_or_else(|| {
                    panic!("应用图标未找到，请检查 tauri.conf.json 中 bundle.icon 配置")
                });

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .tooltip("CrabProxy")
                .icon_as_template(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // 静默启动：隐藏主窗口到托盘
            if is_silent {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::config_cmd::get_all_configs,
            commands::config_cmd::load_config,
            commands::config_cmd::save_config,
            commands::config_cmd::delete_config,
            commands::config_cmd::rename_config,
            commands::proxy_cmd::switch_config,
            commands::proxy_cmd::toggle_all_ports,
            commands::proxy_cmd::update_port,
            commands::proxy_cmd::add_port,
            commands::proxy_cmd::remove_port,
            commands::proxy_cmd::get_proxy_status,
            commands::proxy_cmd::restore_last_session,
            commands::system_cmd::get_startup,
            commands::system_cmd::set_startup,
            commands::system_cmd::set_window_theme,
        ])
        .run(tauri::generate_context!())
        .expect("启动应用时发生错误");
}
