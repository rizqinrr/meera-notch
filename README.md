# meera-notch

Dynamic top-bezel productivity hub untuk Linux (GNOME Shell 46 di Wayland, Zorin OS 18.1 / Ubuntu 24.04 LTS), dirancang khusus untuk ASUS VivoBook X415EA.

## Arsitektur

`meera-notch` menggunakan arsitektur hibrida ringan untuk menjaga penggunaan RAM di bawah 10 MiB dan bebas lag pada utas kompositor GNOME Shell:

1. **`meera-notchd` (Daemon Rust)**
   Menjalankan seluruh komputasi latar belakang dan menyediakan antarmuka D-Bus sesi `io.github.rizqinrr.MeeraNotch` (`/io/github/rizqinrr/MeeraNotch`, interface `io.github.rizqinrr.MeeraNotch1`):
   - **Jadwal Sholat & Penanggalan Hijriah Offline**: Perhitungan astronomi murni berdasarkan standar Kemenag RI (Subuh -20°, Isya -18°, Ashar Syafi'i 1x bayangan, Ihtiyath +2 menit, Imsak 10 menit sebelum Subuh).
   - **Murottal Al-Qur'an 114 Surah**: Pemutar audio latar belakang menggunakan GStreamer (`gst-launch-1.0`) / PipeWire dengan unduhan cache otomatis di `~/.cache/meera-notch/quran/<qari>/<nnn>.mp3` untuk 4 Qari (Mishary Rashid Alafasy, Abdurrahman As-Sudais, Saad Al-Ghamdi, Mahmoud Khalil Al-Husary).
   - **Telemetri Perangkat Keras & Baterai**: Membaca `/proc/stat` (8 utas i5-1135G7), suhu paket `coretemp`, `/proc/meminfo`, kecepatan jaringan `/proc/net/dev`, serta status baterai `BAT0` dan batas pengisian aktif `ncharge` (`charge_control_end_threshold`).
   - **Cuaca & Lokasi**: Geolokasi otomatis berbasis IP (`ipapi.co`) dan prakiraan cuaca `wttr.in` dengan cache lokal 30 menit (`~/.cache/meera-notch/weather.json`).
   - **Fokus Pomodoro, Catatan Kilat & Daya**: Timer kerja/istirahat (preset 25/5 dan 50/10) dengan riwayat sesi di `~/.local/share/meera-notch/pomodoro.jsonl`, catatan teks di `~/.local/share/meera-notch/notes.txt`, dan aksi sistem (`lock`, `suspend`, serta `reboot`/`poweroff` dengan pengaman dua klik).

2. **`meera-notch@rizqinrr.github.io` (Ekstensi GNOME Shell 46)**
   Antarmuka Clutter/St tipis yang menempel di bibir atas layar (`y = 0`) menggunakan `Main.layoutManager.addTopChrome()` dengan lengkungan kuping terbalik (*inverse top-corner ears* 14px via Cairo) sehingga menyatu dengan bezel layar atas.

## Instalasi

Pastikan toolchain Rust (`cargo`), `curl`, dan `gst-launch-1.0` sudah terpasang, lalu jalankan skrip instalasi:

```bash
chmod +x packaging/install.sh packaging/uninstall.sh
./packaging/install.sh
```

## Uji Cepat D-Bus

Anda dapat memeriksa status daemon secara langsung melalui `busctl`:

```bash
busctl --user call io.github.rizqinrr.MeeraNotch /io/github/rizqinrr/MeeraNotch io.github.rizqinrr.MeeraNotch1 Ping
busctl --user get-property io.github.rizqinrr.MeeraNotch /io/github/rizqinrr/MeeraNotch io.github.rizqinrr.MeeraNotch1 Pill
busctl --user get-property io.github.rizqinrr.MeeraNotch /io/github/rizqinrr/MeeraNotch io.github.rizqinrr.MeeraNotch1 Prayer
```

## Lisensi

Hak Cipta (c) 2026 M Rizqi Nurrahman.
Didistribusikan di bawah lisensi **Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International (CC BY-NC-SA 4.0)**. Bebas dimodifikasi dan dibagikan ulang dengan atribusi dan lisensi serupa, namun **dilarang keras untuk diperjualbelikan atau digunakan secara komersial**. Lihat berkas `LICENSE` untuk teks lengkap.
