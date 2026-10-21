---
id: library-management-api
title: Library Management API
sidebar_position: 4
---

# Library Management API

Endpoints for creating media libraries, browsing filesystem directories, initiating background folder scans, and managing items.

---

### List Libraries

Returns all libraries accessible to the authenticated profile:

```http
GET /api/libraries
Authorization: Bearer <token>
```

#### Response (`200 OK`)
```json
[
  {
    "id": 1,
    "name": "Movies",
    "kind": "movie",
    "item_count": 482,
    "roots": ["/media/movies"]
  },
  {
    "id": 2,
    "name": "TV Shows",
    "kind": "show",
    "item_count": 34,
    "roots": ["/media/tv"]
  }
]
```

---

### Create Library (Admin)

Creates a new library associated with one or more host filesystem roots:

```http
POST /api/libraries
Authorization: Bearer <ADMIN_TOKEN>
Content-Type: application/json

{
  "name": "Anime",
  "kind": "show",
  "roots": ["/media/anime"],
  "language": "ja"
}
```

#### Response (`200 OK`)
```json
{
  "id": 3,
  "name": "Anime",
  "kind": "show"
}
```

---

### Trigger Library Rescan (Admin)

Dispatches an asynchronous background scan job to index new or modified files:

```http
POST /api/libraries/:id/scan
Authorization: Bearer <ADMIN_TOKEN>
```

#### Response (`200 OK`)
```json
{
  "status": "scan_queued"
}
```

---

### Interactive Server Directory Browser (Admin)

Enables administrative users to navigate the host filesystem to select media folders:

```http
GET /api/fs?path=/media
Authorization: Bearer <ADMIN_TOKEN>
```

#### Response (`200 OK`)
```json
{
  "current": "/media",
  "parent": "/",
  "directories": ["movies", "tv", "music", "photos"]
}
```

---

### Update Playback Progress

Updates the current user's watch progress for an item:

```http
PUT /api/items/:id/progress
Authorization: Bearer <token>
Content-Type: application/json

{
  "position_s": 842.5,
  "watched": false
}
```

#### Response (`200 OK`)
```json
{
  "ok": true
}
```
