use crate::config;
use std::sync::{Arc, Mutex};
use tauri::menu::MenuItem;
use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, SubmenuBuilder};
use tauri::tray::TrayIconBuilder;

use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_autostart::ManagerExt;

/// 应用共享状态
pub struct AppState {
    pub shortcut_enabled: bool,
    pub shortcut_key: String,
}

/// 初始化系统托盘图标及菜单
pub fn init_tray(app: &tauri::App, state: Arc<Mutex<AppState>>) -> tauri::Result<()> {
    // ========== 创建基础菜单项 ==========
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let config_item = MenuItem::with_id(app, "config", "Config", true, None::<&str>)?;
    let shortcut_text = {
        let s = state.lock().unwrap();
        if s.shortcut_enabled {
            "Disable Shortcut"
        } else {
            "Enable Shortcut"
        }
    };
    let shortcut_item = MenuItem::with_id(
        app,
        "shortcut",
        shortcut_text,
        true,
        None::<&str>,
    )?;

    // 获取当前选中的模型
    let current_model = config::read_or_create_config().unwrap_or_default().select.llm;

    // 创建带勾选状态的模型项
    let deepseek_check_item = CheckMenuItemBuilder::new("DeepSeek")
        .id("model_deepseek")
        .checked(current_model == "deepseek")
        .build(app)?;

    let doubao_check_item = CheckMenuItemBuilder::new("Doubao")
        .id("model_doubao")
        .checked(current_model == "doubao")
        .build(app)?;

    let autostart_enable_item = CheckMenuItemBuilder::new("Autostart")
        .id("autostart_enable")
        .checked(false)
        .build(app)?;

    // 构建 Model 子菜单
    let model_submenu = SubmenuBuilder::new(app, "Model")
        .item(&deepseek_check_item)
        .item(&doubao_check_item)
        .build()?;

    let main_menu = MenuBuilder::new(app)
        .item(&config_item)
        .item(&model_submenu)
        .item(&shortcut_item)
        .item(&autostart_enable_item)
        .item(&quit_item)
        .build()?;

    // 克隆菜单项用于事件闭包
    let shortcut_item_clone = shortcut_item.clone();
    let deepseek_check_clone = deepseek_check_item.clone();
    let doubao_check_clone = doubao_check_item.clone();
    let autostart_enable_item_clone = autostart_enable_item.clone();

    // 创建托盘图标并绑定菜单事件
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&main_menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "config" => {
                let _ = config::open_config_file(app);
            }
            "shortcut" => {
                let mut s = state.lock().unwrap();
                s.shortcut_enabled = !s.shortcut_enabled;

                if s.shortcut_enabled {
                    // 启用：注册快捷键
                    let _ = app.global_shortcut().register(&*s.shortcut_key);
                } else {
                    // 禁用：注销快捷键
                    let _ = app.global_shortcut().unregister(&*s.shortcut_key);
                }

                let new_text = if s.shortcut_enabled {
                    "Disable Shortcut"
                } else {
                    "Enable Shortcut"
                };
                let _ = shortcut_item_clone.set_text(new_text);
            }
            "model_deepseek" => {
                let _ = config::switch_model("deepseek");
                let _ = deepseek_check_clone.set_checked(true);
                let _ = doubao_check_clone.set_checked(false);
            }
            "model_doubao" => {
                let _ = config::switch_model("doubao");
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
