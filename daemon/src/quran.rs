use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Instant;

pub struct SurahMeta {
    pub number: u16,
    pub name: &'static str,
    pub meaning: &'static str,
    pub ayahs: u16,
    pub place: &'static str,
}

pub const SURAHS: [SurahMeta; 114] = [
    SurahMeta { number: 1, name: "Al-Fatihah", meaning: "Pembukaan", ayahs: 7, place: "Makkiyah" },
    SurahMeta { number: 2, name: "Al-Baqarah", meaning: "Sapi Betina", ayahs: 286, place: "Madaniyah" },
    SurahMeta { number: 3, name: "Ali 'Imran", meaning: "Keluarga Imran", ayahs: 200, place: "Madaniyah" },
    SurahMeta { number: 4, name: "An-Nisa'", meaning: "Wanita", ayahs: 176, place: "Madaniyah" },
    SurahMeta { number: 5, name: "Al-Ma'idah", meaning: "Hidangan", ayahs: 120, place: "Madaniyah" },
    SurahMeta { number: 6, name: "Al-An'am", meaning: "Binatang Ternak", ayahs: 165, place: "Makkiyah" },
    SurahMeta { number: 7, name: "Al-A'raf", meaning: "Tempat Tertinggi", ayahs: 206, place: "Makkiyah" },
    SurahMeta { number: 8, name: "Al-Anfal", meaning: "Harta Rampasan", ayahs: 75, place: "Madaniyah" },
    SurahMeta { number: 9, name: "At-Taubah", meaning: "Pengampunan", ayahs: 129, place: "Madaniyah" },
    SurahMeta { number: 10, name: "Yunus", meaning: "Nabi Yunus", ayahs: 109, place: "Makkiyah" },
    SurahMeta { number: 11, name: "Hud", meaning: "Nabi Hud", ayahs: 123, place: "Makkiyah" },
    SurahMeta { number: 12, name: "Yusuf", meaning: "Nabi Yusuf", ayahs: 111, place: "Makkiyah" },
    SurahMeta { number: 13, name: "Ar-Ra'd", meaning: "Guruh", ayahs: 43, place: "Madaniyah" },
    SurahMeta { number: 14, name: "Ibrahim", meaning: "Nabi Ibrahim", ayahs: 52, place: "Makkiyah" },
    SurahMeta { number: 15, name: "Al-Hijr", meaning: "Pegunungan Al-Hijr", ayahs: 99, place: "Makkiyah" },
    SurahMeta { number: 16, name: "An-Nahl", meaning: "Lebah", ayahs: 128, place: "Makkiyah" },
    SurahMeta { number: 17, name: "Al-Isra'", meaning: "Perjalanan Malam", ayahs: 111, place: "Makkiyah" },
    SurahMeta { number: 18, name: "Al-Kahf", meaning: "Penghuni Gua", ayahs: 110, place: "Makkiyah" },
    SurahMeta { number: 19, name: "Maryam", meaning: "Maryam", ayahs: 98, place: "Makkiyah" },
    SurahMeta { number: 20, name: "Taha", meaning: "Taha", ayahs: 135, place: "Makkiyah" },
    SurahMeta { number: 21, name: "Al-Anbiya'", meaning: "Para Nabi", ayahs: 112, place: "Makkiyah" },
    SurahMeta { number: 22, name: "Al-Hajj", meaning: "Ibadah Haji", ayahs: 78, place: "Madaniyah" },
    SurahMeta { number: 23, name: "Al-Mu'minun", meaning: "Orang-Orang Mukmin", ayahs: 118, place: "Makkiyah" },
    SurahMeta { number: 24, name: "An-Nur", meaning: "Cahaya", ayahs: 64, place: "Madaniyah" },
    SurahMeta { number: 25, name: "Al-Furqan", meaning: "Pembeda", ayahs: 77, place: "Makkiyah" },
    SurahMeta { number: 26, name: "Asy-Syu'ara'", meaning: "Para Penyair", ayahs: 227, place: "Makkiyah" },
    SurahMeta { number: 27, name: "An-Naml", meaning: "Semut", ayahs: 93, place: "Makkiyah" },
    SurahMeta { number: 28, name: "Al-Qasas", meaning: "Kisah-Kisah", ayahs: 88, place: "Makkiyah" },
    SurahMeta { number: 29, name: "Al-'Ankabut", meaning: "Laba-Laba", ayahs: 69, place: "Makkiyah" },
    SurahMeta { number: 30, name: "Ar-Rum", meaning: "Bangsa Romawi", ayahs: 60, place: "Makkiyah" },
    SurahMeta { number: 31, name: "Luqman", meaning: "Keluarga Luqman", ayahs: 34, place: "Makkiyah" },
    SurahMeta { number: 32, name: "As-Sajdah", meaning: "Sujud", ayahs: 30, place: "Makkiyah" },
    SurahMeta { number: 33, name: "Al-Ahzab", meaning: "Golongan Bersekutu", ayahs: 73, place: "Madaniyah" },
    SurahMeta { number: 34, name: "Saba'", meaning: "Kaum Saba'", ayahs: 54, place: "Makkiyah" },
    SurahMeta { number: 35, name: "Fatir", meaning: "Pencipta", ayahs: 45, place: "Makkiyah" },
    SurahMeta { number: 36, name: "Yasin", meaning: "Yasin", ayahs: 83, place: "Makkiyah" },
    SurahMeta { number: 37, name: "As-Saffat", meaning: "Barisan-Barisan", ayahs: 182, place: "Makkiyah" },
    SurahMeta { number: 38, name: "Sad", meaning: "Sad", ayahs: 88, place: "Makkiyah" },
    SurahMeta { number: 39, name: "Az-Zumar", meaning: "Rombongan", ayahs: 75, place: "Makkiyah" },
    SurahMeta { number: 40, name: "Ghafir", meaning: "Maha Pengampun", ayahs: 85, place: "Makkiyah" },
    SurahMeta { number: 41, name: "Fussilat", meaning: "Yang Dijelaskan", ayahs: 54, place: "Makkiyah" },
    SurahMeta { number: 42, name: "Asy-Syura", meaning: "Musyawarah", ayahs: 53, place: "Makkiyah" },
    SurahMeta { number: 43, name: "Az-Zukhruf", meaning: "Perhiasan", ayahs: 89, place: "Makkiyah" },
    SurahMeta { number: 44, name: "Ad-Dukhan", meaning: "Kabut", ayahs: 59, place: "Makkiyah" },
    SurahMeta { number: 45, name: "Al-Jasiyah", meaning: "Yang Berlutut", ayahs: 37, place: "Makkiyah" },
    SurahMeta { number: 46, name: "Al-Ahqaf", meaning: "Bukit Pasir", ayahs: 35, place: "Makkiyah" },
    SurahMeta { number: 47, name: "Muhammad", meaning: "Nabi Muhammad", ayahs: 38, place: "Madaniyah" },
    SurahMeta { number: 48, name: "Al-Fath", meaning: "Kemenangan", ayahs: 29, place: "Madaniyah" },
    SurahMeta { number: 49, name: "Al-Hujurat", meaning: "Kamar-Kamar", ayahs: 18, place: "Madaniyah" },
    SurahMeta { number: 50, name: "Qaf", meaning: "Qaf", ayahs: 45, place: "Makkiyah" },
    SurahMeta { number: 51, name: "Az-Zariyat", meaning: "Angin yang Menerbangkan", ayahs: 60, place: "Makkiyah" },
    SurahMeta { number: 52, name: "At-Tur", meaning: "Bukit Tursina", ayahs: 49, place: "Makkiyah" },
    SurahMeta { number: 53, name: "An-Najm", meaning: "Bintang", ayahs: 62, place: "Makkiyah" },
    SurahMeta { number: 54, name: "Al-Qamar", meaning: "Bulan", ayahs: 55, place: "Makkiyah" },
    SurahMeta { number: 55, name: "Ar-Rahman", meaning: "Maha Pengasih", ayahs: 78, place: "Madaniyah" },
    SurahMeta { number: 56, name: "Al-Waqi'ah", meaning: "Hari Kiamat", ayahs: 96, place: "Makkiyah" },
    SurahMeta { number: 57, name: "Al-Hadid", meaning: "Besi", ayahs: 29, place: "Madaniyah" },
    SurahMeta { number: 58, name: "Al-Mujadilah", meaning: "Gugatan", ayahs: 22, place: "Madaniyah" },
    SurahMeta { number: 59, name: "Al-Hasyr", meaning: "Pengusiran", ayahs: 24, place: "Madaniyah" },
    SurahMeta { number: 60, name: "Al-Mumtahanah", meaning: "Wanita yang Diuji", ayahs: 13, place: "Madaniyah" },
    SurahMeta { number: 61, name: "As-Saff", meaning: "Barisan", ayahs: 14, place: "Madaniyah" },
    SurahMeta { number: 62, name: "Al-Jumu'ah", meaning: "Hari Jumat", ayahs: 11, place: "Madaniyah" },
    SurahMeta { number: 63, name: "Al-Munafiqun", meaning: "Orang-Orang Munafik", ayahs: 11, place: "Madaniyah" },
    SurahMeta { number: 64, name: "At-Taghabun", meaning: "Hari Dinampakkan Kesalahan", ayahs: 18, place: "Madaniyah" },
    SurahMeta { number: 65, name: "At-Talaq", meaning: "Talak", ayahs: 12, place: "Madaniyah" },
    SurahMeta { number: 66, name: "At-Tahrim", meaning: "Mengharamkan", ayahs: 12, place: "Madaniyah" },
    SurahMeta { number: 67, name: "Al-Mulk", meaning: "Kerajaan", ayahs: 30, place: "Makkiyah" },
    SurahMeta { number: 68, name: "Al-Qalam", meaning: "Pena", ayahs: 52, place: "Makkiyah" },
    SurahMeta { number: 69, name: "Al-Haqqah", meaning: "Hari Kiamat yang Pasti", ayahs: 52, place: "Makkiyah" },
    SurahMeta { number: 70, name: "Al-Ma'arij", meaning: "Tempat Naik", ayahs: 44, place: "Makkiyah" },
    SurahMeta { number: 71, name: "Nuh", meaning: "Nabi Nuh", ayahs: 28, place: "Makkiyah" },
    SurahMeta { number: 72, name: "Al-Jinn", meaning: "Jin", ayahs: 28, place: "Makkiyah" },
    SurahMeta { number: 73, name: "Al-Muzzammil", meaning: "Orang yang Berselimut", ayahs: 20, place: "Makkiyah" },
    SurahMeta { number: 74, name: "Al-Muddassir", meaning: "Orang yang Berkemul", ayahs: 56, place: "Makkiyah" },
    SurahMeta { number: 75, name: "Al-Qiyamah", meaning: "Hari Kebangkitan", ayahs: 40, place: "Makkiyah" },
    SurahMeta { number: 76, name: "Al-Insan", meaning: "Manusia", ayahs: 31, place: "Madaniyah" },
    SurahMeta { number: 77, name: "Al-Mursalat", meaning: "Malaikat yang Diutus", ayahs: 50, place: "Makkiyah" },
    SurahMeta { number: 78, name: "An-Naba'", meaning: "Berita Besar", ayahs: 40, place: "Makkiyah" },
    SurahMeta { number: 79, name: "An-Nazi'at", meaning: "Malaikat yang Mencabut", ayahs: 46, place: "Makkiyah" },
    SurahMeta { number: 80, name: "'Abasa", meaning: "Ia Bermuka Masam", ayahs: 42, place: "Makkiyah" },
    SurahMeta { number: 81, name: "At-Takwir", meaning: "Menggulung", ayahs: 29, place: "Makkiyah" },
    SurahMeta { number: 82, name: "Al-Infitar", meaning: "Terbelah", ayahs: 19, place: "Makkiyah" },
    SurahMeta { number: 83, name: "Al-Mutaffifin", meaning: "Orang-Orang yang Curang", ayahs: 36, place: "Makkiyah" },
    SurahMeta { number: 84, name: "Al-Insyiqaq", meaning: "Terbelah", ayahs: 25, place: "Makkiyah" },
    SurahMeta { number: 85, name: "Al-Buruj", meaning: "Gugusan Bintang", ayahs: 22, place: "Makkiyah" },
    SurahMeta { number: 86, name: "At-Tariq", meaning: "Yang Datang di Malam Hari", ayahs: 17, place: "Makkiyah" },
    SurahMeta { number: 87, name: "Al-A'la", meaning: "Yang Paling Tinggi", ayahs: 19, place: "Makkiyah" },
    SurahMeta { number: 88, name: "Al-Ghasyiyah", meaning: "Hari Pembalasan", ayahs: 26, place: "Makkiyah" },
    SurahMeta { number: 89, name: "Al-Fajr", meaning: "Fajar", ayahs: 30, place: "Makkiyah" },
    SurahMeta { number: 90, name: "Al-Balad", meaning: "Negeri", ayahs: 20, place: "Makkiyah" },
    SurahMeta { number: 91, name: "Asy-Syams", meaning: "Matahari", ayahs: 15, place: "Makkiyah" },
    SurahMeta { number: 92, name: "Al-Lail", meaning: "Malam", ayahs: 21, place: "Makkiyah" },
    SurahMeta { number: 93, name: "Ad-Duha", meaning: "Waktu Dhuha", ayahs: 11, place: "Makkiyah" },
    SurahMeta { number: 94, name: "Asy-Syarh", meaning: "Melapangkan", ayahs: 8, place: "Makkiyah" },
    SurahMeta { number: 95, name: "At-Tin", meaning: "Buah Tin", ayahs: 8, place: "Makkiyah" },
    SurahMeta { number: 96, name: "Al-'Alaq", meaning: "Segumpal Darah", ayahs: 19, place: "Makkiyah" },
    SurahMeta { number: 97, name: "Al-Qadr", meaning: "Kemuliaan", ayahs: 5, place: "Makkiyah" },
    SurahMeta { number: 98, name: "Al-Bayyinah", meaning: "Bukti yang Nyata", ayahs: 8, place: "Madaniyah" },
    SurahMeta { number: 99, name: "Az-Zalzalah", meaning: "Kegoncangan", ayahs: 8, place: "Madaniyah" },
    SurahMeta { number: 100, name: "Al-'Adiyat", meaning: "Kuda Perang yang Berlari", ayahs: 11, place: "Makkiyah" },
    SurahMeta { number: 101, name: "Al-Qari'ah", meaning: "Hari Kiamat yang Menggetarkan", ayahs: 11, place: "Makkiyah" },
    SurahMeta { number: 102, name: "At-Takasur", meaning: "Bermegah-Megahan", ayahs: 8, place: "Makkiyah" },
    SurahMeta { number: 103, name: "Al-'Asr", meaning: "Masa", ayahs: 3, place: "Makkiyah" },
    SurahMeta { number: 104, name: "Al-Humazah", meaning: "Pengumpat", ayahs: 9, place: "Makkiyah" },
    SurahMeta { number: 105, name: "Al-Fil", meaning: "Gajah", ayahs: 5, place: "Makkiyah" },
    SurahMeta { number: 106, name: "Quraisy", meaning: "Suku Quraisy", ayahs: 4, place: "Makkiyah" },
    SurahMeta { number: 107, name: "Al-Ma'un", meaning: "Barang yang Berguna", ayahs: 7, place: "Makkiyah" },
    SurahMeta { number: 108, name: "Al-Kausar", meaning: "Nikmat yang Berlimpah", ayahs: 3, place: "Makkiyah" },
    SurahMeta { number: 109, name: "Al-Kafirun", meaning: "Orang-Orang Kafir", ayahs: 6, place: "Makkiyah" },
    SurahMeta { number: 110, name: "An-Nasr", meaning: "Pertolongan", ayahs: 3, place: "Madaniyah" },
    SurahMeta { number: 111, name: "Al-Lahab", meaning: "Gejolak Api", ayahs: 5, place: "Makkiyah" },
    SurahMeta { number: 112, name: "Al-Ikhlas", meaning: "Ikhlas", ayahs: 4, place: "Makkiyah" },
    SurahMeta { number: 113, name: "Al-Falaq", meaning: "Waktu Subuh", ayahs: 5, place: "Makkiyah" },
    SurahMeta { number: 114, name: "An-Nas", meaning: "Manusia", ayahs: 6, place: "Makkiyah" },
];

