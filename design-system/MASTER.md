# Kinema Design System — MASTER

Single source of truth for the web and client UI. Structure follows Domus (`../domus`): Material 3 tonal tokens as RGB triplets, light and dark schemes, the same component kit (Button, Field, Card, Modal, Badge, EmptyState, Notice).

Kinema's seed is **Ember** `#E8553A`: a warm cinema red-orange. It keeps artwork as the hero, reads well on dark surfaces, and stays clearly apart from other media servers' brand colours (no amber/gold, no Netflix red).

Implementation:
- Tokens in `apps/web/src/index.css` (`:root` dark, `[data-theme='light']`, and `prefers-color-scheme`)
- Tailwind mapping in `apps/web/tailwind.config.js`
- Theme choice in `kinema.theme` / `kinema.dev.theme` (`apps/web/src/lib/theme.ts`)

## Principles

1. **Content first**: posters, backdrops and video carry the colour. Chrome is warm-neutral and quiet.
2. **One accent**: `primary` marks play buttons, progress, the active nav item and the seek bar. Nothing else competes.
3. **Player is always dark**: black surface, white controls, primary seek fill, regardless of theme.
4. **Readable over art**: hero text sits on a `bg` gradient; subtitles use `.text-over-media` shadow plus a translucent plate.

## Color tokens

| Token | Dark | Light | Use |
|---|---|---|---|
| `bg` | `#1A110F` | `#FFF8F6` | Page background |
| `panel` | `#231917` | `#FFFFFF` | Sidebar, cards, dialogs |
| `raised` | `#322825` | `#FCEAE6` | Inputs, hover, skeletons |
| `border` | `#534340` | `#D8C2BE` | Hairlines |
| `border-strong` | `#A08C88` | `#85736F` | Focused inputs |
| `fg` | `#F1DFDB` | `#231917` | Primary text |
| `muted` | `#D8C2BE` | `#534340` | Secondary text |
| `subtle` | `#A08C88` | `#85736F` | Captions, placeholders |
| `primary` | `#FFB4A3` | `#A23F26` | Play, progress, active |
| `on-primary` | `#5F1505` | `#FFFFFF` | Text on primary |
| `primary-container` | `#802A15` | `#FFDAD2` | Active nav, selected cards |
| `on-primary-container` | `#FFDAD2` | `#3D0700` | Text on container |
| `info` (tertiary) | `#DDC48C` | `#6D5E2F` | Ratings, folders, DEV pill |
| `warn` | `#F59E0B` | `#B45309` | Warnings |
| `danger` | `#FFB4AB` | `#BA1A1A` | Errors, destructive |

Items without artwork get a deterministic two-stop gradient from the title hash, so a grid of unmatched files still reads as distinct cards.

## Typography

Inter Variable. Scale (Tailwind `text-*`): `caption` 12/16, `label` 13/18, `body` 14/20, `title` 16/22, `page` 20/28, `hero` 32/38.

## Layout

- Sidebar 240 px, sticky; becomes a drawer under `md`.
- Top bar 64 px, translucent `.chrome` blur, search + theme + user.
- Poster cards 2:3, radius 10 px, hover scale 1.02 with a play disc; progress bar at the bottom; watched check top-right.
- Wide cards 16:9 for Continue Watching, episodes and "Other videos" libraries.
- Radius: 6 px controls, 10 px cards. Motion: 120–200 ms, `cubic-bezier(0.2, 0, 0, 1)`, reduced-motion respected.
