---
id: overview
title: API Reference Overview
sidebar_position: 1
---

# API Reference Overview

The Kinema Server exposes a clean, high-performance REST API over HTTP/HTTPS.

---

## Base URL & Authentication

All API endpoints reside under the `/api` prefix:

```
https://<server-ip>:3000/api
```

### Passing Authentication

You can authenticate requests using either:

1. **Bearer Token in HTTP Header**:
   ```http
   Authorization: Bearer <token>
   ```
2. **HttpOnly Cookie**:
   ```http
   Cookie: kinema_session=<token>
   ```

---

## Response Formats & Error Handling

All JSON endpoints return data with standard HTTP status codes:

- `200 OK`: Request succeeded.
- `400 Bad Request`: Invalid parameters or validation failure.
- `401 Unauthorized`: Missing or invalid session token.
- `403 Forbidden`: Account lacks permissions or admin privileges.
- `404 Not Found`: Media item, library, or file not found.
- `429 Too Many Requests`: Account locked out due to brute-force protection (5 failed attempts within 60s).
- `503 Service Unavailable`: Server reached its maximum concurrent transcoding limit (`KINEMA_MAX_TRANSCODES`).

### Standard Error Object

```json
{
  "error": "auth.locked",
  "message": "Too many failed login attempts. Account locked for 60 seconds."
}
```

---

## Endpoints Summary

- [Authentication API](./authentication-api.md): Setup, login, profiles, sessions.
- [Media & Streaming API](./media-streaming-api.md): Playback plan, stream transport, subtitles, artwork.
- [Library Management API](./library-management-api.md): Libraries, items, scan jobs, metadata matching.
- [Server Admin API](./server-admin-api.md): System health, transcoding stats, settings, cache controls.
