use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ai_service: AIServiceConfig,
    pub league_client: LeagueClientConfig,
    pub ui: UIConfig,
    pub notifications: NotificationConfig,
    pub auto_refresh: AutoRefreshConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIServiceConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub model_name: String,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeagueClientConfig {
    pub auto_detect: bool,
    pub manual_port: Option<u16>,
    pub manual_password: Option<String>,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub theme: Theme,
    pub language: String,
    pub window_size: WindowSize,
    pub navigation_rail_expanded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub sound_enabled: bool,
    pub champion_suggestions: bool,
    pub ability_recommendations: bool,
    pub game_events: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRefreshConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
    pub champion_select: bool,
    pub live_game: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ai_service: AIServiceConfig::default(),
            league_client: LeagueClientConfig::default(),
            ui: UIConfig::default(),
            notifications: NotificationConfig::default(),
            auto_refresh: AutoRefreshConfig::default(),
        }
    }
}

impl Default for AIServiceConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8000".to_string(),
            timeout_seconds: 60,
            model_name: "llama3.2:8b".to_string(),
            max_tokens: 2048,
        }
    }
}

impl Default for LeagueClientConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            manual_port: None,
            manual_password: None,
            connection_timeout: 5000,
        }
    }
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            language: "en".to_string(),
            window_size: WindowSize { width: 1200, height: 800 },
            navigation_rail_expanded: true,
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sound_enabled: true,
            champion_suggestions: true,
            ability_recommendations: true,
            game_events: true,
        }
    }
}

impl Default for AutoRefreshConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 30,
            champion_select: true,
            live_game: true,
        }
    }
}

pub struct ConfigManager {
    config: AppConfig,
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = Self::get_config_dir()?;
        let config_path = config_dir.join("wingbot_config.json");
        
        let config = if config_path.exists() {
            Self::load_from_file(&config_path)?
        } else {
            let default_config = AppConfig::default();
            Self::save_to_file(&config_path, &default_config)?;
            default_config
        };
        
        Ok(Self {
            config,
            config_path,
        })
    }

    fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = if cfg!(target_os = "windows") {
            let app_data = std::env::var("APPDATA")?;
            PathBuf::from(app_data).join("WingBot")
        } else {
            let home = std::env::var("HOME")?;
            PathBuf::from(home).join(".config").join("wingbot")
        };
        
        fs::create_dir_all(&config_dir)?;
        Ok(config_dir)
    }

    fn load_from_file(path: &PathBuf) -> Result<AppConfig, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    fn save_to_file(path: &PathBuf, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(config)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    pub fn update_config(&mut self, new_config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.config = new_config;
        ConfigManager::save_to_file(&self.config_path, &self.config)?;
        Ok(())
    }

    pub fn update_ai_service_config(&mut self, ai_config: AIServiceConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.config.ai_service = ai_config;
        ConfigManager::save_to_file(&self.config_path, &self.config)?;
        Ok(())
    }

    pub fn update_league_client_config(&mut self, lc_config: LeagueClientConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.config.league_client = lc_config;
        ConfigManager::save_to_file(&self.config_path, &self.config)?;
        Ok(())
    }

    pub fn update_ui_config(&mut self, ui_config: UIConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.config.ui = ui_config;
        ConfigManager::save_to_file(&self.config_path, &self.config)?;
        Ok(())
    }

    pub fn get_ai_service_url(&self) -> &str {
        &self.config.ai_service.base_url
    }

    pub fn get_ai_service_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.config.ai_service.timeout_seconds)
    }

    pub fn is_auto_detect_enabled(&self) -> bool {
        self.config.league_client.auto_detect
    }

    pub fn get_manual_connection_info(&self) -> Option<(u16, String)> {
        match (self.config.league_client.manual_port, &self.config.league_client.manual_password) {
            (Some(port), Some(password)) => Some((port, password.clone())),
            _ => None,
        }
    }

    pub fn get_theme(&self) -> &Theme {
        &self.config.ui.theme
    }

    pub fn get_window_size(&self) -> &WindowSize {
        &self.config.ui.window_size
    }

    pub fn is_navigation_rail_expanded(&self) -> bool {
        self.config.ui.navigation_rail_expanded
    }

    pub fn are_notifications_enabled(&self) -> bool {
        self.config.notifications.enabled
    }

    pub fn is_auto_refresh_enabled(&self) -> bool {
        self.config.auto_refresh.enabled
    }

    pub fn get_refresh_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.config.auto_refresh.interval_seconds)
    }
}

// Global configuration instance
static CONFIG_MANAGER: OnceLock<Mutex<ConfigManager>> = OnceLock::new();

pub fn get_config_manager() -> &'static Mutex<ConfigManager> {
    CONFIG_MANAGER.get_or_init(|| {
        Mutex::new(ConfigManager::new().unwrap_or_else(|_| {
            eprintln!("Failed to load config, using defaults");
            ConfigManager::new().unwrap_or_else(|_| {
                panic!("Failed to create config manager")
            })
        }))
    })
}
