#!/bin/bash

set -euo pipefail

APP_NAME="ExamUQ Client.app"
APP_DEFAULT="/Applications/${APP_NAME}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_LOCAL="${SCRIPT_DIR}/${APP_NAME}"

if [ -d "$APP_DEFAULT" ]; then
  TARGET_APP="$APP_DEFAULT"
elif [ -d "$APP_LOCAL" ]; then
  TARGET_APP="$APP_LOCAL"
else
  osascript -e 'display dialog "ExamUQ Client.app tidak ditemukan. Pindahkan aplikasinya ke folder Applications terlebih dahulu, lalu jalankan file ini lagi." buttons {"OK"} default button "OK" with title "ExamUQ Client"'
  exit 1
fi

xattr -dr com.apple.quarantine "$TARGET_APP"
open "$TARGET_APP"

osascript -e 'display dialog "Pemeriksaan selesai. ExamUQ Client akan dibuka sekarang." buttons {"OK"} default button "OK" with title "ExamUQ Client"'
