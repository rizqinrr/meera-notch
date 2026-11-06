pub mod config;
pub mod notes;
pub mod pomodoro;
pub mod power;
pub mod prayer;
pub mod quran;
pub mod system;
pub mod weather;

use std::collections::HashMap;
use std::sync::Mutex;
use zbus::interface;

pub struct AppState {
    pub cfg: config::Config,
    pub sys: system::SystemMonitor,
    pub weather: weather::WeatherCache,
    pub pomodoro: pomodoro::PomodoroEngine,
    pub quran: quran::QuranPlayer,
    pub expanded: bool,
}

pub struct MeeraNotchService {
    pub state: Mutex<AppState>,
}

#[interface(name = "io.github.rizqinrr.MeeraNotch1")]
impl MeeraNotchService {
    #[zbus(property)]
    fn version(&self) -> &str {
        "0.1.0"
    }

    #[zbus(property)]
    fn pill(&self) -> HashMap<String, String> {
        let mut st = self.state.lock().unwrap();
        let sched = prayer::compute_schedule(
            st.cfg.latitude,
            st.cfg.longitude,
            st.cfg.timezone_offset_hours,
            st.cfg.ihtiyath_minutes,
        );
        let (bat_pct, _, _, _) = system::read_battery();
        let pomo_str = st.pomodoro.format_mm_ss();
        let qmap = st.quran.to_map();

        let mut map = HashMap::new();
        map.insert("prayer_name".to_string(), sched.next_name);
        map.insert("prayer_time".to_string(), sched.next_time);
        map.insert("countdown".to_string(), sched.countdown_short);
        map.insert("pomodoro".to_string(), pomo_str);
        map.insert("battery".to_string(), format!("{}%", bat_pct));
        map.insert(
            "surah_title".to_string(),
            qmap.get("title").cloned().unwrap_or_default(),
        );
        map.insert(
            "qari_name".to_string(),
            qmap.get("qari_name").cloned().unwrap_or_default(),
        );
        map.insert(
            "quran_elapsed".to_string(),
            qmap.get("elapsed").cloned().unwrap_or_default(),
        );
        map.insert(
            "is_playing".to_string(),
            qmap.get("is_playing").cloned().unwrap_or_else(|| "false".to_string()),
        );
        map.insert(
            "is_quran_active".to_string(),
            qmap.get("is_active").cloned().unwrap_or_else(|| "false".to_string()),
        );
        map.insert("open_mode".to_string(), st.cfg.open_mode.clone());
        map
    }

    #[zbus(property)]
    fn prayer(&self) -> HashMap<String, String> {
        let st = self.state.lock().unwrap();
        let sched = prayer::compute_schedule(
            st.cfg.latitude,
            st.cfg.longitude,
            st.cfg.timezone_offset_hours,
            st.cfg.ihtiyath_minutes,
        );
        sched.to_map(&st.cfg.city, st.cfg.latitude, st.cfg.longitude)
    }

    #[zbus(property)]
    fn quran(&self) -> HashMap<String, String> {
        let mut st = self.state.lock().unwrap();
        st.quran.to_map()
    }

    #[zbus(property)]
    fn system(&self) -> HashMap<String, String> {
        let mut st = self.state.lock().unwrap();
        st.sys.refresh().clone()
    }

    #[zbus(property)]
    fn weather(&self) -> HashMap<String, String> {
        let mut st = self.state.lock().unwrap();
        let city = st.cfg.city.clone();
        let lat = st.cfg.latitude;
        let lon = st.cfg.longitude;
        st.weather.get_or_refresh(&city, lat, lon)
    }

    #[zbus(property)]
    fn pomodoro(&self) -> HashMap<String, String> {
        let mut st = self.state.lock().unwrap();
        st.pomodoro.to_map()
    }

    fn ping(&self) -> String {
        "pong".to_string()
    }

    fn set_expanded(&self, expanded: bool) {
        let mut st = self.state.lock().unwrap();
        st.expanded = expanded;
        if expanded {
            st.sys.refresh();
        }
    }

    fn quran_play(&self, qari: &str, surah: u16) -> HashMap<String, String> {
        let mut st = self.state.lock().unwrap();
        st.quran.play(surah, qari);
        st.cfg.current_surah = st.quran.current_surah;
        if !qari.is_empty() {
            st.cfg.qari = qari.to_string();
        }
        let _ = st.cfg.save();
        st.quran.to_map()
    }

