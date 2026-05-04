mod commands;
mod config;
mod mouse_shortcut;
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
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(panel) = app.get_webview_window("panel") {
                let _ = panel.show();
                let _ = panel.set_focus();
            }
        }))
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
                mouse_stop: None,
            }));

            // 初始化托盘菜单
            tray::init_tray(app, state.clone())?;

            // 先检查更新，完成后再注册快捷键
            if auto_update {
                let handle = handle.clone();
                tauri::async_runtime::block_on(async move {
                    tray::check_and_prompt_update(&handle, true).await;
                });
            }

            // 创建 panel 窗口
            panel(&handle.clone());

            #[cfg(desktop)]
            {
                if mouse_shortcut::is_mouse_shortcut(&shortcut_key) {
                    let stop = mouse_shortcut::start(app.handle().clone(), &shortcut_key);
                    state.lock().unwrap().mouse_stop = Some(stop);
                } else {
                    shortcut::setup_shortcut(app, &shortcut_key)?;
                }
            }

            // 将共享状态注入 Tauri 管理
            app.manage(state);

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
