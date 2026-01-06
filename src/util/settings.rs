use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use dirs::config_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub auto_start: bool,
    pub show_console: bool,
    pub custom_status_template: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_start: false,
            show_console: false,
            custom_status_template: None,
        }
    }
}

impl Settings {
    pub fn config_path() -> PathBuf {
        let mut path = config_dir().expect("Unable to find config directory");
        path.push("roblox_discord_presence");
        path.push("settings.toml");
        path
    }

    pub fn load() -> Self {
        let config_path = Self::config_path();
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => toml::from_str(&content).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            let defaults = Self::default();
            // Try explicit save so the user sees the file
            if let Some(parent) = config_path.parent() {
                 let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(config_path, toml::to_string_pretty(&defaults).unwrap_or_default());
            defaults
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }
}
