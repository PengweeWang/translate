use selection::get_text;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
mod windows;
use tauri::Emitter;
use windows::panel;
mod config;
use config::get_config;
use config::open_config_file;

#[tauri::command]
fn send_text() -> String {
    let text = get_text();
    println!("{:?}", text);
    text
}

#[tauri::command]
async fn hide_panel(window: tauri::Window) {
    let _ = window.hide();
}

#[tauri::command]
async fn start_drag(window: tauri::Window) {
    #[cfg(desktop)]
    {
        let _ = window.start_dragging();
    }
}

// 初始化托盘图标，接收 shortcut_enabled 状态用于控制菜单
fn init_tray(app: &tauri::App, shortcut_enabled: Arc<Mutex<bool>>) -> tauri::Result<()> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let config_i = MenuItem::with_id(app, "config", "Config", true, None::<&str>)?;
    // 初始状态为启用，所以显示 "Disable Shortcut"
    let shortcut_i = MenuItem::with_id(
        app,
        "shortcut",
        if *shortcut_enabled.lock().unwrap() {
            "Disable Shortcut"
        } else {
            "Enable Shortcut"
        },
        true,
        None::<&str>,
    )?;

    let menu = Menu::with_items(app, &[&config_i, &shortcut_i, &quit_i])?;

    let shortcut_i_clone = shortcut_i.clone();
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "config" => {
                let _ = open_config_file(app);
            }
            "shortcut" => {
                let mut enabled = shortcut_enabled.lock().unwrap();
                *enabled = !*enabled;

                let new_text = if *enabled {
                    "Disable Shortcut"
                } else {
                    "Enable Shortcut"
                };
                let _ = shortcut_i_clone.set_text(new_text);
            }
            _ => {
                println!("menu item {:?} not handled", event.id);
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg(desktop)]
fn setup_capslock_shortcut(app: &tauri::App, enabled: Arc<Mutex<bool>>) -> tauri::Result<()> {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

    let caps_lock = Shortcut::new(None, Code::CapsLock);

    // 共享状态：记录上次按下时间
    let last_press = Arc::new(Mutex::new(Option::<Instant>::None));
    let last_press_clone = last_press.clone();

    let panel = app
        .get_webview_window("panel")
        .expect("Failed to get panel window");

    let panel_clone = panel.clone();
    let enabled_clone = enabled.clone();

    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app_handle, shortcut, event| {
                // 检查是否启用快捷键
                if !*enabled_clone.lock().unwrap() {
                    return;
                }

                if shortcut == &caps_lock {
                    if let ShortcutState::Pressed = event.state() {
                        let now = Instant::now();
                        let mut last = last_press_clone.lock().unwrap();

                        if let Some(last_time) = *last {
                            let elapsed = now.duration_since(last_time);
                            if elapsed < Duration::from_millis(300) {
                                let text = get_text();
                                if !text.is_empty() {
                                    let _ = panel_clone.emit("get_text", text);
                                    let _ = panel_clone.show();
                                }
                            }
                            *last = None; // 防止三连击误触发
                        } else {
                            *last = Some(now);
                        }
                    }
                }
            })
            .build(),
    )?;

    // 注册快捷键（监听始终存在，但是否响应由 enabled 控制）
    let _ = app.global_shortcut().register(caps_lock);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle();

            // 共享状态：是否启用快捷键
            let shortcut_enabled = Arc::new(Mutex::new(true));

            // 初始化托盘菜单
            init_tray(app, shortcut_enabled.clone())?;

            // 创建 panel 窗口
            panel(&handle.clone());

            #[cfg(desktop)]
            {
                // 设置 CapsLock 双击监听（逻辑由 shortcut_enabled 控制）
                setup_capslock_shortcut(app, shortcut_enabled.clone())?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![send_text, hide_panel, get_config, start_drag])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
