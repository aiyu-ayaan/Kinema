# Clean-Room Policy

Kinema is inspired by Plex Media Server's publicly visible behavior. It is not derived from Plex code.

## Allowed

- Public product pages, support articles and API documentation from Plex and others.
- Observing the behavior of a UI you have legitimate access to (layout, flows, feature set).
- Public standards: HLS, DASH, WebVTT, SQLite, TMDB API, mDNS.
- Our own design decisions documented in `development/docs/`.

## Not allowed

- Copying, translating or paraphrasing Plex's decompiled or disassembled code.
- Reusing Plex's proprietary assets, logos, icons, UI images or trademarks. Kinema uses its own name and branding.
- Implementing Plex's private cloud protocols or bypassing Plex account, licensing or Plex Pass checks.

## About the `research/` directory

Plex's license terms restrict reverse engineering. Any Plex package kept under `research/` is for personal reading of publicly observable facts only. Nothing there feeds into the build. If a design question could only be answered by reading Plex's code, answer it independently and record the reasoning in `development/docs/`.

Whether a given reverse-engineering activity is lawful depends on your jurisdiction and the license you accepted. This file is a project policy, not legal advice.

## Third-party content

Kinema displays metadata and artwork from TMDB. Follow TMDB's terms: show attribution ("This product uses the TMDB API but is not endorsed or certified by TMDB") and do not cache beyond their allowed limits.
