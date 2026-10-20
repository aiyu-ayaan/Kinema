#!/usr/bin/env bash
# Build the Android app, install it on a connected phone or emulator and open it.
# Starts the first emulator when nothing is connected. Run `pnpm dev` (or `pnpm dev:backend`) alongside for the server.
set -euo pipefail
cd "$(dirname "$0")/.."
SDK="${ANDROID_HOME:-$HOME/Android/Sdk}"
ADB="$SDK/platform-tools/adb"

if ! "$ADB" devices | awk 'NR>1 && $2=="device"' | grep -q .; then
  AVD="$("$SDK/emulator/emulator" -list-avds | head -1)"
  [ -n "$AVD" ] || { echo "No phone connected and no emulator set up. Plug in a phone with USB debugging on." >&2; exit 1; }
  echo "Starting emulator $AVD…"
  nohup "$SDK/emulator/emulator" -avd "$AVD" >/dev/null 2>&1 &
  "$ADB" wait-for-device
  until [ "$("$ADB" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = "1" ]; do sleep 2; done
fi

(cd apps/android && ./gradlew :app:installDevDebug)
bash scripts/android-gateway.sh || true
"$ADB" shell am start -n com.aiyu.kinema.dev/com.aiyu.kinema.MainActivity >/dev/null
echo "Kinema Dev is open. Dev server: tap the discovered server, or enter localhost:3000."
