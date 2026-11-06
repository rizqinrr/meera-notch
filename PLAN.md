# meera-notch — Rencana Arsitektur & Implementasi

> **Proyek:** `meera-notch`, pill ala Dynamic Island di tengah atas layar, untuk GNOME Shell 46
> **Lisensi:** CC BY-NC-SA 4.0 (boleh dimodifikasi dan disebar, dilarang dijual atau dipakai komersial). Lihat §7 untuk catatan lisensi bagian extension.
> **Mesin target:** ASUS VivoBook X415EA, Intel Core i5-1135G7 (4C/8T), RAM 7,5 GiB, layar 1920×1080 @ scale 1.0
> **OS:** Zorin OS 18.1 (basis Ubuntu 24.04), GNOME Shell 46.0, Wayland, PipeWire + WirePlumber
> **Workspace:** `/media/rizqi/DATA/01_Coding/nProjects/meera-notch`
> **Maintainer:** M Rizqi Nurrahman (`rizqinrr`)

---

## 1. Latar belakang

- NexNotch (GPL-3.0+) hanya mendukung GNOME Shell 49–50 dan menjalankan seluruh logika (polling sysfs tiap detik, HTTP, I/O file) di thread JavaScript GNOME Shell. Di GNOME 46 extension itu tidak bisa dipasang.
- `meera-notch` adalah **tulis ulang dari nol (clean-room)**. NexNotch hanya dipakai sebagai referensi fitur. Tidak ada kode, CSS, atau aset NexNotch yang disalin.
- RAM bebas di laptop ini sering tinggal ±700 MiB, jadi kerja berat dipindah ke daemon Rust kecil. Extension hanya merender.
- Taskbar Zorin ada di **bawah**, jadi tengah atas layar kosong. Tapi window yang di-maximize memakai area itu, jadi pill akan menimpa bagian tengah titlebar (lihat §6, keputusan D1).

## 2. Arsitektur

