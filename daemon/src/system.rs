use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct CpuSample {
    pub idle: u64,
    pub total: u64,
}

pub struct SystemMonitor {
    prev_cpu: Vec<CpuSample>,
    prev_net: Option<(u64, u64, Instant)>,
    pub last_snapshot: HashMap<String, String>,
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut mon = Self {
            prev_cpu: Vec::new(),
            prev_net: None,
            last_snapshot: HashMap::new(),
        };
        mon.refresh();
        mon
    }

    pub fn refresh(&mut self) -> &HashMap<String, String> {
        let mut map = HashMap::new();

        // 1. CPU & per-thread from /proc/stat
        let current_samples = read_proc_stat();
        let mut total_pct = 0u32;
        let mut thread_pcts = Vec::new();
        if !self.prev_cpu.is_empty() && self.prev_cpu.len() == current_samples.len() {
            for (i, (prev, cur)) in self.prev_cpu.iter().zip(current_samples.iter()).enumerate() {
                let d_total = cur.total.saturating_sub(prev.total);
                let d_idle = cur.idle.saturating_sub(prev.idle);
                let pct = (d_total.saturating_sub(d_idle) * 100)
                    .checked_div(d_total)
                    .unwrap_or(0) as u32;
                if i == 0 {
                    total_pct = pct;
                } else {
                    thread_pcts.push(format!("{}%", pct));
                }
            }
        }
        self.prev_cpu = current_samples;

        let load_avg = fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|s| s.split_whitespace().next().map(|v| v.to_string()))
            .unwrap_or_else(|| "0.00".to_string());

        let temp_c = read_cpu_temp();
        map.insert("cpu_pct".to_string(), format!("{}%", total_pct));
        map.insert("cpu_temp".to_string(), format!("{}°C", temp_c));
        map.insert(
            "cpu_threads".to_string(),
            if thread_pcts.is_empty() {
                format!("Load: {}", load_avg)
            } else {
                format!("Per-thread: {} · Load: {}", thread_pcts.join("  "), load_avg)
            },
        );

        // 2. RAM from /proc/meminfo
        let (used_gib, total_gib, avail_gib) = read_meminfo();
        let ram_pct = if total_gib > 0.0 {
            ((used_gib / total_gib) * 100.0).round() as u32
        } else {
            0
        };
        map.insert(
            "ram_summary".to_string(),
            format!("{:.1} / {:.1} GiB (tersedia {:.1} GiB)", used_gib, total_gib, avail_gib),
        );
        map.insert("ram_pct".to_string(), ram_pct.to_string());

        // 3. Battery BAT0 + ncharge threshold
        let (bat_pct, bat_status, bat_watts, bat_limit) = read_battery();
        map.insert("bat_pct".to_string(), format!("{}%", bat_pct));
        map.insert(
            "bat_summary".to_string(),
            if bat_watts > 0.1 {
                format!("{}% · {} ({:.1} W)", bat_pct, bat_status, bat_watts)
            } else {
                format!("{}% · {}", bat_pct, bat_status)
            },
        );
        map.insert(
            "bat_limit".to_string(),
            format!("Batas pengisian aktif: {}%", bat_limit),
        );

        // 4. Network RX/TX from /proc/net/dev
        let (iface, rx_bytes, tx_bytes) = read_net_bytes();
        let now = Instant::now();
        let (rx_rate, tx_rate) = if let Some((p_rx, p_tx, p_time)) = self.prev_net {
            let dt = now.duration_since(p_time).as_secs_f64().max(0.1);
            let r_s = (rx_bytes.saturating_sub(p_rx) as f64 / dt) as u64;
            let t_s = (tx_bytes.saturating_sub(p_tx) as f64 / dt) as u64;
            (format_rate(r_s), format_rate(t_s))
        } else {
            ("0 KB/s".to_string(), "0 KB/s".to_string())
        };
        self.prev_net = Some((rx_bytes, tx_bytes, now));
        map.insert("net_iface".to_string(), iface);
        map.insert("net_rx".to_string(), format!("Turun {}", rx_rate));
        map.insert("net_tx".to_string(), format!("Naik {}", tx_rate));

        self.last_snapshot = map;
        &self.last_snapshot
    }
}

