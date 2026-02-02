mod config;
mod windows;
use config::get_config;
use config::open_config_file;
use config::read_or_create_config;
use config::switch_model;
use selection::get_text;
use std::sync::{Arc, Mutex};
use tauri::menu::MenuItem;
use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, SubmenuBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_global_shortcut::Code;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_global_shortcut::Modifiers;
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





// 初始化托盘图标，接收 shortcut_enabled 状态用于控制菜单
fn init_tray(app: &tauri::App, shortcut_enabled: Arc<Mutex<bool>>) -> tauri::Result<()> {
    // ========== 1. 创建基础菜单项 ==========
    // 退出菜单项
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    // 配置菜单项
    let config_item = MenuItem::with_id(app, "config", "Config", true, None::<&str>)?;
    // 快捷键开关菜单项
    let shortcut_item = MenuItem::with_id(
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
        .item(&autostart_enable_item)
        .item(&quit_item) // 退出项
        .build()?; // 构建主菜单

    // 3. 克隆菜单项用于事件闭包
    let shortcut_item_clone = shortcut_item.clone();
    let deepseek_check_clone = deepseek_check_item.clone();
    let doubao_check_clone = doubao_check_item.clone();
    let autostart_enable_item_clone = autostart_enable_item.clone();

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
                let shortcut_key = Shortcut::new(Some(Modifiers::ALT), Code::F1);
                let mut enabled = shortcut_enabled.lock().unwrap();
                *enabled = !*enabled;

                if *enabled {
                    let _ = app.global_shortcut().register(shortcut_key);
                } else {
                    let _ = app.global_shortcut().unregister(shortcut_key);
                }

                let new_text = if *enabled {
                    "Disable Shortcut"
                } else {
                    "Enable Shortcut"
                };
                let _ = shortcut_item_clone.set_text(new_text);
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
fn setup_capslock_shortcut(app: &tauri::App) -> tauri::Result<()> {
    use tauri_plugin_global_shortcut::{
        Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
    };

    let shortcut_key = Shortcut::new(Some(Modifiers::ALT), Code::F1);

    let panel = app
        .get_webview_window("panel")
        .expect("Failed to get panel window");

    let panel_clone = panel.clone();

    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app_handle, shortcut, event| {
                if shortcut == &shortcut_key {
                    if let ShortcutState::Released = event.state() {
                        let text = get_text();
                        if !text.is_empty() {
                            let _ = panel_clone.emit("get_text", text);
                            let _ = panel_clone.show();
                            let _ = panel_clone.set_focus();
                        }
                    }
                }
            })
            .build(),
    )?;

    // 注册快捷键（监听始终存在，但是否响应由 enabled 控制）
    let _ = app.global_shortcut().register(shortcut_key);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
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
                setup_capslock_shortcut(app)?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            send_text, hide_panel, get_config, start_drag
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