```
┌──────────────────────────── GNOME Shell 46 (Wayland) ────────────────────────────┐
│  Extension  meera-notch@rizqinrr.github.io   (GJS, ESM)                           │
│  - Main.layoutManager.addTopChrome(), tengah atas monitor utama                  │
│  - State: collapsed (pill) / expanded (hub bertab)                               │
│  - Mode buka: hover | klik | selalu terbuka   (GSettings)                        │
│  - Shortcut Super+N untuk toggle                                                 │
│  - Tanpa logika bisnis: hanya render data dari D-Bus dan meneruskan aksi         │
└───────────────────────────────────────▲──────────────────────────────────────────┘
                                        │ D-Bus session bus
                                        │ name: io.github.rizqinrr.MeeraNotch
                                        │ path: /io/github/rizqinrr/MeeraNotch
┌───────────────────────────────────────▼──────────────────────────────────────────┐
│  Daemon  meera-notchd  (Rust, systemd --user service)                            │
│  prayer   jadwal sholat offline (Kemenag RI) + countdown                          │
│  quran    pemutar murottal (rodio → ALSA → PipeWire), cache file MP3             │
│  system   CPU, RAM, suhu, baterai BAT0, jaringan, disk dari /proc dan /sys        │
│  weather  wttr.in, cache 30 menit                                                 │
│  location GeoIP sekali saat start, disimpan, bisa ditimpa manual                  │
│  pomodoro state machine + log JSONL                                               │
│  notes    simpan teks ke disk                                                     │
│  power    lock / suspend / restart / shutdown lewat logind & ScreenSaver D-Bus    │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Pembagian tanggung jawab pengaturan

| Pengaturan | Disimpan di | Pemilik |
|---|---|---|
| Mode buka (hover/klik/selalu), shortcut, tab aktif, lebar pill | GSettings `org.gnome.shell.extensions.meera-notch` | Extension |
| Lokasi (lat/lon/kota, otomatis/manual), qari, koreksi menit sholat, interval cuaca, durasi pomodoro | `~/.config/meera-notch/config.toml` | Daemon |

`prefs.js` membaca dan menulis bagian daemon lewat method D-Bus `GetConfig` / `SetConfig`, sehingga hanya daemon yang menulis file config.

### 2.2 Kontrak D-Bus (v1)

Interface `io.github.rizqinrr.MeeraNotch1`:

| Jenis | Nama | Isi |
|---|---|---|
| Property | `Pill` (`a{sv}`) | jam, sholat berikutnya + sisa menit, status murottal, status pomodoro |
| Property | `Prayer`, `Quran`, `System`, `Weather`, `Pomodoro` (`a{sv}`) | data per tab |
| Method | `SetExpanded(b)` | extension memberi tahu daemon apakah hub terbuka |
| Method | `QuranPlay(su)`, `QuranToggle()`, `QuranNext()`, `QuranPrev()`, `QuranSeek(t)` | kontrol murottal |
| Method | `PomodoroStart(s)`, `PomodoroPause()`, `PomodoroReset()` | kontrol pomodoro |
| Method | `NotesGet() → s`, `NotesSet(s)` | catatan |
| Method | `Power(s)` | `lock`, `suspend`, `reboot`, `poweroff` |
| Method | `GetConfig() → s`, `SetConfig(s)` | TOML/JSON config |
| Method | `RefreshLocation()` | ulangi GeoIP |
| Signal | `PropertiesChanged` standar | dipancarkan hanya saat nilai berubah |

### 2.3 Hemat daya (koreksi dari rencana awal)

- **Saat collapsed:** daemon hanya memperbarui `Pill` sekali per menit (jam dan countdown sholat), kecuali murottal atau pomodoro sedang jalan (sekali per detik).
- **Saat expanded:** `system` dipoll sekali per detik, dan hanya untuk tab yang terlihat.
- Tidak ada polling 1 detik terus-menerus seperti NexNotch.

## 3. Modul

### 3.1 Jadwal sholat (`prayer`)
- Hitung offline, metode **Kemenag RI**: Subuh −20°, Isya −18°, Ashar madzhab Syafi'i (bayangan = 1× tinggi benda), **ihtiyath +2 menit**, **Imsak = Subuh −10 menit**.
- Waktu yang ditampilkan: Imsak, Subuh, Terbit, Dhuha, Dzuhur, Ashar, Maghrib, Isya.
- Pill: nama sholat berikutnya + sisa waktu ("Ashar · 35m").
- Saat masuk waktu: notifikasi GNOME (default). Audio adzan opsional, mati secara default (lihat D4).
- Koreksi manual ±menit per waktu sholat lewat config.
- Tanggal Hijriah (aritmetika/tabular, ditandai sebagai perkiraan).
- Divalidasi terhadap jadwal resmi Kemenag untuk Bandung (selisih ≤ 1 menit).

### 3.2 Murottal (`quran`)
- Katalog 114 surah disematkan di binary: nomor, nama Latin, nama Arab, arti (ID), jumlah ayat, Makkiyah/Madaniyah.
- Sumber audio: CDN murottal per surah (kandidat: mp3quran.net). Pola URL dan izin pemakaian diverifikasi dulu di task T3.1.
- `rodio` butuh sumber yang bisa `Read + Seek`, jadi file **diunduh dulu ke cache** `~/.cache/meera-notch/quran/<qari>/<nnn>.mp3`, lalu diputar. Selama mengunduh, UI menampilkan progres.
- Batas cache bisa diatur (default 1 GiB, LRU).
- Kontrol: play/pause, surah berikutnya/sebelumnya, seek, ganti qari, ulangi surah.
- Opsional: daemon mengekspos **MPRIS** supaya tombol media keyboard dan kontrol media GNOME ikut bekerja.

### 3.3 Monitor sistem (`system`)
- CPU total + 8 thread dari `/proc/stat`, load average.
- RAM dan swap dari `/proc/meminfo` (swap saat ini 0 B, tampilkan "tidak ada swap").
- Suhu paket CPU: cari hwmon `coretemp` (`Package id 0`), fallback ke `thermal_zone` bertipe `x86_pkg_temp`.
- Baterai `BAT0`: kapasitas, status, daya (W), dan `charge_control_end_threshold` (nilai yang diatur `ncharge`).
- Jaringan: rx/tx per detik dari `/proc/net/dev`, abaikan `lo`.
- Disk: pemakaian `/` dan `/media/rizqi/DATA` (statvfs), I/O dari `/proc/diskstats`.

### 3.4 Cuaca (`weather`)
- `wttr.in/<lat>,<lon>?format=j1`, cache 30 menit, retry dengan backoff saat offline.
- Lokasi dari modul `location`.

### 3.5 Lokasi (`location`)
- GeoIP sekali saat pertama jalan (ipapi.co, fallback ip-api.com), hasil disimpan ke config. Tes saat ini: Bandung, −6,92 / 107,62, Asia/Jakarta.
- Tidak dipanggil ulang tiap start. Ulang hanya lewat tombol "Deteksi ulang" atau jika config kosong.
- Bisa diganti manual (nama kota atau koordinat) karena GeoIP berbasis ISP bisa meleset puluhan km, dan itu memengaruhi jadwal sholat.

### 3.6 Pomodoro (`pomodoro`)
- Preset 25/5 dan 50/10, durasi bisa diatur.
- Log sesi di `~/.local/share/meera-notch/pomodoro.jsonl`. Ringkasan: hari ini, 7 hari, streak.
- Bunyi chime lewat engine audio yang sama.

### 3.7 Catatan (`notes`)
- Satu area teks, autosave (debounce 1 detik) ke `~/.local/share/meera-notch/notes.txt`.
- Saat mengetik, extension memegang key focus supaya input tidak bocor ke window lain.

### 3.8 Aksi cepat (`power`)
- Lock: `org.gnome.ScreenSaver.Lock`. Suspend/restart/shutdown: `org.freedesktop.login1.Manager`.
- Restart dan shutdown butuh dua kali klik (tombol "dipersenjatai" selama 3 detik).

### 3.9 Ditunda (bukan v1)
- Indikator mic/kamera aktif (butuh binding PipeWire atau scan `/proc/*/fd`; GNOME 46 sudah punya indikator privasi sendiri).
- Persentase baterai perangkat Bluetooth.
- Kuota Claude Code.

## 4. Stack Rust

| Kebutuhan | Crate | Catatan |
|---|---|---|
| D-Bus | `zbus` 5 | server interface + client logind/ScreenSaver |
| Async runtime | `tokio` (current-thread) | satu thread untuk event loop |
| HTTP | `ureq` + rustls | ringan, dijalankan di `spawn_blocking` |
| Audio | `rodio` (fitur `mp3` saja) | butuh `libasound2-dev` saat build |
| Config | `serde` + `toml` | |
| Waktu | `chrono` | zona Asia/Jakarta dari config |
| Log | `tracing` + journald | |

Matematika jadwal sholat ditulis sendiri (rumus astronomi standar), tanpa crate tambahan.

## 5. Target performa

| Metrik | Target |
|---|---|
| RAM daemon idle (collapsed, tanpa audio) | ≤ 15 MB RSS |
| RAM daemon saat memutar murottal | ≤ 30 MB RSS |
| CPU daemon idle | ≤ 0,1 % rata-rata |
| Tambahan RAM gnome-shell karena extension | ≤ 10 MB |
| Animasi expand/collapse | 250 ms, tanpa frame drop di Intel Iris Xe |
| Ukuran binary release (strip, LTO) | ≤ 8 MB |

## 6. Keputusan yang masih terbuka

| ID | Pertanyaan | Usulan default |
|---|---|---|
| D1 | Pill menimpa tengah titlebar window yang di-maximize. Diterima, atau pill disembunyikan saat ada window maximize/fullscreen? | Tetap tampil, sembunyi otomatis hanya saat fullscreen |
| D2 | Mode "selalu terbuka": hub penuh selalu tampil (menutupi konten), atau pill versi lebar berisi info ringkas? | Pill lebar (jam · sholat berikutnya · murottal), tidak menutupi area kerja |
| D3 | Bahasa UI | Bahasa Indonesia |
| D4 | Adzan bersuara saat masuk waktu sholat? | Notifikasi saja; adzan opsional |
| D5 | Qari default | Mishary Rashid Alafasy |
| D6 | Lisensi: tetap CC BY-NC-SA 4.0, atau PolyForm Noncommercial 1.0.0 (lisensi non-komersial yang dirancang untuk software)? | Lihat §7 |

## 7. Catatan lisensi

- Creative Commons sendiri **tidak menyarankan** lisensi CC untuk software, karena tidak mengatur source code, paten, atau linking. Alternatif yang memang ditulis untuk software dan tetap melarang komersial: **PolyForm Noncommercial 1.0.0** (boleh modifikasi, dilarang komersial, tapi tidak ada klausul ShareAlike).
- Bagian **extension** memakai API internal GNOME Shell (GPL-2.0+). Extension dengan lisensi non-komersial bisa dipakai pribadi dan dibagikan lewat GitHub, tapi **tidak akan diterima di extensions.gnome.org**, yang mensyaratkan lisensi kompatibel GPL.
- Bukan nasihat hukum.

## 8. Struktur folder

```
meera-notch/
├── Cargo.toml                  # workspace
├── LICENSE
├── README.md
├── PLAN.md
├── task.md
├── daemon/
│   ├── Cargo.toml
│   ├── data/surah.json         # metadata 114 surah (di-embed)
│   └── src/
│       ├── main.rs
│       ├── bus.rs              # interface D-Bus
│       ├── config.rs
│       ├── location.rs
│       ├── prayer.rs
│       ├── hijri.rs
│       ├── quran/{mod.rs, catalog.rs, cache.rs, player.rs}
│       ├── system.rs
│       ├── weather.rs
│       ├── pomodoro.rs
│       ├── notes.rs
│       └── power.rs
├── extension/
│   ├── metadata.json           # uuid meera-notch@rizqinrr.github.io, shell-version ["46"]
│   ├── extension.js
│   ├── notch.js                # pill + hub
│   ├── dbus.js                 # proxy ke daemon
│   ├── tabs/{prayer.js, quran.js, system.js, weather.js, pomodoro.js, notes.js, power.js}
│   ├── prefs.js
│   ├── stylesheet.css
│   └── schemas/org.gnome.shell.extensions.meera-notch.gschema.xml
├── packaging/
│   ├── systemd/meera-notchd.service
│   ├── install.sh
│   └── uninstall.sh
└── docs/
    ├── dbus-api.md
    └── bench.md
```
