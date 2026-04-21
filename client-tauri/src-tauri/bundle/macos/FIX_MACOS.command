#!/bin/bash

set -euo pipefail

APP_NAME="ExamUQ Client.app"
DEFAULT_APP="/Applications/${APP_NAME}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LOCAL_APP="${SCRIPT_DIR}/${APP_NAME}"

if [ -d "$DEFAULT_APP" ]; then
  TARGET_APP="$DEFAULT_APP"
elif [ -d "$LOCAL_APP" ]; then
  TARGET_APP="$LOCAL_APP"
else
  osascript -e 'display dialog "ExamUQ Client.app tidak ditemukan. Pindahkan aplikasinya ke folder Applications terlebih dahulu, lalu jalankan script ini lagi." buttons {"OK"} default button "OK" with title "ExamUQ Client"'
  exit 1
fi

xattr -dr com.apple.quarantine "$TARGET_APP"
open "$TARGET_APP"

osascript -e 'display dialog "Quarantine sudah dibersihkan. ExamUQ Client akan dibuka sekarang." buttons {"OK"} default button "OK" with title "ExamUQ Client"'
