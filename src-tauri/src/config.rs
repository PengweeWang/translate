use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri_plugin_opener::OpenerExt;

pub const DEFAULT_SHORTCUT_TRIGGER: &str = "Alt+W";

#[tauri::command]
pub async fn get_config() -> Result<Config, String> {
    match read_or_create_config() {
        Ok(cfg) => Ok(cfg),
        Err(e) => Err(format!("Failed to load config: {}", e)),
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SelectConfig {
    pub llm: String,
    pub prompt: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeepSeekConfig {
    pub api_base: String,
    pub api_key: String,
    pub default_model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    pub thinking: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DoubaoConfig {
    pub api_base: String,
    pub api_key: String,
    pub default_model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    pub thinking: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ShortcutConfig {
    #[serde(default = "default_shortcut_trigger")]
    pub trigger_translation: String,
    #[serde(default = "default_shortcut_enabled")]
    pub enabled_by_default: bool,
}

fn default_temperature() -> f32 {
    1.3
}

fn default_shortcut_trigger() -> String {
    DEFAULT_SHORTCUT_TRIGGER.to_string()
}

fn default_shortcut_enabled() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub select: SelectConfig,
    pub deepseek: DeepSeekConfig,
    pub doubao: DoubaoConfig,
    #[serde(default)]
    pub shortcuts: ShortcutConfig,
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

impl Default for SelectConfig {
    fn default() -> Self {
        Self {
            llm: "deepseek".to_string(),
            prompt: "请将以下内容准确、流畅地翻译成简体中文：\n\n${text}".to_string(),
        }
    }
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            trigger_translation: default_shortcut_trigger(),
            enabled_by_default: default_shortcut_enabled(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            deepseek: DeepSeekConfig::default(),
            select: SelectConfig::default(),
            doubao: DoubaoConfig::default(),
            shortcuts: ShortcutConfig::default(),
        }
    }
}

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

    // 如果目录不存在，创建它
    if !config_dir.exists() {
        fs::create_dir_all(config_dir)?;
    }

    // 如果配置文件不存在，写入默认配置
    if !config_path.exists() {
        let default_config = Config::default();
        let toml = toml::to_string_pretty(&default_config)?;
        fs::write(&config_path, toml)?;
        println!("Created default config at: {:?}", config_path);
    }

    // 读取并解析配置文件
    let contents = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

pub fn open_config_file(app: &tauri::AppHandle) -> Result<(), String> {
    // 构建配置文件路径（与 config.rs 一致）
    let config_path = get_config_path();

    // 确保文件存在（如果不存在，先创建）
    if !config_path.exists() {
        // 调用你的配置初始化逻辑（或简单创建目录+文件）
        std::fs::create_dir_all(config_path.parent().unwrap())
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
        // 写入默认内容（可选：调用 config::read_or_create_config() 会自动创建）
        let default_toml = r#"[select]
llm = "deepseek"
prompt = "请将以下内容准确、流畅地翻译成简体中文：\n\n${text}"

[deepseek]
api_base = "https://api.deepseek.com/v1"
api_key = "sk-<YOUR_API_KEY_HERE>"
default_model = "deepseek-chat"
temperature = 1.3
thinking = "disabled"

[doubao]
api_base = "https://ark.cn-beijing.volces.com/api/v3"
api_key = "<your token>"
default_model = "doubao-seed-1-6-251015"
temperature = 1.0
thinking = "disabled"

[shortcuts]
trigger_translation = "Alt+W"
enabled_by_default = true
"#;
        std::fs::write(&config_path, default_toml)
            .map_err(|e| format!("Failed to create config file: {}", e))?;
    }

    // 使用 opener 插件打开文件
    let _ = app
        .opener()
        .open_path(config_path.to_string_lossy(), None::<&str>);

    Ok(())
}

// pub fn get_available_models() -> Vec<String> {
//     vec!["deepseek".to_string(), "doubao".to_string()]
// }

pub fn switch_model(model_name: &str) -> Result<(), String> {
    // 读取当前配置
    let mut config =
        read_or_create_config().map_err(|e| format!("Failed to read config: {}", e))?;

    // 更新选择的模型
    config.select.llm = model_name.to_string();

    // 将更新后的配置写回文件
    let config_path = get_config_path();
    let toml_content = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&config_path, toml_content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}
