use crate::mouse_shortcut;
use crate::tray::AppState;
use selection::get_text;
use std::sync::{atomic::Ordering, Arc, Mutex};
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// 设置全局快捷键：从配置读取快捷键字符串，释放时获取选中文本并显示翻译面板
#[cfg(desktop)]
pub fn setup_shortcut(app: &tauri::App, shortcut_str: &str) -> tauri::Result<()> {
    let panel = app
        .get_webview_window("panel")
        .expect("Failed to get panel window");

    let panel_clone = panel.clone();

    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app_handle, _shortcut, event| {
                if let ShortcutState::Released = event.state() {
                    let text = get_text();
                    if !text.is_empty() {
                        let _ = panel_clone.emit("get_text", text);
                        let _ = panel_clone.show();
                        let _ = panel_clone.set_focus();
                    }
                }
            })
            .with_shortcut(shortcut_str)
            .map_err(|e| tauri::Error::Setup((Box::new(e) as Box<dyn std::error::Error>).into()))?
            .build(),
    )?;

    Ok(())
}

/// 重新注册快捷键（先注销旧的，再注册新的）
#[cfg(desktop)]
pub fn reregister_shortcut(
    app: &tauri::AppHandle,
    old_shortcut_str: &str,
    new_shortcut_str: &str,
    state: &Arc<Mutex<AppState>>,
) -> Result<(), String> {
    if let Some(flag) = state.lock().unwrap().mouse_stop.take() {
        flag.store(true, Ordering::Relaxed);
    }
    if !mouse_shortcut::is_mouse_shortcut(old_shortcut_str) {
        let _ = app.global_shortcut().unregister(old_shortcut_str);
    }
    if mouse_shortcut::is_mouse_shortcut(new_shortcut_str) {
        let stop = mouse_shortcut::start(app.clone(), new_shortcut_str);
        state.lock().unwrap().mouse_stop = Some(stop);
    } else {
        app.global_shortcut()
            .register(new_shortcut_str)
            .map_err(|e| format!("Failed to register shortcut '{}': {}", new_shortcut_str, e))?;
    }
    Ok(())
}
