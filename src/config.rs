use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub background_color: String,
    pub input_bg_color: String,
    pub text_color: String,
    pub text_dim_color: String,
    pub icon_color: String,
    pub accent_color: String,
    pub accent_rest_color: String,
    pub font_family: String,
    #[serde(default)]
    pub transparent: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background_color: "#000000".to_string(),
            input_bg_color: "#111111".to_string(),
            text_color: "#cccccc".to_string(),
            text_dim_color: "#777777".to_string(),
            icon_color: "#888888".to_string(),
            accent_color: "#4CAF50".to_string(),
            accent_rest_color: "#2196F3".to_string(),
            font_family: "TX02 Nerd Font".to_string(),
            transparent: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub timer_duration_minutes: u32,
    pub rest_duration_minutes: u32,
    pub alarm_hour: u32,
    pub alarm_min: u32,
    #[serde(default)]
    pub theme: Theme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timer_duration_minutes: 25,
            rest_duration_minutes: 5,
            alarm_hour: 10,
            alarm_min: 20,
            theme: Theme::default(),
        }
    }
}

impl Config {
    /// Get the config file path (~/.config/rsmodoro/config.json)
    fn get_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("rsmodoro").join("config.json"))
    }

    /// Load config from file, or return default if not found
    pub fn load() -> Self {
        let Some(path) = Self::get_config_path() else {
            return Self::default();
        };

        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<(), std::io::Error> {
        let Some(path) = Self::get_config_path() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine config directory",
            ));
        };

        // Create rsmodoro directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        Ok(())
    }
}
