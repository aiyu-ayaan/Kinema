---
id: docker-guide
title: Docker & Reverse Proxy Guide
sidebar_position: 1
---

# Docker & Reverse Proxy Deployment

Kinema is packaged as an optimized, multi-architecture Docker container (`ghcr.io/aiyu-ayaan/kinema:latest`) supporting `linux/amd64` and `linux/arm64` (e.g. Raspberry Pi 4/5, Apple Silicon).

---

## Production Docker Compose

```yaml
version: '3.8'

services:
  kinema:
    image: ghcr.io/aiyu-ayaan/kinema:latest
    container_name: kinema
    network_mode: host # Enables UDP port 48555 LAN discovery
    restart: unless-stopped
    environment:
      - KINEMA_DATA_DIR=/app/data
      - KINEMA_PUBLIC_URL=https://media.example.com
      - TZ=UTC
    volumes:
      - ./data:/app/data
      - /mnt/storage/movies:/media/movies:ro
      - /mnt/storage/tv:/media/tv:ro
    devices:
      - /dev/dri:/dev/dri # Optional: Hardware acceleration (VAAPI / DRM)
```

---

## Reverse Proxy Setup

If you prefer running Kinema behind a reverse proxy (Caddy or Nginx) rather than exposing port 3000 directly:

### Caddyfile (Recommended)

Caddy automatically provisions free Let's Encrypt / ZeroSSL certificates:

```caddy
media.example.com {
    reverse_proxy 127.0.0.1:3000 {
        header_up X-Forwarded-Proto {scheme}
        header_up X-Real-IP {remote_host}
    }
}
```

### Nginx Configuration

Ensure buffer sizes and timeout limits are configured for streaming long video files:

```nginx
server {
    server_name media.example.com;
    listen 443 ssl http2;

    ssl_certificate /etc/letsencrypt/live/media.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/media.example.com/privkey.pem;

    client_max_body_size 100M;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Disable buffering for progressive fMP4 chunks
        proxy_buffering off;
        proxy_read_timeout 3600s;
        proxy_send_timeout 3600s;
    }
}
```
