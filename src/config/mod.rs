use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::common::{PriorityClass, ConfigResult, EndlessOptError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub auto_optimize: bool,
    pub auto_interval: u64,
    pub auto_game_mode: bool,
    pub game_priority: PriorityClass,
    pub bg_priority: PriorityClass,
    pub mem_clean: bool,
    pub net_optimize: bool,
    pub game_processes: Vec<String>,
    pub blacklisted_processes: Vec<String>,
    pub theme: Theme,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            auto_optimize: false,
            auto_interval: 30,
            auto_game_mode: false,
            game_priority: PriorityClass::High,
            bg_priority: PriorityClass::BelowNormal,
            mem_clean: true,
            net_optimize: true,
            game_processes: vec![
                "minecraft.exe".to_string(),
                "minecraftlauncher.exe".to_string(),
                "javaw.exe".to_string(),
                "steam.exe".to_string(),
            ],
            blacklisted_processes: vec![
                "system".to_string(),
                "system idle process".to_string(),
                "registry".to_string(),
                "smss.exe".to_string(),
                "csrss.exe".to_string(),
                "wininit.exe".to_string(),
                "services.exe".to_string(),
                "lsass.exe".to_string(),
                "svchost.exe".to_string(),
                "explorer.exe".to_string(),
            ],
            theme: Theme::Dark,
        }
    }
}

impl Config {
    pub fn load() -> ConfigResult<Self> {
        let config_path = Self::get_config_path();

        if config_path.exists() {
            let content = fs::read_to_string(&config_path).map_err(|e| {
                Box::new(EndlessOptError::FileSystem {
                    path: config_path.display().to_string(),
                    operation: "read".to_string(),
                    details: e.to_string(),
                }) as Box<dyn std::error::Error>
            })?;

            serde_json::from_str(&content).map_err(|e| {
                Box::new(EndlessOptError::Config(format!("Failed to parse config: {}", e))) as Box<dyn std::error::Error>
            })
        } else {
            let default = Config::default();
            default.save()?;
            Ok(default)
        }
    }

    pub fn save(&self) -> ConfigResult<()> {
        let config_path = Self::get_config_path();

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                Box::new(EndlessOptError::FileSystem {
                    path: parent.display().to_string(),
                    operation: "create_dir".to_string(),
                    details: e.to_string(),
                }) as Box<dyn std::error::Error>
            })?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            Box::new(EndlessOptError::Config(format!("Failed to serialize config: {}", e))) as Box<dyn std::error::Error>
        })?;

        fs::write(&config_path, content).map_err(|e| {
            Box::new(EndlessOptError::FileSystem {
                path: config_path.display().to_string(),
                operation: "write".to_string(),
                details: e.to_string(),
            }) as Box<dyn std::error::Error>
        })?;

        Ok(())
    }

    fn get_config_path() -> PathBuf {
        // Try to get user's home directory, fallback to current directory
        if let Ok(home) = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
        {
            let mut path = PathBuf::from(home);
            path.push(".endlessopt");
            path.push("config.json");
            path
        } else {
            PathBuf::from("config.json")
        }
    }

    #[allow(dead_code)]
    pub fn is_process_blacklisted(&self, name: &str) -> bool {
        self.blacklisted_processes.iter()
            .any(|p| p.eq_ignore_ascii_case(name))
    }

    #[allow(dead_code)]
    pub fn is_game_process(&self, name: &str) -> bool {
        self.game_processes.iter()
            .any(|p| p.eq_ignore_ascii_case(name))
    }
}