pub fn qari_display_name(qari_id: &str) -> &'static str {
    match qari_id {
        "alafasy" => "Mishary Rashid Alafasy",
        "sudais" => "Abdurrahman As-Sudais",
        "ghamdi" => "Saad Al-Ghamdi",
        "husary" => "Mahmoud Khalil Al-Husary",
        _ => "Mishary Rashid Alafasy",
    }
}

pub fn qari_stream_url(qari_id: &str, surah: u16) -> String {
    let s = surah.clamp(1, 114);
    match qari_id {
        "sudais" => format!("https://cdn.mp3quran.net/audio/abdulrahman-sudais/r1/{:03}.mp3", s),
        "ghamdi" => format!("https://cdn.mp3quran.net/audio/saad-ghamdi/r1/{:03}.mp3", s),
        "husary" => format!("https://cdn.mp3quran.net/audio/mahmoud-husary/r1/{:03}.mp3", s),
        _ => format!("https://cdn.mp3quran.net/audio/mishary-alafasy/r1/{:03}.mp3", s),
    }
}

pub struct QuranPlayer {
    pub current_surah: u16,
    pub qari: String,
    pub is_playing: bool,
    pub is_paused: bool,
    child: Option<Child>,
    started_at: Option<Instant>,
    paused_elapsed: u64,
}

