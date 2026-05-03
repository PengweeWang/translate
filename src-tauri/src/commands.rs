use crate::config;
use crate::prompt;
use crate::shortcut;
use crate::tray::AppState;
use std::sync::{Arc, Mutex};
use serde::Serialize;

use tauri_plugin_global_shortcut::GlobalShortcutExt;

/// 获取当前选中的文本
#[tauri::command]
pub fn send_text() -> String {
    selection::get_text()
}

/// 隐藏面板窗口
#[tauri::command]
pub async fn hide_panel(window: tauri::Window) {
    let _ = window.hide();
}

/// 开始窗口拖拽
#[tauri::command]
pub async fn start_drag(window: tauri::Window) {
    #[cfg(desktop)]
    {
        let _ = window.start_dragging();
    }
}

/// 获取当前配置
#[tauri::command]
pub async fn get_config() -> Result<config::Config, String> {
    match config::read_or_create_config() {
        Ok(cfg) => Ok(cfg),
        Err(e) => Err(format!("Failed to load config: {}", e)),
    }
}

/// 分别返回翻译指令 prompt 和用户原始输入
#[derive(Serialize)]
pub struct TranslatePrompt {
    pub prompt: String,
    pub input: String,
}

/// 根据文本类型返回对应的 prompt 指令（去除 ${text}）和原始输入文本
#[tauri::command]
pub async fn get_translate_prompt(text: String) -> Result<TranslatePrompt, String> {
    let cfg =
        config::read_or_create_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let template = prompt::select_template(&text, &cfg.select.word_prompt, &cfg.select.sentence_prompt);
    let prompt_clean = template.replace("${text}", "").trim().to_string();
    Ok(TranslatePrompt {
        prompt: prompt_clean,
        input: text,
    })
}

/// 返回指定主题的 CSS 内容，空字符串主题名返回 None（使用内置样式）
#[tauri::command]
pub async fn get_theme_css(theme_name: String) -> Result<Option<String>, String> {
    if theme_name.is_empty() {
        return Ok(None);
    }
    let mut path = config::get_theme_dir();
    path.push(format!("{}.css", theme_name));
    let css = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("Failed to read theme: {}", e)),
    };
    Ok(Some(css))
}

/// 设置自定义快捷键（格式如 "Alt+F1", "Ctrl+Shift+A"）
#[tauri::command]
pub async fn set_shortcut(
    app: tauri::AppHandle,
    shortcut_str: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    // 先验证快捷键格式是否合法（尝试注册来验证）
    app.global_shortcut().is_registered(&*shortcut_str);

    // 获取旧快捷键
    let old_shortcut = {
        let s = state.lock().unwrap();
        s.shortcut_key.clone()
    };

    // 更新配置文件
    config::update_shortcut(&shortcut_str)?;

    // 重新注册快捷键
    #[cfg(desktop)]
    {
        let enabled = state.lock().unwrap().shortcut_enabled;
        if enabled {
            shortcut::reregister_shortcut(&app, &old_shortcut, &shortcut_str)?;
        }
    }

    // 更新内存中的快捷键
    {
        let mut s = state.lock().unwrap();
        s.shortcut_key = shortcut_str.clone();
    }

    Ok(shortcut_str)
}