    fn quran_toggle(&self) -> HashMap<String, String> {
        let mut st = self.state.lock().unwrap();
        st.quran.toggle();
        st.quran.to_map()
    }

    fn quran_stop(&self) -> HashMap<String, String> {
        let mut st = self.state.lock().unwrap();
        st.quran.stop();
        st.quran.to_map()
    }

    fn quran_next(&self) -> HashMap<String, String> {
        let mut st = self.state.lock().unwrap();
        st.quran.next_surah();
        st.cfg.current_surah = st.quran.current_surah;
        let _ = st.cfg.save();
        st.quran.to_map()
    }

    fn quran_prev(&self) -> HashMap<String, String> {
        let mut st = self.state.lock().unwrap();
        st.quran.prev_surah();
        st.cfg.current_surah = st.quran.current_surah;
        let _ = st.cfg.save();
        st.quran.to_map()
    }

    fn pomodoro_start(&self, preset: &str) -> HashMap<String, String> {
        let mut st = self.state.lock().unwrap();
        if preset == "50/10" {
            st.pomodoro.set_preset(50, 10);
            st.cfg.pomodoro_focus_mins = 50;
            st.cfg.pomodoro_break_mins = 10;
        } else if preset == "25/5" {
            st.pomodoro.set_preset(25, 5);
            st.cfg.pomodoro_focus_mins = 25;
            st.cfg.pomodoro_break_mins = 5;
        }
        st.pomodoro.start_focus();
        let _ = st.cfg.save();
        st.pomodoro.to_map()
    }

    fn pomodoro_toggle(&self) -> HashMap<String, String> {
        let mut st = self.state.lock().unwrap();
        st.pomodoro.toggle_pause();
        st.pomodoro.to_map()
    }

    fn pomodoro_reset(&self) -> HashMap<String, String> {
        let mut st = self.state.lock().unwrap();
        st.pomodoro.reset();
        st.pomodoro.to_map()
    }

    fn notes_get(&self) -> String {
        notes::read_notes()
    }

    fn notes_set(&self, content: &str) -> bool {
        notes::write_notes(content).is_ok()
    }

    fn power(&self, action: &str) -> String {
        power::execute_power_action(action)
    }

    fn get_config(&self) -> String {
        let st = self.state.lock().unwrap();
        serde_json::to_string(&st.cfg).unwrap_or_default()
    }

    fn set_open_mode(&self, mode: &str) -> bool {
        let mut st = self.state.lock().unwrap();
        if matches!(mode, "hover" | "click" | "always") {
            st.cfg.open_mode = mode.to_string();
            return st.cfg.save().is_ok();
        }
        false
    }

    fn refresh_location(&self) -> HashMap<String, String> {
        let mut st = self.state.lock().unwrap();
        if let Some((city, lat, lon)) = weather::detect_geoip() {
            st.cfg.city = city;
            st.cfg.latitude = lat;
            st.cfg.longitude = lon;
            let _ = st.cfg.save();
        }
        let sched = prayer::compute_schedule(
            st.cfg.latitude,
            st.cfg.longitude,
            st.cfg.timezone_offset_hours,
            st.cfg.ihtiyath_minutes,
        );
        sched.to_map(&st.cfg.city, st.cfg.latitude, st.cfg.longitude)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting meera-notchd v0.1.0...");

    let cfg = config::Config::load();
    let pomo = pomodoro::PomodoroEngine::new(cfg.pomodoro_focus_mins, cfg.pomodoro_break_mins);
    let quran = quran::QuranPlayer::new(cfg.current_surah, cfg.qari.clone());

    let service = MeeraNotchService {
        state: Mutex::new(AppState {
            cfg,
            sys: system::SystemMonitor::new(),
            weather: weather::WeatherCache::new(),
            pomodoro: pomo,
            quran,
            expanded: false,
        }),
    };

    let _conn = zbus::connection::Builder::session()?
        .name("io.github.rizqinrr.MeeraNotch")?
        .serve_at("/io/github/rizqinrr/MeeraNotch", service)?
        .build()
        .await?;

    tracing::info!(
        "D-Bus service io.github.rizqinrr.MeeraNotch registered at /io/github/rizqinrr/MeeraNotch"
    );

    tokio::signal::ctrl_c().await?;
    tracing::info!("meera-notchd shutting down gracefully...");

    Ok(())
}
