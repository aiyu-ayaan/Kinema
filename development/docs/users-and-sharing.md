# Users, Profiles and Sharing

## Accounts

- First run creates an admin through a setup screen.
- Admin creates profiles. A profile is a user: name, avatar, optional password, optional 4-digit PIN.
- Passwords hashed with argon2id. PINs are for switching profiles on a trusted LAN device, not for remote login.

## Profile picker

`GET /api/users` returns names and avatars only. Selecting a profile with a PIN or password calls `/auth/login`. Profiles without either are allowed only from LAN addresses (configurable).

## Per-profile state

Watch progress, watched flags, "Continue Watching", playlists, language and subtitle preferences. Nothing leaks across profiles.

## Permissions

| Role | Can |
|---|---|
| Admin | Everything: libraries, users, server settings, shares for any content |
| User | Browse allowed libraries, play, own playlists, create share links for items they can access |
| Restricted | Like User but with a max content rating and no sharing |

Library access is a whitelist in `user_library_access`. Content rating limits filter queries server side.

## Sessions

Random 256-bit token, stored as a hash. Default lifetime 30 days sliding. Cookie is `HttpOnly`, `Secure`, `SameSite=Lax`. Revocable per session.

## Sharing

Three modes:

1. Invite a user: admin creates a profile for them and grants libraries.
2. Share link: `POST /api/shares` creates a token for an item, playlist or library. URL form `/s/{token}`. Options: expiry, max uses, allow download.
3. Profile export: a profile's playlists and watch history can be exported as JSON and imported on another Kinema server.

Share link rules:

- Token is 128+ random bits, only its hash stored.
- Grants read-only playback of the target, nothing else. No library browsing outside the target.
- Playback goes through the same stream endpoints with a scoped token. Transcode slots are limited per link.
- Revocation takes effect immediately and kills active sessions from that link.
- Rate limited per IP.

## Remote access

Recommended: reverse proxy (Caddy) with TLS. Built-in option: ACME via `rustls-acme`. UPnP port mapping is off by default.
