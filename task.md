# meera-notch: Daftar Tugas

Urutan dari atas ke bawah. Setiap task punya kriteria selesai.

Legenda: `[ ]` belum · `[~]` sedang · `[x]` selesai · `[-]` ditunda

---

## Fase 0: Keputusan

- [x] **T0.1** Jawab D1 s/d D6 di `PLAN.md` bagian 6.
- [x] **T0.2** Pilih lisensi final (`CC BY-NC-SA 4.0`).

## Fase 1: Setup repo

- [x] **T1.1** `git init`, set `user.name "M Rizqi Nurrahman"` dan `user.email 152271198+rizqinrr@users.noreply.github.com` (lokal repo).
- [x] **T1.2** Tulis `LICENSE` (`CC BY-NC-SA 4.0`) dan `.gitignore`.
- [x] **T1.3** Cargo workspace: `Cargo.toml` root + crate `daemon` (binary `meera-notchd`).
- [x] **T1.4** Gunakan pipeline audio GStreamer (`gst-launch-1.0`) + PipeWire bawaan sistem tanpa dependensi root.
- [x] **T1.5** Kerangka `extension/`: `metadata.json` (`meera-notch@rizqinrr.github.io`, `shell-version: ["46"]`), `extension.js`.
- [x] **T1.6** `README.md` lengkap (arsitektur, instalasi, uji D-Bus, lisensi).

## Fase 2: Inti daemon

- [x] **T2.1** `config.rs`: struct `Config` dengan baca/tulis atomik `~/.config/meera-notch/config.toml` + unit test round-trip.
- [x] **T2.2** `main.rs`: runtime `tokio`, logging `tracing`, D-Bus session server.
- [x] **T2.3** Daftarkan `io.github.rizqinrr.MeeraNotch` di `/io/github/rizqinrr/MeeraNotch` (`io.github.rizqinrr.MeeraNotch1`) dengan `Version` dan `Ping`.

## Fase 3: Modul data daemon

### 3A. Lokasi
- [x] **T3A.1** GeoIP `ipapi.co` dengan timeout dan parser JSON di `weather.rs`.
- [x] **T3A.2** Method D-Bus `RefreshLocation` yang menyimpan hasil ke `config.toml`.

### 3B. Jadwal sholat
- [x] **T3B.1** `prayer.rs`: posisi matahari, waktu Dzuhur, Subuh -20°, Terbit/Maghrib -0.8333°, Isya -18°, Ashar Syafi'i (bayangan 1x).
- [x] **T3B.2** Terapkan ihtiyath +2 menit, Imsak = Subuh - 10 menit, Dhuha = Terbit + 15 menit.
- [x] **T3B.3** Fungsi `compute_schedule` menghitung waktu berikutnya dan hitung mundur (`countdown` & `countdown_hms`), termasuk pergantian ke Subuh esok hari setelah Isya.
- [x] **T3B.4** Unit test akurasi jadwal Bandung (`test_bandung_prayer_schedule`).
- [x] **T3B.5** Konversi tanggal Hijriah tabular (`gregorian_to_hijri_approx`) dengan label "(perkiraan)".
- [x] **T3B.6** Ekspos property `Prayer` dan `Pill` di D-Bus.

### 3C. Monitor sistem
- [x] **T3C.1** CPU: parse `/proc/stat`, hitung persen total + 8 thread (`i5-1135G7`).
- [x] **T3C.2** RAM dari `/proc/meminfo` (`MemAvailable`).
- [x] **T3C.3** Suhu: `coretemp` / `x86_pkg_temp`.
- [x] **T3C.4** Baterai `BAT0`: kapasitas, status, daya Watt, dan `charge_control_end_threshold` (`ncharge`).
- [x] **T3C.5** Jaringan `/proc/net/dev` RX/TX KB/s.

### 3D. Pomodoro dan catatan
- [x] **T3D.1** `pomodoro.rs`: state Fokus/Istirahat, preset 25/5 dan 50/10.
- [x] **T3D.2** Log sesi selesai ke `~/.local/share/meera-notch/pomodoro.jsonl`.
- [x] **T3D.3** `notes.rs`: `NotesGet`/`NotesSet` dengan tulis atomik ke `~/.local/share/meera-notch/notes.txt`.

### 3E. Cuaca
- [x] **T3E.1** `weather.rs`: ambil `wttr.in` format JSON (`?format=j1`).
- [x] **T3E.2** Cache 30 menit di `~/.cache/meera-notch/weather.json`.

### 3F. Murottal
- [x] **T3F.1** 4 Qari terverifikasi (`alafasy`, `sudais`, `ghamdi`, `husary`).
- [x] **T3F.2** `quran.rs`: 114 surah lengkap, unit test `test_114_surahs_and_6236_ayahs` lolos.
- [x] **T3F.3** Unduh cache atomik `.part` ke `~/.cache/meera-notch/quran/<qari>/<nnn>.mp3`.
- [x] **T3F.4** Pemutar `gst-launch-1.0` dengan jeda/lanjut (`SIGSTOP`/`SIGCONT`), `QuranPlay`, `QuranToggle`, `QuranNext`, `QuranPrev`, `QuranStop`.

### 3G. Aksi daya
- [x] **T3G.1** `power.rs`: `lock`, `suspend`, `reboot`, `poweroff`.

## Fase 4: Extension GNOME 46

- [x] **T4.1** `extension/extension.js`: pasang ke `Main.layoutManager.addTopChrome` di tengah bibir atas monitor utama (`y = 0`).
- [x] **T4.2** `extension/notch.js`: kuping lengkung terbalik (*inverse top-corner ears* 14px dengan Cairo) sehingga menyatu dengan bezel layar atas.
- [x] **T4.3** Pill ringkas (`280x36 px`): jadwal sholat berikutnya + hitung mundur, status tilawah aktif, Pomodoro, dan baterai `BAT0`.
- [x] **T4.4** Mode buka: `hover`, `click`, dan `always` + transisi halus Clutter (`EASE_OUT_QUAD`).
- [x] **T4.5** Hub 3 tab: `Sholat & Tilawah`, `Sistem & Cuaca`, dan `Fokus & Daya` (termasuk Catatan Kilat dan pengaman 2 klik untuk Mulai Ulang / Matikan).
- [x] **T4.6** `extension/stylesheet.css`: palet `#0A0A0C` / `#16171B` / `#2EC27E` terverifikasi WCAG AA (`contrast-check.py`).
- [x] **T4.7** `extension/prefs.js`: jendela preferensi Adwaita terhubung langsung ke D-Bus (`SetOpenMode`, `RefreshLocation`, `QuranPlay`).

## Fase 5: Packaging & Verifikasi

- [x] **T5.1** `packaging/systemd/meera-notchd.service`
- [x] **T5.2** `packaging/install.sh` & `packaging/uninstall.sh`
- [x] **T5.3** `cargo test` dan `cargo clippy --workspace -- -D warnings` bersih.
