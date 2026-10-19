#!/bin/sh
# Install, update or remove the Kinema media server on a Linux home server (x86_64 or aarch64).
#
#   curl -fsSL https://raw.githubusercontent.com/aiyu-ayaan/Kinema/master/scripts/install-server.sh | sudo sh
#   ... | sudo sh -s -- --channel beta     beta builds (and stable ones)
#   ... | sudo sh -s -- --version 0.0.1    a specific release
#   ... | sudo sh -s -- --port 32400       the port to try first (default 3000)
#   ... | sudo sh -s -- --uninstall        remove it (the library in /var/lib/kinema stays)
#
# Like Plex: the server, web UI and ffmpeg go to /opt/kinema, a systemd service starts it at boot,
# and the database and caches live in /var/lib/kinema. It runs as the user who ran sudo, so it can
# read that user's media folders; as plain root it runs as a `kinema` system user. If the port is
# taken, the next free one is used. Re-running it updates and keeps the settings.
#
# POSIX sh: it is piped into whatever `sh` is, which on Debian and Ubuntu is dash.
set -eu

API="${KINEMA_RELEASES_URL:-https://api.github.com/repos/aiyu-ayaan/Kinema/releases?per_page=100}"
APP=/opt/kinema
DATA=/var/lib/kinema
UNIT=/etc/systemd/system/kinema.service

CHANNEL=auto
VERSION=""
FROM_FILE=""
PORT=3000
UNINSTALL=0

die() { printf 'error: %s\n' "$*" >&2; exit 1; }

while [ $# -gt 0 ]; do
  case "$1" in
    --channel) CHANNEL="${2:?--channel needs a value}"; shift 2 ;;
    --version) VERSION="${2:?--version needs a value}"; VERSION="${VERSION#v}"; shift 2 ;;
    --port) PORT="${2:?--port needs a value}"; shift 2 ;;
    --from-file) FROM_FILE="${2:?--from-file needs a path}"; shift 2 ;;
    --uninstall) UNINSTALL=1; shift ;;
    -h | --help) sed -n '2,8p' "$0" 2>/dev/null || true; echo "  --from-file PATH   install a local kinema-server tarball"; exit 0 ;;
    *) die "unknown option: $1" ;;
  esac
done
case "$CHANNEL" in auto | stable | beta | alpha) ;; *) die "--channel must be auto, stable, beta or alpha" ;; esac
case "$PORT" in '' | *[!0-9]*) die "--port must be a number" ;; esac
[ "$(id -u)" -eq 0 ] || die "run this with sudo: it installs a system service."
HAS_SYSTEMD=0
[ -d /run/systemd/system ] && command -v systemctl >/dev/null 2>&1 && HAS_SYSTEMD=1

if [ "$UNINSTALL" -eq 1 ]; then
  if [ "$HAS_SYSTEMD" -eq 1 ]; then
    systemctl disable --now kinema 2>/dev/null || true
    rm -f "$UNIT"
    systemctl daemon-reload
  fi
  rm -rf "$APP" "$APP.new"
  echo "Kinema is removed. Its library and settings are kept in $DATA; delete that folder to forget them."
  exit 0
fi

case "$(uname -m)" in
  x86_64 | amd64) ARCH=x86_64 ;;
  aarch64 | arm64) ARCH=aarch64 ;;
  *) die "Kinema is built for x86_64 and aarch64; this machine is $(uname -m)." ;;
esac

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK" "$APP.new"' EXIT INT TERM

if [ -n "$FROM_FILE" ]; then
  cp "$FROM_FILE" "$WORK/kinema.tar.gz" || die "cannot read $FROM_FILE"
  label="$(basename "$FROM_FILE")"
else
  if command -v curl >/dev/null 2>&1; then
    fetch() { curl -fsSL --retry 3 "$1"; }
    fetch_to() { curl -fL --retry 3 --progress-bar -o "$2" "$1"; }
  elif command -v wget >/dev/null 2>&1; then
    fetch() { wget -qO- "$1"; }
    fetch_to() { wget -q --show-progress -O "$2" "$1"; }
  else
    die "curl or wget is needed to download Kinema."
  fi

  echo "Looking up Kinema releases..."
  releases="$(fetch "$API")" || die "could not reach GitHub. Try again in a while."
  # Every server tarball for this machine as "<sort key> <version> <url>"; the newest is last after
  # a sort. The key orders by number, then alpha < beta < stable, then the pre-release number.
  # A channel takes its own builds and every steadier one.
  newest="$(printf '%s\n' "$releases" | grep -oE "https://[^\"]*/kinema-server-[0-9][^\"/]*-linux-$ARCH\.tar\.gz" | sort -u |
    awk -v channel="$CHANNEL" -v want="$VERSION" -v arch="$ARCH" '
      {
        v = $0; sub(/.*\/kinema-server-/, "", v); sub("-linux-" arch "\\.tar\\.gz$", "", v)
        if (v !~ /^[0-9]+\.[0-9]+\.[0-9]+(-(alpha|beta)\.[0-9]+)?$/) next
        split(v, p, /[.-]/)
        stage = 2; n = 99999
        if (p[4] == "alpha") { stage = 0; n = p[5] } else if (p[4] == "beta") { stage = 1; n = p[5] }
        if (want != "") {
          if (v == want) printf "%08d.%08d.%08d.%d.%08d %s %s\n", p[1], p[2], p[3], stage, n, v, $0
          next
        }
        lines[NR] = sprintf("%08d.%08d.%08d.%d.%08d %s %s", p[1], p[2], p[3], stage, n, v, $0)
        stages[NR] = stage
        if (stage > max_stage_seen) max_stage_seen = stage
      }
      END {
        if (want != "") exit
        target_stage = max_stage_seen
        for (i in lines) {
          st = stages[i]
          if (channel == "alpha") {
            print lines[i]
          } else if (channel == "beta") {
            if (st >= 1) print lines[i]
          } else if (channel == "stable") {
            if (st == 2) print lines[i]
          } else {
            # auto: pick highest available stage (stable, then beta, then alpha)
            if (st == target_stage) print lines[i]
          }
        }
      }
    ' | sort | tail -n 1)"
  [ -n "$newest" ] || die "no $ARCH server build found${VERSION:+ for $VERSION}. See the releases page."
  label="$(printf '%s' "$newest" | cut -d' ' -f2)"
  echo "Downloading Kinema $label..."
  fetch_to "$(printf '%s' "$newest" | cut -d' ' -f3)" "$WORK/kinema.tar.gz" || die "the download failed. Nothing was changed."
