#!/usr/bin/env bash
set -euo pipefail

EXT_UUID="meera-notch@rizqinrr.github.io"
EXT_DEST="${HOME}/.local/share/gnome-shell/extensions/${EXT_UUID}"
BIN_DAEMON="${HOME}/.local/bin/meera-notchd"
BIN_APP="${HOME}/.local/bin/meera-notch"
SERVICE_DEST="${HOME}/.config/systemd/user/meera-notchd.service"
ICON_DEST="${HOME}/.local/share/icons/hicolor/scalable/apps/meera-notch.svg"
APP_DEST="${HOME}/.local/share/applications/meera-notch.desktop"

echo "Menghentikan aplikasi meera-notch..."
pkill -f "meera-notch" 2>/dev/null || true

echo "Menonaktifkan ekstensi ${EXT_UUID}..."
gnome-extensions disable "${EXT_UUID}" 2>/dev/null || true
rm -rf "${EXT_DEST}"

echo "Menghentikan layanan meera-notchd.service..."
systemctl --user disable --now meera-notchd.service 2>/dev/null || true
rm -f "${SERVICE_DEST}"
systemctl --user daemon-reload

echo "Menghapus binary dan shortcut aplikasi..."
rm -f "${BIN_DAEMON}" "${BIN_APP}" "${ICON_DEST}" "${APP_DEST}"

echo "meera-notch berhasil dihapus."
