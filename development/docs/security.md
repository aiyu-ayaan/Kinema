# Security

## Threats and mitigations

| Threat | Mitigation |
|---|---|
| Path traversal via stream or image endpoints | Clients never send paths, only IDs. Server resolves paths from the DB and checks they sit under a library root after canonicalization. |
| Malicious media files exploiting ffmpeg | Run ffmpeg/ffprobe as child processes with restricted env, no network (`-protocol_whitelist file,pipe`), time and memory limits, and an optional seccomp or bubblewrap sandbox. |
| Credential theft | argon2id hashes, hashed session tokens, rate-limited login, lockout after repeated failures. |
| Share link guessing | 128-bit random tokens, hashed in DB, per-IP rate limit, expiry and use caps. |
| Cross-profile data leaks | Every query on watch state, playlists and library access filters by authenticated user. |
| CSRF | `SameSite=Lax` cookies plus a custom header required on state-changing requests. |
| XSS via metadata | Metadata is data, never HTML. The UI escapes all fields. CSP header with no inline scripts. |
| SSRF through artwork fetch | Image downloader only accepts `https://image.tmdb.org` and the configured hosts. |
| Resource exhaustion | Transcode session cap, request body limits, bounded job queue, connection limits. |
| Plaintext on the internet | Document TLS setup, refuse non-LAN access over plain HTTP unless opted in. |

## Defaults

- Binds to `127.0.0.1` until the admin enables LAN or remote access.
- Remote access needs TLS.
- Telemetry: none.

## Logging

Structured logs with `tracing`. Never log tokens, passwords or full share URLs.
