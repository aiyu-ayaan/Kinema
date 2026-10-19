#!/bin/bash
# deb post-install for Kinema / Kinema Dev.
APP_DIR=""
BIN_NAME=""
if [ -d "/opt/Kinema Dev" ]; then
    APP_DIR="/opt/Kinema Dev"
    BIN_NAME="kinema-desktop-dev"
elif [ -d "/opt/Kinema" ]; then
    APP_DIR="/opt/Kinema"
    BIN_NAME="kinema-desktop"
fi

if [ -n "$APP_DIR" ]; then
    if type update-alternatives >/dev/null 2>&1; then
        update-alternatives --install "/usr/bin/$BIN_NAME" "$BIN_NAME" "$APP_DIR/$BIN_NAME" 100 || ln -sf "$APP_DIR/$BIN_NAME" "/usr/bin/$BIN_NAME"
    else
        ln -sf "$APP_DIR/$BIN_NAME" "/usr/bin/$BIN_NAME"
    fi
    chmod 4755 "$APP_DIR/chrome-sandbox" || true
fi

if hash update-desktop-database 2>/dev/null; then
    update-desktop-database /usr/share/applications || true
fi
if hash gtk-update-icon-cache 2>/dev/null; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
fi
