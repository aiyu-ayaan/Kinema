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
| Plaintext on the network | HTTPS by default (backend `tls.rs`): a self-signed certificate is made in `data/tls` on first start, or `[tls] cert_path`/`key_path` use a real one. The session cookie is `Secure`. `[tls] enabled = false` only behind a TLS-terminating proxy. |
| Fake server on the LAN | Apps show the certificate's SHA-256 fingerprint on first connect and pin it after the owner approves; the dashboard (Settings > Network) shows the same code. A certificate signed by a public authority (a reverse proxy on the public URL) is trusted as usual. |

## Defaults

- Binds to `127.0.0.1` until the admin enables LAN or remote access.
- HTTPS on, self-signed. Remote access goes through the admin-set public URL, normally a reverse proxy with a real certificate.
- Telemetry: none.

## Logging

Structured logs with `tracing`. Never log tokens, passwords or full share URLs.
