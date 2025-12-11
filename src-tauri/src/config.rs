use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri_plugin_opener::OpenerExt;

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
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeepSeekConfig {
    pub api_base: String,
    pub api_key: String,
    pub default_model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_temperature() -> f32 {
    1.3
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub select: SelectConfig,
    pub deepseek: DeepSeekConfig,
}

impl Default for DeepSeekConfig {
    fn default() -> Self {
        Self {
            api_base: "https://api.deepseek.com/v1".to_string(),
            api_key: "sk-<YOUR_API_KEY_HERE>".to_string(),
            default_model: "deepseek-chat".to_string(),
            temperature: 1.3,
        }
    }
}

impl Default for SelectConfig {
    fn default() -> Self {
        Self {
            llm: "deepseek".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            deepseek: DeepSeekConfig::default(),
            select: SelectConfig::default(),
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

[deepseek]
api_base = "https://api.deepseek.com/v1"
api_key = "sk-<YOUR_API_KEY_HERE>"
default_model = "deepseek-chat"
temperature = 1.3
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
