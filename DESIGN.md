# DESIGN.md: Sistem Visual meera-notch

> **Design Read:** Reading this as: Desktop Shell Utility & Preferences UI for a Linux productivity user on Zorin OS 18.1, in a hardware-integrated Adwaita style, dial `ENERGY 1 / RHYTHM 2 / MOTION 2`.

---

## 1. Identitas & Karakter Produk

`meera-notch` adalah komponen shell tingkat sistem yang menempel langsung pada bibir atas layar (`y = 0`) ASUS VivoBook X415EA (1920x1080). Karena bagian atas layar menyatu dengan bezel fisik monitor yang berwarna hitam doff, permukaan notch menggunakan hitam pekat (`#0A0A0C`) dengan telinga lengkung terbalik (*inverse top-corner ears*, radius 12px) di kiri dan kanan atas agar tampak tumbuh langsung dari bingkai layar, persis seperti referensi perangkat keras yang diminta pengguna.

Jendela pengaturan (*Preferences*) menggunakan bahasa desain native Libadwaita (GNOME 46) yang terang, tenang, dan berfokus pada keterbacaan daftar pengaturan.

---

## 2. Three Dials (Liveliness Toolkit)

- **ENERGY 1 (Calm):** Permukaan tenang tanpa dekorasi berlebih agar tidak mengganggu fokus mata saat bekerja di layar utama.
- **RHYTHM 2 (Balanced):** Tata letak kartu di dalam notch membedakan hirarki antara penunjuk waktu utama (ukuran besar) dan daftar jadwal/kontrol sekunder (baris padat).
- **MOTION 2 (Transitions):** Transisi fisik terukur (240ms `cubic-bezier(0.22, 1, 0.36, 1)`) saat notch melebar dari mode ringkas ke hub penuh, serta bar gelombang suara saat murottal berputar. Berhenti otomatis bila `prefers-reduced-motion` aktif.

---

## 3. Palet Warna & Bukti Kontras WCAG AA (R-25, R-29)

Seluruh pasangan warna telah diverifikasi menggunakan `contrast-check.py`:

### A. Permukaan Notch (Hardware Bezel Surface)
| Token | Hex | Peran | Kontras vs Latar | Status WCAG AA |
|---|---|---|---|---|
| `notch-bg` | `#0A0A0C` | Latar utama notch (menyatu dengan bezel fisik monitor) | - | - |
| `notch-surface` | `#16171B` | Kartu dalam hub dan tombol kontrol | - | - |
| `notch-border` | `#282A32` | Garis pemisah tipis bagian bawah dan kartu | - | - |
| `notch-fg` | `#F4F4F6` | Teks utama (nama sholat, surah, jam, persentase) | `18.01:1` (di `#0A0A0C`), `16.31:1` (di `#16171B`) | PASS |
| `notch-muted` | `#A0A4AE` | Teks sekunder (label satuan, nama qari, arti surah) | `7.93:1` (di `#0A0A0C`), `7.18:1` (di `#16171B`) | PASS |
| `notch-accent` | `#2EC27E` | Aksen tunggal: waktu sholat berikutnya, gelombang audio, tab aktif | `8.60:1` (di `#0A0A0C`), `7.79:1` (di `#16171B`) | PASS |
| `notch-on-accent`| `#0A0A0C` | Teks di atas chip/tombol aksen `#2EC27E` | `8.60:1` | PASS |
| `notch-danger` | `#FF5C5C` | Tombol stop rekaman/timer atau aksi konfirmasi daya | `6.12:1` (di `#0A0A0C`) | PASS |

### B. Jendela Pengaturan Libadwaita (Light Mode Default)
| Token | Hex | Peran | Kontras vs Latar | Status WCAG AA |
|---|---|---|---|---|
| `prefs-bg` | `#FAFAFC` | Latar jendela AdwPreferencesWindow | - | - |
| `prefs-card` | `#FFFFFF` | Latar baris grup pengaturan | - | - |
| `prefs-border` | `#DCDCE2` | Garis tepi kartu dan pemisah baris | - | - |
| `prefs-fg` | `#1C1C1F` | Judul grup dan label pengaturan | `16.31:1` | PASS |
| `prefs-muted` | `#5E5E66` | Sub-label penjelasan baris | `6.42:1` (di `#FFFFFF`) | PASS |
| `prefs-accent` | `#166942` | Tombol utama dan status terpilih di latar terang | `6.70:1` (dengan teks `#FFFFFF`) | PASS |

---

## 4. Tipografi & Skala Sudut (R-06, R-11)

- **Font Family:** `Inter` / `Cantarell` (font sistem bawaan GNOME/Zorin OS) untuk keterbacaan UI ukuran kecil, dengan `tabular-nums` wajib pada semua angka jam, hitung mundur, dan persentase agar lebar karakter tidak bergeser saat detik berjalan.
- **Skala Ukuran:**
  - Pill ringkas: `13px` SemiBold (`#F4F4F6`) untuk informasi utama, `12px` Medium (`#A0A4AE`) untuk keterangan pendamping.
  - Angka sorotan di dalam Hub (hitung mundur sholat / timer fokus): `28px` Bold.
  - Judul kartu / tab: `13px` SemiBold.
  - Baris daftar: `13px` Regular.
- **Skala Border Radius (Bukan Semua Pill, R-11):**
  - Bodi Notch Atas: `[0, 0, 18, 18]` (rata di bibir atas `y = 0`, membulat 18px di sudut kiri-kanan bawah) ditambah telinga lengkung 12px di luar sudut kiri-kanan atas.
  - Kartu dalam Hub: `10px`.
  - Baris item / input: `6px`.
  - Tombol kontrol media bundar (Play/Pause/Stop): `16px` (lingkaran penuh `32x32px` sesuai referensi pengguna).

---

## 5. Alasan Satu Baris untuk Setiap Keputusan Utama (R-31)

1. **Warna latar `#0A0A0C` pada notch:** Menyatu dengan bezel fisik layar laptop yang berwarna hitam sehingga notch tampak sebagai bagian dari layar, bukan jendela mengambang biasa.
2. **Telinga lengkung atas (*inverse corner ears*):** Menghilangkan patahan tajam 90 derajat antara garis tepi atas layar dan sisi vertikal notch sesuai gambar referensi pengguna.
3. **Satu warna aksen `#2EC27E` (Emerald):** Menandai satu titik fokus per layar (waktu sholat terdekat atau audio yang sedang diputar) dengan kontras tinggi `8.60:1` di atas hitam.
4. **Tanpa emoji dekoratif di teks UI:** Menjaga ketajaman keterbacaan pada ukuran 12-13px dan menghindari kesan teks buatan AI; status ditunjukkan dengan bentuk geometri вектор sederhana (gelombang suara, titik aktif, ikon play/pause).
5. **Tata letak 6 layar di FigCraft:** Mencakup seluruh kondisi nyata (idle tenang, murottal aktif seperti gambar referensi, hub jadwal sholat & murottal, hub monitor sistem & pomodoro, jendela pengaturan Adwaita, serta state offline/daemon mati sesuai R-27).
