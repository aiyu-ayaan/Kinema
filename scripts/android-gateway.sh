#!/usr/bin/env bash
# Dev gateway: make the local Kinema server reachable from every connected phone/emulator, then prove it.
# USB phone and emulator: `adb reverse` maps device localhost:PORT to this computer. Broadcast discovery can't cross either link.
# Usage: scripts/android-gateway.sh [port]   (default 3000, the dev server)
set -euo pipefail
PORT="${1:-3000}"
ADB="${ANDROID_HOME:-$HOME/Android/Sdk}/platform-tools/adb"
command -v "$ADB" >/dev/null || ADB=adb

echo "Host server:"
if curl -sk -m3 "https://localhost:$PORT/api/server/info" | grep -q '"name"'; then echo "  ok   https://localhost:$PORT"
else echo "  FAIL nothing answers on https://localhost:$PORT. Run: pnpm dev:backend" >&2; exit 1; fi

serials="$("$ADB" devices | awk 'NR>1 && $2=="device"{print $1}')"
[ -n "$serials" ] || { echo "No device. Plug in a phone (USB debugging) or start an emulator." >&2; exit 1; }

rc=0
for s in $serials; do
  "$ADB" -s "$s" reverse "tcp:$PORT" "tcp:$PORT" >/dev/null
  # toybox nc exit code is unreliable; require a TLS-free TCP connect that sends bytes back is overkill, so just connect.
  if "$ADB" -s "$s" shell "nc -w2 localhost $PORT </dev/null" >/dev/null 2>&1; then
    echo "Device $s: ok   enter localhost:$PORT in the app (accept the self-signed fingerprint)"
  else echo "Device $s: FAIL localhost:$PORT unreachable" >&2; rc=1; fi
done

# Wi-Fi phones skip adb; they need the server bound on the LAN and the port open in the firewall.
echo "Wi-Fi phone, same network: $(hostname -I | tr ' ' '\n' | grep -E '^(10\.|192\.168\.|172\.(1[6-9]|2[0-9]|3[01])\.)' | grep -v '^172\.\(17\|18\|19\|20\|21\)\.' | sed "s|.*|https://&:$PORT|" | tr '\n' ' ')"
exit $rc
