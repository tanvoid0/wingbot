use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub league_client: LeagueClientConfig,
    #[serde(default)]
    pub riot_api: RiotApiConfig,
    #[serde(default)]
    pub ollama: OllamaConfig,
    pub ui: UIConfig,
    pub notifications: NotificationConfig,
    pub auto_refresh: AutoRefreshConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub enabled: bool,
    pub base_url: String,
    pub model_name: String,
    pub poll_interval_sec: u64,
    /// Live in-game / champ select coaching cards.
    #[serde(default = "default_true")]
    pub live_coach: bool,
    #[serde(default = "default_true")]
    pub insights_match_history: bool,
    #[serde(default = "default_true")]
    pub insights_scout: bool,
    #[serde(default = "default_true")]
    pub insights_rune: bool,
    #[serde(default = "default_true")]
    pub insights_lobby: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiotApiConfig {
    /// Developer API key from https://developer.riotgames.com/
    pub api_key: Option<String>,
    /// Platform shard (e.g. na1, euw1) — used to derive routing region.
    pub platform: String,
    /// Match-V5 routing region: americas, europe, asia, sea.
    pub routing_region: String,
    pub enabled: bool,
    /// ISO-8601 timestamp of the last successful Riot API call.
    #[serde(default)]
    pub last_verified_at: Option<String>,
    /// Last Riot API error message (used to detect expired dev keys).
    #[serde(default)]
    pub last_error: Option<String>,
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

impl Default for RiotApiConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            platform: "na1".to_string(),
            routing_region: String::new(),
            enabled: false,
            last_verified_at: None,
            last_error: None,
        }
    }
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "http://127.0.0.1:11434".to_string(),
            model_name: "wingbot-lol".to_string(),
            poll_interval_sec: 30,
            live_coach: true,
            insights_match_history: true,
            insights_scout: true,
            insights_rune: true,
            insights_lobby: true,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            league_client: LeagueClientConfig::default(),
            riot_api: RiotApiConfig::default(),
            ollama: OllamaConfig::default(),
            ui: UIConfig::default(),
            notifications: NotificationConfig::default(),
            auto_refresh: AutoRefreshConfig::default(),
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
        let old = self.config.clone();
        let mut merged = new_config;

        let old_key = old
            .riot_api
            .api_key
            .as_deref()
            .map(sanitize_riot_api_key);
        let new_key = merged
            .riot_api
            .api_key
            .as_deref()
            .map(sanitize_riot_api_key);

        if old_key == new_key {
            // Verification state is owned by the backend (test + match fetches).
            merged.riot_api.last_error = old.riot_api.last_error;
            merged.riot_api.last_verified_at = old.riot_api.last_verified_at;
        } else {
            merged.riot_api.last_error = None;
            merged.riot_api.last_verified_at = None;
        }

        if merged.riot_api.routing_region.is_empty()
            && !old.riot_api.routing_region.is_empty()
        {
            merged.riot_api.routing_region = old.riot_api.routing_region;
        }

        self.config = merged;
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

    pub fn update_ollama_config(&mut self, ollama_config: OllamaConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.config.ollama = ollama_config;
        ConfigManager::save_to_file(&self.config_path, &self.config)?;
        Ok(())
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

    pub fn record_riot_api_verification(
        &mut self,
        success: bool,
        error: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if success {
            self.config.riot_api.last_error = None;
            self.config.riot_api.last_verified_at =
                Some(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
        } else if let Some(message) = error {
            self.config.riot_api.last_error = Some(message);
        }
        ConfigManager::save_to_file(&self.config_path, &self.config)
    }
}

/// Strip whitespace/quotes and invisible characters from pasted API keys.
pub fn sanitize_riot_api_key(key: &str) -> String {
    key.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{feff}' && *c != '\u{200b}')
        .collect()
}

fn is_placeholder_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    lower.contains("your_riot")
        || lower.contains("your_key")
        || lower == "rgapi-..."
        || lower.len() < 20
}

/// Sanitize the Riot API key saved in Settings (Settings is the only source).
pub fn resolve_riot_config(saved: &RiotApiConfig) -> RiotApiConfig {
    let mut config = saved.clone();

    if let Some(ref key) = config.api_key {
        let cleaned = sanitize_riot_api_key(key);
        if cleaned.is_empty() || is_placeholder_key(&cleaned) {
            config.api_key = None;
        } else {
            config.api_key = Some(cleaned);
        }
    }

    if config.api_key.as_ref().is_some_and(|k| !k.is_empty()) {
        config.enabled = true;
    }

    config
}

pub fn is_riot_key_expired_error(error: &str) -> bool {
    let lower = error.to_lowercase();
    lower.contains("exception decrypting")
        || lower.contains("unknown apikey")
        || lower.contains("expired")
        || lower.contains("http 401")
        || lower.contains("http 403")
}

/// Whether the saved Riot API key should be treated as expired/invalid.
pub fn is_riot_key_expired(
    last_error: &Option<String>,
    last_verified_at: &Option<String>,
) -> bool {
    let Some(err) = last_error else {
        return false;
    };
    if !is_riot_key_expired_error(err) {
        return false;
    }

    let Some(verified) = last_verified_at else {
        return true;
    };

    let Ok(verified_at) = chrono::DateTime::parse_from_rfc3339(verified) else {
        return true;
    };

    // A recent successful call means the key still works even if an older error
    // was accidentally re-saved from the settings UI.
    chrono::Utc::now().signed_duration_since(verified_at).num_hours() >= 24
}

pub fn mask_riot_api_key(key: &str) -> String {
    let cleaned = sanitize_riot_api_key(key);
    if cleaned.len() <= 8 {
        return "RGAPI-••••".to_string();
    }
    format!("RGAPI-...{}", &cleaned[cleaned.len().saturating_sub(4)..])
}