impl QuranPlayer {
    pub fn new(surah: u16, qari: String) -> Self {
        Self {
            current_surah: surah.clamp(1, 114),
            qari,
            is_playing: false,
            is_paused: false,
            child: None,
            started_at: None,
            paused_elapsed: 0,
        }
    }

    fn cache_file_path(&self) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home)
            .join(".cache")
            .join("meera-notch")
            .join("quran")
            .join(&self.qari)
            .join(format!("{:03}.mp3", self.current_surah))
    }

    pub fn play(&mut self, surah: u16, qari: &str) {
        self.stop();
        self.current_surah = surah.clamp(1, 114);
        if !qari.is_empty() {
            self.qari = qari.to_string();
        }

        let cache_path = self.cache_file_path();
        let uri = if cache_path.exists() {
            format!("file://{}", cache_path.display())
        } else {
            let url = qari_stream_url(&self.qari, self.current_surah);
            // Trigger background cache download so subsequent plays are offline
            if let Some(parent) = cache_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let part = cache_path.with_extension("mp3.part");
            let dest = cache_path.clone();
            let dl_url = url.clone();
            std::thread::spawn(move || {
                let status = Command::new("curl")
                    .args(["-s", "-L", "-o", &part.to_string_lossy(), &dl_url])
                    .status();
                if let Ok(st) = status {
                    if st.success() {
                        let _ = fs::rename(&part, &dest);
                    }
                }
            });
            url
        };

        if let Ok(child) = Command::new("gst-launch-1.0")
            .args(["-q", "playbin", &format!("uri={}", uri)])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            self.child = Some(child);
            self.is_playing = true;
            self.is_paused = false;
            self.started_at = Some(Instant::now());
            self.paused_elapsed = 0;
        }
    }

    pub fn toggle(&mut self) {
        if !self.is_playing {
            let s = self.current_surah;
            let q = self.qari.clone();
            self.play(s, &q);
            return;
        }
        if let Some(ref child) = self.child {
            let pid = child.id().to_string();
            if self.is_paused {
                let _ = Command::new("kill").args(["-CONT", &pid]).status();
                self.is_paused = false;
                self.started_at = Some(Instant::now());
            } else {
                let _ = Command::new("kill").args(["-STOP", &pid]).status();
                if let Some(st) = self.started_at.take() {
                    self.paused_elapsed += st.elapsed().as_secs();
                }
                self.is_paused = true;
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.is_playing = false;
        self.is_paused = false;
        self.started_at = None;
        self.paused_elapsed = 0;
    }

    pub fn next_surah(&mut self) {
        let next = if self.current_surah >= 114 {
            1
        } else {
            self.current_surah + 1
        };
        let q = self.qari.clone();
        self.play(next, &q);
    }

    pub fn prev_surah(&mut self) {
        let prev = if self.current_surah <= 1 {
            114
        } else {
            self.current_surah - 1
        };
        let q = self.qari.clone();
        self.play(prev, &q);
    }

    pub fn elapsed_mm_ss(&mut self) -> String {
        if let Some(ref mut child) = self.child {
            if let Ok(Some(_)) = child.try_wait() {
                self.child = None;
                self.is_playing = false;
                self.is_paused = false;
                self.started_at = None;
                self.paused_elapsed = 0;
            }
        }
        let secs = self.paused_elapsed
            + self
                .started_at
                .map(|s| s.elapsed().as_secs())
                .unwrap_or(0);
        format!("{:02}:{:02}", secs / 60, secs % 60)
    }

    pub fn to_map(&mut self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        let idx = (self.current_surah.clamp(1, 114) - 1) as usize;
        let meta = &SURAHS[idx];
        let cached = self.cache_file_path().exists();
        let cache_label = if cached {
            "Tersimpan di cache"
        } else {
            "Streaming & simpan ke cache"
        };

        m.insert("surah_num".to_string(), meta.number.to_string());
        m.insert(
            "title".to_string(),
            format!("{}. {} ({})", meta.number, meta.name, meta.meaning),
        );
        m.insert(
            "subtitle".to_string(),
            format!("{} · {} Ayat · {}", meta.place, meta.ayahs, cache_label),
        );
        m.insert("qari_id".to_string(), self.qari.clone());
        m.insert(
            "qari_name".to_string(),
            qari_display_name(&self.qari).to_string(),
        );
        m.insert("elapsed".to_string(), self.elapsed_mm_ss());
        m.insert(
            "is_playing".to_string(),
            (self.is_playing && !self.is_paused).to_string(),
        );
        m.insert("is_active".to_string(), self.is_playing.to_string());
        m
    }
}

impl Drop for QuranPlayer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_114_surahs_and_6236_ayahs() {
        assert_eq!(SURAHS.len(), 114);
        let total_ayahs: u32 = SURAHS.iter().map(|s| s.ayahs as u32).sum();
        assert_eq!(total_ayahs, 6236);
    }
}
