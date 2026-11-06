use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PomodoroPhase {
    Idle,
    Focus,
    Break,
    PausedFocus(u32),
    PausedBreak(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntry {
    pub date: String,
    pub minutes: u32,
}

pub struct PomodoroEngine {
    pub phase: PomodoroPhase,
    pub focus_mins: u32,
    pub break_mins: u32,
    started_at: Option<Instant>,
    target_secs: u32,
}

impl PomodoroEngine {
    pub fn new(focus_mins: u32, break_mins: u32) -> Self {
        Self {
            phase: PomodoroPhase::Idle,
            focus_mins,
            break_mins,
            started_at: None,
            target_secs: focus_mins * 60,
        }
    }

    pub fn start_focus(&mut self) {
        self.target_secs = self.focus_mins * 60;
        self.started_at = Some(Instant::now());
        self.phase = PomodoroPhase::Focus;
    }

    pub fn set_preset(&mut self, focus_mins: u32, break_mins: u32) {
        self.focus_mins = focus_mins;
        self.break_mins = break_mins;
        if self.phase == PomodoroPhase::Idle {
            self.target_secs = focus_mins * 60;
        }
    }

    pub fn toggle_pause(&mut self) {
        let rem = self.remaining_secs();
        match self.phase {
            PomodoroPhase::Focus => {
                self.phase = PomodoroPhase::PausedFocus(rem);
                self.started_at = None;
            }
            PomodoroPhase::Break => {
                self.phase = PomodoroPhase::PausedBreak(rem);
                self.started_at = None;
            }
            PomodoroPhase::PausedFocus(r) => {
                self.target_secs = r;
                self.started_at = Some(Instant::now());
                self.phase = PomodoroPhase::Focus;
            }
            PomodoroPhase::PausedBreak(r) => {
                self.target_secs = r;
                self.started_at = Some(Instant::now());
                self.phase = PomodoroPhase::Break;
            }
            PomodoroPhase::Idle => {
                self.start_focus();
            }
        }
    }

    pub fn reset(&mut self) {
        self.phase = PomodoroPhase::Idle;
        self.started_at = None;
        self.target_secs = self.focus_mins * 60;
    }

    pub fn remaining_secs(&mut self) -> u32 {
        match self.phase {
            PomodoroPhase::Idle => self.focus_mins * 60,
            PomodoroPhase::PausedFocus(r) | PomodoroPhase::PausedBreak(r) => r,
            PomodoroPhase::Focus | PomodoroPhase::Break => {
                if let Some(start) = self.started_at {
                    let elapsed = start.elapsed().as_secs() as u32;
                    if elapsed >= self.target_secs {
                        if self.phase == PomodoroPhase::Focus {
                            let _ = log_session(self.focus_mins);
                            self.phase = PomodoroPhase::Break;
                            self.target_secs = self.break_mins * 60;
                            self.started_at = Some(Instant::now());
                            return self.target_secs;
                        } else {
                            self.reset();
                            return self.target_secs;
                        }
                    }
                    self.target_secs.saturating_sub(elapsed)
                } else {
                    self.target_secs
                }
            }
        }
    }

    pub fn format_mm_ss(&mut self) -> String {
        let secs = self.remaining_secs();
        format!("{:02}:{:02}", secs / 60, secs % 60)
    }

    pub fn to_map(&mut self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        let timer = self.format_mm_ss();
        let state_str = match self.phase {
            PomodoroPhase::Idle => "Siap Fokus",
            PomodoroPhase::Focus => "Fokus Berjalan",
            PomodoroPhase::Break => "Istirahat",
            PomodoroPhase::PausedFocus(_) | PomodoroPhase::PausedBreak(_) => "Dijeda",
        };
        m.insert("timer".to_string(), timer);
        m.insert("state".to_string(), state_str.to_string());
        m.insert(
            "preset".to_string(),
            format!(
                "Preset aktif: {} mnt fokus · {} mnt istirahat",
                self.focus_mins, self.break_mins
            ),
        );
        m.insert("stats".to_string(), compute_stats_summary());
        m
    }
}

fn log_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("meera-notch")
        .join("pomodoro.jsonl")
}

fn log_session(minutes: u32) -> std::io::Result<()> {
    let path = log_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let entry = SessionEntry {
        date: Local::now().format("%Y-%m-%d").to_string(),
        minutes,
    };
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    if let Ok(line) = serde_json::to_string(&entry) {
        writeln!(file, "{}", line)?;
    }
    Ok(())
}

pub fn compute_stats_summary() -> String {
    let path = log_path();
    let today = Local::now().format("%Y-%m-%d").to_string();
    let mut today_count = 0u32;
    let mut today_mins = 0u32;
    let mut total_count = 0u32;

    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            if let Ok(entry) = serde_json::from_str::<SessionEntry>(line) {
                total_count += 1;
                if entry.date == today {
                    today_count += 1;
                    today_mins += entry.minutes;
                }
            }
        }
    }
    format!(
        "Hari ini: {} sesi ({} mnt) · Total tercatat: {} sesi",
        today_count, today_mins, total_count
    )
}
