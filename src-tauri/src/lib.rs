mod commands;
mod config;
mod error;
mod proxy;
mod startup;

use proxy::engine::create_shared_engine;
use tauri::Manager;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
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

            // 构建系统托盘图标
            // 使用应用默认图标（来自 bundle 配置），确保 Windows/macOS/Linux 托盘图标正常显示
            let tray_icon = app
                .default_window_icon()
                .cloned()
                .unwrap_or_else(|| {
                    // 回退：编译时嵌入 32x32 PNG → 原始 RGBA 像素
                    // 此宏由 tauri-codegen 在编译时处理
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
                    // 双击托盘图标显示窗口
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

            Ok(())
        })
        // 窗口关闭时隐藏到托盘而非退出
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // 阻止关闭，隐藏到托盘
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            // 配置命令
            commands::config_cmd::get_all_configs,
            commands::config_cmd::load_config,
            commands::config_cmd::save_config,
            commands::config_cmd::delete_config,
            commands::config_cmd::rename_config,
            // 代理命令
            commands::proxy_cmd::switch_config,
            commands::proxy_cmd::toggle_all_ports,
            commands::proxy_cmd::update_port,
            commands::proxy_cmd::add_port,
            commands::proxy_cmd::remove_port,
            commands::proxy_cmd::get_proxy_status,
            commands::proxy_cmd::restore_last_session,
            // 系统命令
            commands::system_cmd::get_startup,
            commands::system_cmd::set_startup,
            commands::system_cmd::set_window_theme,
        ])
        .run(tauri::generate_context!())
        .expect("启动应用时发生错误");
}
