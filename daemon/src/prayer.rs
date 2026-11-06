use chrono::{Datelike, Local, NaiveDate, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrayerSchedule {
    pub imsak: String,
    pub subuh: String,
    pub terbit: String,
    pub dzuhur: String,
    pub ashar: String,
    pub maghrib: String,
    pub isya: String,
    pub next_name: String,
    pub next_time: String,
    pub countdown_hms: String,
    pub countdown_short: String,
    pub hijri_date: String,
}

fn deg_to_rad(d: f64) -> f64 {
    d * PI / 180.0
}

fn rad_to_deg(r: f64) -> f64 {
    r * 180.0 / PI
}

fn normalize_angle(a: f64) -> f64 {
    let mut b = a % 360.0;
    if b < 0.0 {
        b += 360.0;
    }
    b
}

/// Returns (declination_deg, equation_of_time_hours) for a given Julian Date
fn sun_position(jd: f64) -> (f64, f64) {
    let d = jd - 2451545.0;
    let g = normalize_angle(357.529 + 0.98560028 * d);
    let q = normalize_angle(280.459 + 0.98564736 * d);
    let l = normalize_angle(q + 1.915 * deg_to_rad(g).sin() + 0.020 * deg_to_rad(2.0 * g).sin());
    let e = 23.439 - 0.00000036 * d;

    let ra = rad_to_deg(
        (deg_to_rad(e).cos() * deg_to_rad(l).sin()).atan2(deg_to_rad(l).cos()),
    ) / 15.0;
    let mut ra_norm = ra % 24.0;
    if ra_norm < 0.0 {
        ra_norm += 24.0;
    }

    let decl = rad_to_deg((deg_to_rad(e).sin() * deg_to_rad(l).sin()).asin());
    let eqt = q / 15.0 - ra_norm;
    (decl, eqt)
}

fn julian_date(year: i32, month: u32, day: u32) -> f64 {
    let mut y = year as f64;
    let mut m = month as f64;
    let d = day as f64;
    if m <= 2.0 {
        y -= 1.0;
        m += 12.0;
    }
    let a = (y / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + d + b - 1524.5
}

fn hour_angle(lat: f64, decl: f64, altitude_deg: f64) -> f64 {
    let lat_r = deg_to_rad(lat);
    let decl_r = deg_to_rad(decl);
    let alt_r = deg_to_rad(altitude_deg);
    let cos_ha = (alt_r.sin() - lat_r.sin() * decl_r.sin()) / (lat_r.cos() * decl_r.cos());
    let clamped = cos_ha.clamp(-1.0, 1.0);
    rad_to_deg(clamped.acos()) / 15.0
}

fn asr_hour_angle(lat: f64, decl: f64, shadow_factor: f64) -> f64 {
    let diff_r = deg_to_rad((lat - decl).abs());
    let alt_r = (1.0 / (shadow_factor + diff_r.tan())).atan();
    let alt_deg = rad_to_deg(alt_r);
    hour_angle(lat, decl, alt_deg)
}

fn hours_to_minutes(h: f64, ihtiyath_mins: i32) -> i32 {
    let raw_mins = (h * 60.0).ceil() as i32 + ihtiyath_mins;
    raw_mins.rem_euclid(24 * 60)
}

fn format_hm(total_mins: i32) -> String {
    let m = total_mins.rem_euclid(24 * 60);
    format!("{:02}:{:02}", m / 60, m % 60)
}

/// Tabular Islamic calendar calculation
pub fn to_hijri_string(date: NaiveDate) -> String {
    let jd = julian_date(date.year(), date.month(), date.day()) as i64;
    let l = jd - 1948440 + 10632;
    let n = (l - 1) / 10631;
    let l = l - 10631 * n + 354;
    let j = ((10985 - l) / 5316) * ((50 * l) / 17719) + (l / 5670) * ((43 * l) / 15238);
    let l = l - ((30 - j) / 15) * ((17719 * j) / 50) - (j / 16) * ((15238 * j) / 43) + 29;
    let month = (24 * l) / 709;
    let day = l - (709 * month) / 24;
    let year = 30 * n + j - 30;

    let month_names = [
        "Muharram",
        "Safar",
        "Rabiul Awal",
        "Rabiul Akhir",
        "Jumadil Awal",
        "Jumadil Akhir",
        "Rajab",
        "Syaban",
        "Ramadhan",
        "Syawal",
        "Dzulqaidah",
        "Dzulhijjah",
    ];
    let m_idx = (month.clamp(1, 12) - 1) as usize;
    format!("{} {} {} H (perkiraan)", day, month_names[m_idx], year)
}

pub fn compute_schedule(
    lat: f64,
    lon: f64,
    tz_offset: f64,
    ihtiyath_mins: i32,
) -> PrayerSchedule {
    let now = Local::now();
    let today = now.date_naive();
    let jd = julian_date(today.year(), today.month(), today.day()) - tz_offset / 24.0;
    let (decl, eqt) = sun_position(jd + 0.5);

    let transit = 12.0 + tz_offset - (lon / 15.0) - eqt;
    let ha_fajr = hour_angle(lat, decl, -20.0);
    let ha_sun = hour_angle(lat, decl, -0.8333);
    let ha_asr = asr_hour_angle(lat, decl, 1.0);
    let ha_isha = hour_angle(lat, decl, -18.0);

    let subuh_m = hours_to_minutes(transit - ha_fajr, ihtiyath_mins);
    let imsak_m = (subuh_m - 10).rem_euclid(24 * 60);
    let terbit_m = hours_to_minutes(transit - ha_sun, -ihtiyath_mins);
    let dzuhur_m = hours_to_minutes(transit, ihtiyath_mins + 1);
    let ashar_m = hours_to_minutes(transit + ha_asr, ihtiyath_mins);
    let maghrib_m = hours_to_minutes(transit + ha_sun, ihtiyath_mins);
    let isya_m = hours_to_minutes(transit + ha_isha, ihtiyath_mins);

    let now_secs = (now.hour() * 3600 + now.minute() * 60 + now.second()) as i32;
    let prayers = [
        ("Imsak", imsak_m),
        ("Subuh", subuh_m),
        ("Dzuhur", dzuhur_m),
        ("Ashar", ashar_m),
        ("Maghrib", maghrib_m),
        ("Isya", isya_m),
    ];

    let mut next_name = "Subuh".to_string();
    let mut next_mins = subuh_m;
    let mut diff_secs = (subuh_m * 60 + 24 * 3600) - now_secs;

    for (name, p_mins) in prayers.iter() {
        let p_secs = p_mins * 60;
        if p_secs > now_secs {
            next_name = (*name).to_string();
            next_mins = *p_mins;
            diff_secs = p_secs - now_secs;
            break;
        }
    }

    let rem_h = diff_secs / 3600;
    let rem_m = (diff_secs % 3600) / 60;
    let rem_s = diff_secs % 60;

    let countdown_hms = format!("-{:02}:{:02}:{:02}", rem_h, rem_m, rem_s);
    let countdown_short = if rem_h > 0 {
        format!("{}j {}m", rem_h, rem_m)
    } else {
        format!("{}m", rem_m.max(1))
    };

    PrayerSchedule {
        imsak: format_hm(imsak_m),
        subuh: format_hm(subuh_m),
        terbit: format_hm(terbit_m),
        dzuhur: format_hm(dzuhur_m),
        ashar: format_hm(ashar_m),
        maghrib: format_hm(maghrib_m),
        isya: format_hm(isya_m),
        next_name,
        next_time: format_hm(next_mins),
        countdown_hms,
        countdown_short,
        hijri_date: to_hijri_string(today),
    }
}

impl PrayerSchedule {
    pub fn to_map(&self, city: &str, lat: f64, lon: f64) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("city".to_string(), city.to_string());
        m.insert("coords".to_string(), format!("{:.2}, {:.2}", lat, lon));
        m.insert("imsak".to_string(), self.imsak.clone());
        m.insert("subuh".to_string(), self.subuh.clone());
        m.insert("terbit".to_string(), self.terbit.clone());
        m.insert("dzuhur".to_string(), self.dzuhur.clone());
        m.insert("ashar".to_string(), self.ashar.clone());
        m.insert("maghrib".to_string(), self.maghrib.clone());
        m.insert("isya".to_string(), self.isya.clone());
        m.insert("next_name".to_string(), self.next_name.clone());
        m.insert("next_time".to_string(), self.next_time.clone());
        m.insert("countdown_hms".to_string(), self.countdown_hms.clone());
        m.insert("countdown_short".to_string(), self.countdown_short.clone());
        m.insert("hijri_date".to_string(), self.hijri_date.clone());
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bandung_prayer_schedule() {
        let sched = compute_schedule(-6.9203, 107.6233, 7.0, 2);
        assert_eq!(sched.subuh.len(), 5);
        assert_eq!(sched.dzuhur.len(), 5);
        assert_eq!(sched.ashar.len(), 5);
        assert_eq!(sched.maghrib.len(), 5);
        assert_eq!(sched.isya.len(), 5);
        assert!(!sched.hijri_date.is_empty());
    }
}
