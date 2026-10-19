# Kinema Design System — MASTER

Single source of truth for the web and client UI.
Kinema adopts a cinematic, content-first dark interface with gold/amber media accents (`#E5A00D`) reminiscent of premium cinema and streaming experiences.

Implementation:
- Tokens in `apps/web/src/index.css`
- Tailwind configuration in `apps/web/tailwind.config.js`
- Design tokens documented here

## Principles

1. **Content First**: Artwork, backdrops, and posters are the heroes. The UI chrome is dark and unobtrusive.
2. **Cinematic Accents**: Warm amber/gold primary accent for progress bars, active selections, and interactive states.
3. **Responsive Media Grids**: 2:3 aspect ratio poster cards with lazy loading, subtle hover zoom, and watch progress bars.
4. **Fluid Transitions**: Fast, predictable transitions for menus, modals, and playback controls.

## Color Tokens

| Token | Dark (Default) | Use |
|---|---|---|
| `bg` | `#0F1115` | Main application background |
| `panel` | `#16191F` | Sidebars, modals, cards |
| `raised` | `#1E222A` | Search input, hover states, elevated controls |
| `border` | `#2A2F3A` | Hairline dividers and borders |
| `border-strong` | `#3E4554` | Focused inputs, active borders |
| `fg` | `#F3F4F6` | Primary high-contrast text |
| `muted` | `#9CA3AF` | Secondary labels, metadata (year, runtime) |
| `subtle` | `#6B7280` | Overlines, inactive icons, placeholders |
| `primary` | `#E5A00D` | Play buttons, active tabs, progress bars |
| `on-primary` | `#000000` | Text/icons on primary button |
| `primary-container` | `#30250C` | Subtle primary highlights |
| `on-primary-container` | `#FDE68A` | Text on primary containers |
| `warn` | `#EAB308` | Warnings and notices |
| `danger` | `#EF4444` | Errors, destructive actions |
| `info` | `#3B82F6` | Informational badges |

## Typography

- Font family: `Inter Variable`, fallback `system-ui, -apple-system, sans-serif`.
- Sizes:
  - Caption: `12px` (line height 16px)
  - Label: `13px` (line height 18px)
  - Body: `14px` (line height 20px)
  - Title: `16px` (line height 22px)
  - Section Header: `20px` (line height 28px)
  - Hero / Heading: `28px` (line height 34px)

## Components & Layout

- **Sidebar**: 240px wide, sticky left navigation.
- **Top Bar**: 64px high, translucent blur with search and profile controls.
- **Poster Cards**: 2:3 aspect ratio, rounded corners (8px), watched checkmarks, and bottom progress indicator.
