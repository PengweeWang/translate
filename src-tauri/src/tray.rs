use crate::config;
use std::sync::{Arc, Mutex};
use tauri::menu::MenuItem;
use tauri::menu::{CheckMenuItem, CheckMenuItemBuilder, MenuBuilder, SubmenuBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Emitter;
use tauri::Wry;

use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_updater::UpdaterExt;

pub async fn check_and_prompt_update(app: &tauri::AppHandle, silent: bool) {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(_) => return,
    };
    let update = match updater.check().await {
        Ok(Some(u)) => u,
        Ok(None) => {
            if !silent {
                app.dialog().message("已是最新版本").title("检查更新").show(|_| {});
            }
            return;
        }
        Err(e) => {
            eprintln!("Update check failed: {}", e);
            return;
        }
    };

    let version = update.version.clone();
    let notes = update.body.clone().unwrap_or_default();
    let msg = format!("New version {} is available.\n\n{}\n\nInstall now?", version, notes);

    let (tx, rx) = std::sync::mpsc::channel::<bool>();
    app.dialog()
        .message(msg)
        .title("Update Available")
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Install".to_string(),
            "Later".to_string(),
        ))
        .show(move |confirmed| {
            let _ = tx.send(confirmed);
        });

    if rx.recv().unwrap_or(false) {
        if let Err(e) = update.download_and_install(|_, _| {}, || {}).await {
            eprintln!("Failed to install update: {}", e);
        }
    }
}

/// 应用共享状态
pub struct AppState {
    pub shortcut_enabled: bool,
    pub shortcut_key: String,
    pub auto_update_enabled: bool,
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
    let shortcut_item = MenuItem::with_id(app, "shortcut", shortcut_text, true, None::<&str>)?;

    // 获取当前选中的模型
    let config = config::read_or_create_config().unwrap_or_default();
    let current_model = config.select.llm.clone();
    let current_theme = config.select.theme.clone();

    // 创建带勾选状态的模型项
    let deepseek_check_item = CheckMenuItemBuilder::new("DeepSeek")
        .id("model_deepseek")
        .checked(current_model == "deepseek")
        .build(app)?;

    let doubao_check_item = CheckMenuItemBuilder::new("Doubao")
        .id("model_doubao")
        .checked(current_model == "doubao")
        .build(app)?;

    let autostart_state = app.autolaunch().is_enabled().unwrap_or(false);
    println!("Autostart enabled: {}", autostart_state);
    let autostart_enable_item = CheckMenuItemBuilder::new("Autostart")
        .id("autostart_enable")
        .checked(autostart_state)
        .build(app)?;

    let auto_update_enabled = {
        let s = state.lock().unwrap();
        s.auto_update_enabled
    };
    let auto_update_item = CheckMenuItemBuilder::new("Auto Update")
        .id("auto_update")
        .checked(auto_update_enabled)
        .build(app)?;

    let check_update_item = MenuItem::with_id(app, "check_update", "Check for Updates", true, None::<&str>)?;

    // 构建 Model 子菜单
    let model_submenu = SubmenuBuilder::new(app, "Model")
        .item(&deepseek_check_item)
        .item(&doubao_check_item)
        .build()?;

    // 动态构建 Theme 子菜单
    let themes = config::list_themes();
    let mut theme_items: Vec<CheckMenuItem<Wry>> = Vec::new();
    let mut theme_builder = SubmenuBuilder::new(app, "Theme");
    for name in &themes {
        let label = if name.is_empty() { "Default" } else { name.as_str() };
        let item = CheckMenuItemBuilder::new(label)
            .id(format!("theme_{}", name))
            .checked(*name == current_theme)
            .build(app)?;
        theme_builder = theme_builder.item(&item);
        theme_items.push(item);
    }
    let theme_submenu = theme_builder.build()?;

    let main_menu = MenuBuilder::new(app)
        .item(&config_item)
        .item(&model_submenu)
        .item(&theme_submenu)
        .item(&shortcut_item)
        .item(&auto_update_item)
        .item(&check_update_item)
        .item(&autostart_enable_item)
        .item(&quit_item)
        .build()?;

    // 克隆菜单项用于事件闭包
    let shortcut_item_clone = shortcut_item.clone();
    let deepseek_check_clone = deepseek_check_item.clone();
    let doubao_check_clone = doubao_check_item.clone();
    let autostart_enable_item_clone = autostart_enable_item.clone();
    let auto_update_item_clone = auto_update_item.clone();
    let theme_items_clone = theme_items.clone();

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
            "auto_update" => {
                let checked = auto_update_item_clone.is_checked().unwrap_or_default();
                let _ = config::set_auto_update(checked);
                let mut s = state.lock().unwrap();
                s.auto_update_enabled = checked;
            }
            "check_update" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    check_and_prompt_update(&app, false).await;
                });
            }
            _ => {
                let id = event.id.as_ref();
                if id.starts_with("theme_") {
                    let name = &id["theme_".len()..];
                    let _ = config::switch_theme(name);
                    for item in &theme_items_clone {
                        let item_name = item.id().as_ref().strip_prefix("theme_").unwrap_or("");
                        let _ = item.set_checked(item_name == name);
                    }
                    let _ = app.emit("theme-changed", name.to_string());
                } else {
                    println!("menu item {:?} not handled", event.id);
                }
            }
        })
        .build(app)?;

    Ok(())
}
