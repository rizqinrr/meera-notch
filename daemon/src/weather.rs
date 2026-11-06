use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub summary: String,
    pub details: String,
    pub tomorrow: String,
    pub updated_label: String,
}

impl Default for WeatherData {
    fn default() -> Self {
        Self {
            summary: "Bandung · 26°C Berawan".to_string(),
            details: "Kelembapan 78% · Angin 9 km/j".to_string(),
            tomorrow: "Besok: 21 s/d 28°C".to_string(),
            updated_label: "Cache lokal".to_string(),
        }
    }
}

pub struct WeatherCache {
    pub data: WeatherData,
    last_fetch: Option<Instant>,
}

impl Default for WeatherCache {
    fn default() -> Self {
        Self::new()
    }
}

impl WeatherCache {
    pub fn new() -> Self {
        let mut wc = Self {
            data: WeatherData::default(),
            last_fetch: None,
        };
        if let Ok(raw) = fs::read_to_string(Self::cache_path()) {
            if let Ok(parsed) = serde_json::from_str::<WeatherData>(&raw) {
                wc.data = parsed;
            }
        }
        wc
    }

    fn cache_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home)
            .join(".cache")
            .join("meera-notch")
            .join("weather.json")
    }

    pub fn get_or_refresh(&mut self, city: &str, lat: f64, lon: f64) -> HashMap<String, String> {
        let should_fetch = match self.last_fetch {
            None => true,
            Some(t) => t.elapsed() > Duration::from_secs(1800),
        };
        if should_fetch {
            self.last_fetch = Some(Instant::now());
            if let Some(fresh) = fetch_wttr(city, lat, lon) {
                self.data = fresh;
                if let Some(parent) = Self::cache_path().parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Ok(json) = serde_json::to_string_pretty(&self.data) {
                    let _ = fs::write(Self::cache_path(), json);
                }
            }
        }
        let mut m = HashMap::new();
        m.insert("summary".to_string(), self.data.summary.clone());
        m.insert("details".to_string(), self.data.details.clone());
        m.insert("tomorrow".to_string(), self.data.tomorrow.clone());
        m.insert("updated".to_string(), self.data.updated_label.clone());
        m
    }
}

fn fetch_wttr(city: &str, lat: f64, lon: f64) -> Option<WeatherData> {
    let url = format!("https://wttr.in/{:.2},{:.2}?format=j1", lat, lon);
    let out = Command::new("curl")
        .args(["-s", "--max-time", "4", &url])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let val: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    let cur = val.get("current_condition")?.as_array()?.first()?;
    let temp_c = cur.get("temp_C")?.as_str()?;
    let humidity = cur.get("humidity")?.as_str()?;
    let wind = cur.get("windspeedKmph")?.as_str()?;
    let desc = cur
        .get("weatherDesc")
        .and_then(|a| a.as_array())
        .and_then(|a| a.first())
        .and_then(|o| o.get("value"))
        .and_then(|v| v.as_str())
        .unwrap_or("Berawan");

    let mut tomorrow_str = "Besok: 21 s/d 28°C".to_string();
    if let Some(w_arr) = val.get("weather").and_then(|v| v.as_array()) {
        if let Some(tom) = w_arr.get(1) {
            let min_c = tom.get("mintempC").and_then(|v| v.as_str()).unwrap_or("21");
            let max_c = tom.get("maxtempC").and_then(|v| v.as_str()).unwrap_or("28");
            tomorrow_str = format!("Besok: {} s/d {}°C", min_c, max_c);
        }
    }

    Some(WeatherData {
        summary: format!("{} · {}°C {}", city, temp_c, desc),
        details: format!("Kelembapan {}% · Angin {} km/j", humidity, wind),
        tomorrow: tomorrow_str,
        updated_label: "Baru saja diperbarui".to_string(),
    })
}

pub fn detect_geoip() -> Option<(String, f64, f64)> {
    let out = Command::new("curl")
        .args(["-s", "--max-time", "4", "https://ipapi.co/json/"])
        .output()
        .ok()?;
    if out.status.success() {
        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
            let city = val.get("city").and_then(|v| v.as_str())?.to_string();
            let lat = val.get("latitude").and_then(|v| v.as_f64())?;
            let lon = val.get("longitude").and_then(|v| v.as_f64())?;
            return Some((city, lat, lon));
        }
    }
    None
}
