use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub open_mode: String,
    pub hide_on_fullscreen: bool,
    pub hover_delay_ms: u32,
    pub collapse_delay_ms: u32,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone_offset_hours: f64,
    pub ihtiyath_minutes: i32,
    pub qari: String,
    pub current_surah: u16,
    pub repeat_mode: String,
    pub pomodoro_focus_mins: u32,
    pub pomodoro_break_mins: u32,
    pub cache_limit_mb: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            open_mode: "click".to_string(),
            hide_on_fullscreen: true,
            hover_delay_ms: 180,
            collapse_delay_ms: 300,
            city: "Bandung".to_string(),
            latitude: -6.9203,
            longitude: 107.6233,
            timezone_offset_hours: 7.0,
            ihtiyath_minutes: 2,
            qari: "alafasy".to_string(),
            current_surah: 67,
            repeat_mode: "surah".to_string(),
            pomodoro_focus_mins: 25,
            pomodoro_break_mins: 5,
            cache_limit_mb: 1024,
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        let base = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(home).join(".config")
            });
        base.join("meera-notch").join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = fs::read_to_string(&path) {
            match toml::from_str::<Config>(&content) {
                Ok(cfg) => return cfg,
                Err(e) => {
                    tracing::warn!("Config file invalid, using defaults: {}", e);
                }
            }
        }
        let cfg = Self::default();
        let _ = cfg.save();
        cfg
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let tmp_path = path.with_extension("toml.tmp");
        let body = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&tmp_path, body).map_err(|e| e.to_string())?;
        fs::rename(&tmp_path, &path).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_roundtrip() {
        let cfg = Config::default();
        let encoded = toml::to_string_pretty(&cfg).unwrap();
        let decoded: Config = toml::from_str(&encoded).unwrap();
        assert_eq!(decoded.city, "Bandung");
        assert_eq!(decoded.current_surah, 67);
        assert_eq!(decoded.ihtiyath_minutes, 2);
    }
}
