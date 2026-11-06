#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXT_UUID="meera-notch@rizqinrr.github.io"
EXT_DEST="${HOME}/.local/share/gnome-shell/extensions/${EXT_UUID}"
BIN_DEST="${HOME}/.local/bin"
SYSTEMD_USER_DIR="${HOME}/.config/systemd/user"
ICON_DEST="${HOME}/.local/share/icons/hicolor/scalable/apps"
APP_DEST="${HOME}/.local/share/applications"

echo "[1/5] Membangun binary rilis meera-notchd..."
cd "${REPO_DIR}"
cargo build --release -p meera-notchd

echo "[2/5] Memasang meera-notchd dan aplikasi GUI meera-notch ke ${BIN_DEST}..."
mkdir -p "${BIN_DEST}"
install -m 0755 "${REPO_DIR}/target/release/meera-notchd" "${BIN_DEST}/meera-notchd"
install -m 0755 "${REPO_DIR}/app/meera-notch-app.py" "${BIN_DEST}/meera-notch"

echo "[3/5] Memasang ikon dan shortcut menu aplikasi (meera-notch.desktop)..."
mkdir -p "${ICON_DEST}" "${APP_DEST}"
install -m 0644 "${REPO_DIR}/packaging/meera-notch.svg" "${ICON_DEST}/meera-notch.svg"
sed "s|^Exec=.*|Exec=${BIN_DEST}/meera-notch|" "${REPO_DIR}/packaging/meera-notch.desktop" > "${APP_DEST}/meera-notch.desktop"
chmod 0644 "${APP_DEST}/meera-notch.desktop"
update-desktop-database "${APP_DEST}" 2>/dev/null || true

echo "[4/5] Memasang layanan systemd user meera-notchd.service..."
mkdir -p "${SYSTEMD_USER_DIR}"
install -m 0644 "${REPO_DIR}/packaging/systemd/meera-notchd.service" "${SYSTEMD_USER_DIR}/meera-notchd.service"
systemctl --user daemon-reload
systemctl --user enable --now meera-notchd.service

echo "[5/5] Memasang ekstensi GNOME Shell 46 (${EXT_UUID})..."
mkdir -p "${EXT_DEST}"
install -m 0644 "${REPO_DIR}/extension/metadata.json" "${EXT_DEST}/metadata.json"
install -m 0644 "${REPO_DIR}/extension/extension.js" "${EXT_DEST}/extension.js"
install -m 0644 "${REPO_DIR}/extension/notch.js" "${EXT_DEST}/notch.js"
install -m 0644 "${REPO_DIR}/extension/prefs.js" "${EXT_DEST}/prefs.js"
install -m 0644 "${REPO_DIR}/extension/stylesheet.css" "${EXT_DEST}/stylesheet.css"
gnome-extensions enable "${EXT_UUID}" 2>/dev/null || true

echo ""
echo "Instalasi meera-notch selesai!"
echo "- Klik ikon 'meera-notch' di Menu Aplikasi Zorin atau jalankan: ~/.local/bin/meera-notch"
