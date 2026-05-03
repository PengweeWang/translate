mod commands;
mod config;
mod prompt;
mod shortcut;
mod tray;
mod windows;

use std::sync::{Arc, Mutex};
use tauri::Manager;
use tray::AppState;
use windows::panel;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle();

            // 读取配置中的快捷键
            let shortcut_key = config::read_or_create_config()
                .map(|c| c.select.shortcut)
                .unwrap_or_else(|_| "Alt+F1".to_string());

            let auto_update = config::read_or_create_config()
                .map(|c| c.general.auto_update)
                .unwrap_or(true);

            // 共享状态
            let state = Arc::new(Mutex::new(AppState {
                shortcut_enabled: true,
                shortcut_key: shortcut_key.clone(),
                auto_update_enabled: auto_update,
            }));

            // 初始化托盘菜单
            tray::init_tray(app, state.clone())?;

            // 创建 panel 窗口
            panel(&handle.clone());

            #[cfg(desktop)]
            {
                // 设置全局快捷键
                shortcut::setup_shortcut(app, &shortcut_key)?;
            }

            // 将共享状态注入 Tauri 管理
            app.manage(state);

            // 启动时自动检查更新
            if auto_update {
                let handle = handle.clone();
                tauri::async_runtime::spawn(async move {
                    tray::check_and_prompt_update(&handle, true).await;
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::send_text,
            commands::hide_panel,
            commands::start_drag,
            commands::get_config,
            commands::get_translate_prompt,
            commands::get_theme_css,
            commands::set_shortcut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