fn read_proc_stat() -> Vec<CpuSample> {
    let mut out = Vec::new();
    if let Ok(content) = fs::read_to_string("/proc/stat") {
        for line in content.lines() {
            if line.starts_with("cpu") {
                let parts: Vec<u64> = line
                    .split_whitespace()
                    .skip(1)
                    .filter_map(|s| s.parse::<u64>().ok())
                    .collect();
                if parts.len() >= 4 {
                    let idle = parts[3] + parts.get(4).copied().unwrap_or(0);
                    let total: u64 = parts.iter().sum();
                    out.push(CpuSample { idle, total });
                }
            }
        }
    }
    out
}

fn read_cpu_temp() -> u32 {
    if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = fs::read_to_string(path.join("name")).unwrap_or_default();
            if name.trim() == "coretemp" {
                if let Ok(t) = fs::read_to_string(path.join("temp1_input")) {
                    if let Ok(milli) = t.trim().parse::<u32>() {
                        return milli / 1000;
                    }
                }
            }
        }
    }
    if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
        for entry in entries.flatten() {
            let path = entry.path();
            let ttype = fs::read_to_string(path.join("type")).unwrap_or_default();
            if ttype.trim() == "x86_pkg_temp" {
                if let Ok(t) = fs::read_to_string(path.join("temp")) {
                    if let Ok(milli) = t.trim().parse::<u32>() {
                        return milli / 1000;
                    }
                }
            }
        }
    }
    45
}

fn read_meminfo() -> (f64, f64, f64) {
    let mut total_kb = 0u64;
    let mut avail_kb = 0u64;
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("MemTotal:") {
                total_kb = rest
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
                avail_kb = rest
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }
    }
    let total_gib = total_kb as f64 / 1048576.0;
    let avail_gib = avail_kb as f64 / 1048576.0;
    let used_gib = (total_gib - avail_gib).max(0.0);
    (used_gib, total_gib, avail_gib)
}

pub fn read_battery() -> (u32, String, f64, u32) {
    let bat = Path::new("/sys/class/power_supply/BAT0");
    let capacity = fs::read_to_string(bat.join("capacity"))
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(80);
    let raw_status = fs::read_to_string(bat.join("status"))
        .unwrap_or_else(|_| "Unknown".to_string())
        .trim()
        .to_string();
    let status = match raw_status.as_str() {
        "Charging" => "Mengisi".to_string(),
        "Discharging" => "Memakai baterai".to_string(),
        "Not charging" | "Full" => "Penuh / Terjaga".to_string(),
        other => other.to_string(),
    };
    let power_uw = fs::read_to_string(bat.join("power_now"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let watts = power_uw as f64 / 1_000_000.0;
    let limit = fs::read_to_string(bat.join("charge_control_end_threshold"))
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(100);
    (capacity, status, watts, limit)
}

fn read_net_bytes() -> (String, u64, u64) {
    let mut best_iface = "wlan0".to_string();
    let mut best_rx = 0u64;
    let mut best_tx = 0u64;
    if let Ok(content) = fs::read_to_string("/proc/net/dev") {
        for line in content.lines().skip(2) {
            if let Some((iface, rest)) = line.split_once(':') {
                let name = iface.trim();
                if name == "lo" {
                    continue;
                }
                let cols: Vec<u64> = rest
                    .split_whitespace()
                    .filter_map(|s| s.parse::<u64>().ok())
                    .collect();
                if cols.len() >= 9 && cols[0] >= best_rx {
                    best_iface = name.to_string();
                    best_rx = cols[0];
                    best_tx = cols[8];
                }
            }
        }
    }
    (best_iface, best_rx, best_tx)
}

fn format_rate(bytes_per_sec: u64) -> String {
    if bytes_per_sec >= 1_048_576 {
        format!("{:.1} MB/s", bytes_per_sec as f64 / 1_048_576.0)
    } else {
        format!("{} KB/s", bytes_per_sec / 1024)
    }
}