fi

mkdir -p "$WORK/x"
tar -xzf "$WORK/kinema.tar.gz" -C "$WORK/x" || die "that is not a Kinema server tarball. Nothing was changed."
src="$(find "$WORK/x" -mindepth 2 -maxdepth 2 -name kinema -type f | head -n 1)"
[ -n "$src" ] || die "that is not a Kinema server tarball. Nothing was changed."

# The user the service runs as: whoever ran sudo, so their media is readable.
RUN_AS="${SUDO_USER:-}"
if [ -z "$RUN_AS" ] || [ "$RUN_AS" = root ]; then
  RUN_AS=kinema
  id kinema >/dev/null 2>&1 || useradd --system --home-dir "$DATA" --shell /usr/sbin/nologin kinema 2>/dev/null ||
    adduser -S -h "$DATA" -s /sbin/nologin kinema || die "cannot create the kinema user."
fi

port_taken() {
  if command -v ss >/dev/null 2>&1; then
    ss -Htln "sport = :$1" 2>/dev/null | grep -q .
  elif command -v netstat >/dev/null 2>&1; then
    netstat -tln 2>/dev/null | awk '{print $4}' | grep -qE "[:.]$1\$"
  else
    return 1 # cannot tell; the server itself moves on to a free port.
  fi
}

mkdir -p "$DATA"
# First install only: an update keeps the port and every other setting.
if [ ! -f "$DATA/kinema.toml" ]; then
  chosen="$PORT"
  while port_taken "$chosen" && [ "$chosen" -lt $((PORT + 100)) ]; do chosen=$((chosen + 1)); done
  [ "$chosen" = "$PORT" ] || echo "Port $PORT is in use; Kinema will use $chosen."
  printf '# Kinema server settings. Restart after a change: sudo systemctl restart kinema\n[server]\nport = %s\n' "$chosen" >"$DATA/kinema.toml"
fi
chown -R "$RUN_AS" "$DATA"

# Swap the whole folder, so an update never leaves old and new files mixed.
rm -rf "$APP.new"
mv "$(dirname "$src")" "$APP.new"
chmod 755 "$APP.new" "$APP.new/kinema" "$APP.new/ffmpeg" "$APP.new/ffprobe"
rm -rf "$APP.old"
[ ! -d "$APP" ] || mv "$APP" "$APP.old"
mv "$APP.new" "$APP"
rm -rf "$APP.old"

if [ "$HAS_SYSTEMD" -eq 0 ]; then
  echo "Kinema $label is installed in $APP. This system has no systemd, so start it yourself:"
  echo "  cd $DATA && sudo -u $RUN_AS $APP/kinema"
  exit 0
fi

cat >"$UNIT" <<UNIT
[Unit]
Description=Kinema media server
After=network-online.target
Wants=network-online.target

[Service]
User=$RUN_AS
WorkingDirectory=$DATA
ExecStart=$APP/kinema
Environment=NO_COLOR=1
Restart=on-failure
RestartSec=5
# Scans and transcodes must not slow the rest of the machine.
Nice=5

[Install]
WantedBy=multi-user.target
UNIT
systemctl daemon-reload
systemctl enable kinema >/dev/null 2>&1
systemctl restart kinema

# The server picks the next free port by itself if its own is taken by then, so ask it.
port=""
i=0
while [ -z "$port" ] && [ $i -lt 20 ]; do
  sleep 1
  i=$((i + 1))
  port="$(journalctl _SYSTEMD_INVOCATION_ID="$(systemctl show -p InvocationID --value kinema)" -o cat 2>/dev/null | sed -n 's/.*listening on https\{0,1\}:\/\/[^ ]*:\([0-9][0-9]*\).*/\1/p' | tail -n 1)"
done
[ -n "$port" ] || port="$(sed -n 's/^port = //p' "$DATA/kinema.toml")"
ip="$(hostname -I 2>/dev/null | awk '{print $1}')"
echo "Kinema $label is running as $RUN_AS. Open https://${ip:-localhost}:$port in a browser to set it up."
echo "Logs: journalctl -u kinema -f    Settings: $DATA/kinema.toml"
