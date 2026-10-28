# Security

## Threats and mitigations

| Threat | Mitigation |
|---|---|
| Path traversal via stream or image endpoints | Clients never send paths, only IDs. Server resolves paths from the DB and checks they sit under a library root after canonicalization. |
| Malicious media files exploiting ffmpeg | Run ffmpeg/ffprobe as child processes with restricted env, no network (`-protocol_whitelist file,pipe`), time and memory limits, and an optional seccomp or bubblewrap sandbox. |
| Credential theft | argon2id hashes, hashed session tokens, rate-limited login, lockout after repeated failures. |
| Share link guessing | Selector (64 bit) + verifier (256 bit) from the OS RNG; only SHA-256 of the verifier is stored and compared in constant time; every failure is the same 404; links expire (≤ 90 days) and are revocable, and a running share stream stops within 10 s of revocation. Not yet: per-IP rate limit, use caps. |
| Share link forwarded or hammered | Optional viewer limit per link (distinct client addresses, stored as a hash), 300 requests a minute per address, one re-encode at a time per link. The address is the socket peer unless it is a configured trusted proxy (`net::client_ip`); behind an untrusted proxy all viewers look like one address. |
| Share link reaching beyond its title | Share routes are separate from the session API, accept no session, and check on every request that the item / file / subtitle / image sits under the target and that the creator's own library and rating policy still allows it. |
| Unlocked profile opened from the internet | One-tap profiles and `lan_unlock` work only from LAN addresses (`net.rs`); sessions opened that way are `lan_only` and refused from elsewhere. Forwarded headers count only from `[network] trusted_proxies`; from anyone else they make the request remote, so a same-host reverse proxy fails closed. CGNAT and global IPv6 are not LAN. |
| Child profile reaching restricted titles | Content limits are enforced server side on every listing, detail, progress, plan, stream, subtitle and thumbnail path (`access.rs` `Policy`), answering 404; a profile cannot change its own limit, also not by importing a file. |
| Profile export leaking credentials | Exports leave out PIN / password hashes unless an admin asks (`include_lock=true`); imports of hashes need an explicit admin flag and a well-formed argon2 hash. |
| Password shared to extra devices | A non-admin profile gets one web/desktop and one mobile device free; any other device is `pending` until the admin approves it (`routes/auth.rs` `admit`). Admins and LAN-only sessions are exempt. Max 5 waiting requests per profile (429), so one password cannot flood the admin's list or notifications; a retry by a known device raises no new notification. |
| Poll token theft or reuse | The waiting client polls with a 256-bit random secret; only its SHA-256 is stored (`devices.poll_hash`), it is cleared when exchanged for a session, so it works once. Unknown, denied or used tokens all answer 404. Denying a request deletes the row. |
| Removed device keeps access | Sessions reference the device with `ON DELETE CASCADE`; deleting the device ends them at once. |
| Notification secrets leaking | Tokens, the SMTP password and Discord / Slack / webhook addresses are stored server side only. `GET /settings/notifications` returns `••••••••` in their place; a save with the mask keeps the old value, so the page never holds the secret. Endpoints are admin only. Send errors are logged without the request URL (Telegram puts the bot token in it; Discord / Slack webhook addresses are secrets too, so they are masked as well). Outbound targets are set by the admin only. |
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
