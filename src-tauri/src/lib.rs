mod config;
mod windows;
use config::get_config;
use config::DEFAULT_SHORTCUT_TRIGGER;
use config::open_config_file;
use config::read_or_create_config;
use config::switch_model;
use selection::get_text;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tauri::menu::MenuItem;
use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, SubmenuBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_global_shortcut::Shortcut;
use windows::panel;
use tauri_plugin_autostart::ManagerExt;


#[tauri::command]
fn send_text() -> String {
    let text = get_text();
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

#[derive(Clone)]
struct ShortcutRuntimeState {
    shortcut: Shortcut,
    shortcut_text: String,
    enabled: bool,
}

fn parse_shortcut_with_fallback(raw_shortcut: &str) -> (Shortcut, String) {
    let normalized = raw_shortcut.trim();

    match Shortcut::from_str(normalized) {
        Ok(shortcut) => (shortcut, normalized.to_string()),
        Err(err) => {
            eprintln!(
                "Invalid shortcut '{}': {}. Fallback to {}",
                normalized, err, DEFAULT_SHORTCUT_TRIGGER
            );
            let fallback = Shortcut::from_str(DEFAULT_SHORTCUT_TRIGGER)
                .expect("DEFAULT_SHORTCUT_TRIGGER must be valid");
            (fallback, DEFAULT_SHORTCUT_TRIGGER.to_string())
        }
    }
}

fn load_shortcut_runtime_state() -> ShortcutRuntimeState {
    let config = read_or_create_config().unwrap_or_else(|err| {
        eprintln!("Failed to read config for shortcut: {}. Use defaults.", err);
        Default::default()
    });

    let (shortcut, shortcut_text) =
        parse_shortcut_with_fallback(&config.shortcuts.trigger_translation);

    ShortcutRuntimeState {
        shortcut,
        shortcut_text,
        enabled: config.shortcuts.enabled_by_default,
    }
}





// 初始化托盘图标，接收 shortcut_enabled 状态用于控制菜单
fn init_tray(app: &tauri::App, shortcut_state: Arc<Mutex<ShortcutRuntimeState>>) -> tauri::Result<()> {
    // ========== 1. 创建基础菜单项 ==========
    // 退出菜单项
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    // 配置菜单项
    let config_item = MenuItem::with_id(app, "config", "Config", true, None::<&str>)?;
    // 快捷键开关菜单项
    let shortcut_item = MenuItem::with_id(
        app,
        "shortcut",
        if shortcut_state.lock().unwrap().enabled {
            "Disable Shortcut"
        } else {
            "Enable Shortcut"
        },
        true,
        None::<&str>,
    )?;
    let reload_shortcut_item = MenuItem::with_id(
        app,
        "reload_shortcut",
        "Reload Shortcut",
        true,
        None::<&str>,
    )?;

    // 获取当前选中的模型
    let current_model = read_or_create_config().unwrap_or_default().select.llm;

    // 创建带勾选状态的模型项
    let deepseek_check_item = CheckMenuItemBuilder::new("DeepSeek")
        .id("model_deepseek")
        .checked(current_model == "deepseek") // 原生勾选状态，无需手动加✓
        .build(app)?;

    let doubao_check_item = CheckMenuItemBuilder::new("Doubao")
        .id("model_doubao")
        .checked(current_model == "doubao") // 原生勾选状态
        .build(app)?;

    let autostart_enable_item = CheckMenuItemBuilder::new("Autostart")
        .id("autostart_enable").checked(false).build(app)?;

    // 构建Model子菜单
    let model_submenu = SubmenuBuilder::new(app, "Model") // 子菜单名称：Model
        .item(&deepseek_check_item) // 添加DeepSeek勾选项
        .item(&doubao_check_item) // 添加Doubao勾选项
        .build()?; // 构建子菜单

    let main_menu = MenuBuilder::new(app)
        .item(&config_item) // 配置项
        .item(&model_submenu) // Model子菜单（多级核心）
        .item(&shortcut_item) // 快捷键开关
        .item(&reload_shortcut_item)
        .item(&autostart_enable_item)
        .item(&quit_item) // 退出项
        .build()?; // 构建主菜单

    // 3. 克隆菜单项用于事件闭包
    let shortcut_item_clone = shortcut_item.clone();
    let deepseek_check_clone = deepseek_check_item.clone();
    let doubao_check_clone = doubao_check_item.clone();
    let autostart_enable_item_clone = autostart_enable_item.clone();
    let shortcut_state_clone = shortcut_state.clone();

    // 4. 创建托盘图标并绑定菜单事件
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone()) // 托盘图标
        .menu(&main_menu) // 绑定主菜单
        .on_menu_event(move |app, event| match event.id.as_ref() {
            // 退出程序
            "quit" => {
                app.exit(0);
            }
            // 打开配置
            "config" => {
                let _ = open_config_file(app);
            }
            // 切换快捷键开关
            "shortcut" => {
                let mut state = shortcut_state_clone.lock().unwrap();
                let previous_enabled = state.enabled;
                state.enabled = !state.enabled;

                let operation = if state.enabled {
                    app.global_shortcut().register(state.shortcut.clone())
                } else {
                    app.global_shortcut().unregister(state.shortcut.clone())
                };

                if let Err(err) = operation {
                    eprintln!(
                        "Failed to {} shortcut {}: {}",
                        if state.enabled { "register" } else { "unregister" },
                        state.shortcut_text,
                        err
                    );
                    state.enabled = previous_enabled;
                }

                let new_text = if state.enabled {
                    "Disable Shortcut"
                } else {
                    "Enable Shortcut"
                };
                let _ = shortcut_item_clone.set_text(new_text);
            }
            "reload_shortcut" => {
                let config = match read_or_create_config() {
                    Ok(cfg) => cfg,
                    Err(err) => {
                        eprintln!("Failed to reload config: {}", err);
                        return;
                    }
                };

                let (new_shortcut, new_shortcut_text) =
                    parse_shortcut_with_fallback(&config.shortcuts.trigger_translation);
                let mut state = shortcut_state_clone.lock().unwrap();
                let was_enabled = state.enabled;

                if was_enabled {
                    if let Err(err) = app.global_shortcut().unregister(state.shortcut.clone()) {
                        eprintln!(
                            "Failed to unregister old shortcut {}: {}",
                            state.shortcut_text, err
                        );
                    }
                }

                state.shortcut = new_shortcut;
                state.shortcut_text = new_shortcut_text;
                state.enabled = config.shortcuts.enabled_by_default;

                if state.enabled {
                    if let Err(err) = app.global_shortcut().register(state.shortcut.clone()) {
                        eprintln!(
                            "Failed to register reloaded shortcut {}: {}",
                            state.shortcut_text, err
                        );
                    }
                }

                let new_text = if state.enabled {
                    "Disable Shortcut"
                } else {
                    "Enable Shortcut"
                };
                let _ = shortcut_item_clone.set_text(new_text);
                println!(
                    "Shortcut reloaded: {} (enabled: {})",
                    state.shortcut_text, state.enabled
                );
            }
            // 选择DeepSeek模型
            "model_deepseek" => {
                let _ = switch_model("deepseek");
                // 更新原生勾选状态（无需改文本）
                let _ = deepseek_check_clone.set_checked(true);
                let _ = doubao_check_clone.set_checked(false);
            }
            // 选择Doubao模型
            "model_doubao" => {
                let _ = switch_model("doubao");
                // 更新原生勾选状态
                let _ = deepseek_check_clone.set_checked(false);
                let _ = doubao_check_clone.set_checked(true);
            }
            "autostart_enable" => {
                let auto_state = autostart_enable_item_clone.is_checked().unwrap_or_default();
                let autostart_manager = app.autolaunch();
                if auto_state {
                    let _ = autostart_manager.enable();
                } else {
                    let _ = autostart_manager.disable();
                }

            }
            _ => {
                println!("menu item {:?} not handled", event.id);
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg(desktop)]
fn setup_global_shortcut(app: &tauri::App, shortcut_state: Arc<Mutex<ShortcutRuntimeState>>) -> tauri::Result<()> {
    use tauri_plugin_global_shortcut::{
        GlobalShortcutExt, ShortcutState,
    };

    let panel = app
        .get_webview_window("panel")
        .expect("Failed to get panel window");

    let panel_clone = panel.clone();
    let shortcut_state_for_handler = shortcut_state.clone();

    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app_handle, shortcut, event| {
                let should_trigger = if let ShortcutState::Released = event.state() {
                    let state = shortcut_state_for_handler.lock().unwrap();
                    state.enabled && shortcut == &state.shortcut
                } else {
                    false
                };

                if should_trigger {
                    let text = get_text();
                    if !text.is_empty() {
                        let _ = panel_clone.emit("get_text", text);
                        let _ = panel_clone.show();
                        let _ = panel_clone.set_focus();
                    }
                }
            })
            .build(),
    )?;

    let state = shortcut_state.lock().unwrap();
    if state.enabled {
        if let Err(err) = app.global_shortcut().register(state.shortcut.clone()) {
            eprintln!(
                "Failed to register initial shortcut {}: {}",
                state.shortcut_text, err
            );
        }
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle();

            // 共享状态：快捷键内容 + 是否启用
            let shortcut_state = Arc::new(Mutex::new(load_shortcut_runtime_state()));

            // 初始化托盘菜单
            init_tray(app, shortcut_state.clone())?;

            // 创建 panel 窗口
            panel(&handle.clone());

            #[cfg(desktop)]
            {
                // 按配置设置全局快捷键
                setup_global_shortcut(app, shortcut_state.clone())?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            send_text, hide_panel, get_config, start_drag
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
