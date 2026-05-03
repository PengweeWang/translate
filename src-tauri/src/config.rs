use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri_plugin_opener::OpenerExt;

// ========== 配置数据结构 ==========

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct SelectConfig {
    pub llm: String,
    /// 全局快捷键（格式如 "Alt+F1", "Ctrl+Shift+A"）
    pub shortcut: String,
    /// 当前主题名（空字符串 = 默认内置主题）
    pub theme: String,
    /// 单词 prompt 模板（当检测到输入为单个英文单词时使用）
    pub word_prompt: String,
    /// 句子 prompt 模板（当检测到输入为短语/句子/段落时使用）
    pub sentence_prompt: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct DeepSeekConfig {
    pub api_base: String,
    pub api_key: String,
    pub default_model: String,
    pub temperature: f32,
    pub thinking: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct DoubaoConfig {
    pub api_base: String,
    pub api_key: String,
    pub default_model: String,
    pub temperature: f32,
    pub thinking: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct GeneralConfig {
    /// 是否自动检查更新
    pub auto_update: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct Config {
    pub select: SelectConfig,
    pub deepseek: DeepSeekConfig,
    pub doubao: DoubaoConfig,
    pub general: GeneralConfig,
}

// ========== 默认值函数 ==========

fn default_shortcut() -> String {
    "Alt+F1".to_string()
}

fn default_word_prompt() -> String {
    "你是一个专业词典助手。用户会发送一个单词，请提供其词典释义。\n\n输出格式（Markdown，不要输出其他内容）：\n\n## <单词原形>\n\n<词性缩写>. 释义1；释义2；...".to_string()
}

fn default_sentence_prompt() -> String {
    "你是一个专业翻译助手。将用户发送的内容准确、流畅地翻译成简体中文，只输出译文，不要解释。".to_string()
}

// ========== Default 实现 ==========

impl Default for SelectConfig {
    fn default() -> Self {
        Self {
            llm: "deepseek".to_string(),
            shortcut: default_shortcut(),
            theme: String::new(),
            word_prompt: default_word_prompt(),
            sentence_prompt: default_sentence_prompt(),
        }
    }
}

impl Default for DeepSeekConfig {
    fn default() -> Self {
        Self {
            api_base: "https://api.deepseek.com/v1".to_string(),
            api_key: "sk-<YOUR_API_KEY_HERE>".to_string(),
            default_model: "deepseek-chat".to_string(),
            temperature: 1.3,
            thinking: "disabled".to_string(),
        }
    }
}

impl Default for DoubaoConfig {
    fn default() -> Self {
        Self {
            api_base: "https://ark.cn-beijing.volces.com/api/v3".to_string(),
            api_key: "<YOUR_API_KEY_HERE>".to_string(),
            default_model: "doubao-seed-1-6-251015".to_string(),
            temperature: 1.3,
            thinking: "disabled".to_string(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            auto_update: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            select: SelectConfig::default(),
            deepseek: DeepSeekConfig::default(),
            doubao: DoubaoConfig::default(),
            general: GeneralConfig::default(),
        }
    }
}

// ========== 配置文件 I/O ==========

/// 获取配置文件路径：~/.local/translate/config.toml
fn get_config_path() -> PathBuf {
    let mut path = dirs::home_dir().expect("Failed to get home directory");
    path.push(".local");
    path.push("translate");
    path.push("config.toml");
    path
}

/// 读取配置，若不存在则创建默认配置
pub fn read_or_create_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    let config_dir = config_path
        .parent()
        .expect("Config path should have a parent");

    if !config_dir.exists() {
        fs::create_dir_all(config_dir)?;
    }

    if !config_path.exists() {
        let default_config = Config::default();
        let toml = toml::to_string_pretty(&default_config)?;
        fs::write(&config_path, toml)?;
        println!("Created default config at: {:?}", config_path);
        return Ok(default_config);
    }

    let contents = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&contents)?;

    // 升级：重新序列化并与原文件比对，若不一致说明填充了缺失字段，写回完整配置
    let serialized = toml::to_string_pretty(&config)?;
    if serialized != contents {
        fs::write(&config_path, &serialized)?;
        println!("Config migration applied at: {:?}", config_path);
    }

    Ok(config)
}

/// 使用系统默认编辑器打开配置文件
pub fn open_config_file(app: &tauri::AppHandle) -> Result<(), String> {
    let config_path = get_config_path();

    if !config_path.exists() {
        std::fs::create_dir_all(config_path.parent().unwrap())
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
        let default_config = Config::default();
        let toml_content = toml::to_string_pretty(&default_config)
            .map_err(|e| format!("Failed to serialize default config: {}", e))?;
        std::fs::write(&config_path, toml_content)
            .map_err(|e| format!("Failed to create config file: {}", e))?;
    }

    let _ = app
        .opener()
        .open_path(config_path.to_string_lossy(), None::<&str>);

    Ok(())
}

/// 切换当前使用的 LLM 模型
pub fn switch_model(model_name: &str) -> Result<(), String> {
    let mut config =
        read_or_create_config().map_err(|e| format!("Failed to read config: {}", e))?;

    config.select.llm = model_name.to_string();

    let config_path = get_config_path();
    let toml_content = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&config_path, toml_content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

/// 更新全局快捷键
pub fn update_shortcut(shortcut: &str) -> Result<(), String> {
    let mut config =
        read_or_create_config().map_err(|e| format!("Failed to read config: {}", e))?;

    config.select.shortcut = shortcut.to_string();

    let config_path = get_config_path();
    let toml_content = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&config_path, toml_content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

/// 获取主题目录路径：~/.local/translate/theme/
pub fn get_theme_dir() -> PathBuf {
    let mut path = dirs::home_dir().expect("Failed to get home directory");
    path.push(".local/translate/theme");
    path
}

/// 列出可用主题名（不含扩展名），空字符串排首位代表默认主题
pub fn list_themes() -> Vec<String> {
    let mut themes = vec![String::new()];
    let dir = get_theme_dir();
    if let Ok(entries) = fs::read_dir(&dir) {
        let mut names: Vec<String> = entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let p = e.path();
                if p.extension()?.to_str()? == "css" {
                    p.file_stem()?.to_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect();
        names.sort();
        themes.extend(names);
    }
    themes
}

/// 切换当前主题
pub fn switch_theme(theme_name: &str) -> Result<(), String> {
    let mut config =
        read_or_create_config().map_err(|e| format!("Failed to read config: {}", e))?;
    config.select.theme = theme_name.to_string();
    let config_path = get_config_path();
    let toml_content = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(&config_path, toml_content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    Ok(())
}

/// 设置是否自动检查更新
pub fn set_auto_update(enabled: bool) -> Result<(), String> {
    let mut config =
        read_or_create_config().map_err(|e| format!("Failed to read config: {}", e))?;

    config.general.auto_update = enabled;

    let config_path = get_config_path();
    let toml_content = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&config_path, toml_content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}
