---
id: authentication-api
title: Authentication API
sidebar_position: 2
---

# Authentication API

Manage server initialization, profile logins, sessions, and multi-user access control.

---

### Check Setup Status

Check if the server is in first-run setup mode:

```http
GET /api/auth/status
```

#### Response (`200 OK`)
```json
{
  "needs_setup": false
}
```

---

### First-Run Admin Setup

Initializes the administrator profile on a new server. Only available if `needs_setup` is `true`.

```http
POST /api/auth/setup
Content-Type: application/json

{
  "name": "admin",
  "password": "SuperSecretPassword123"
}
```

#### Response (`200 OK`)
Sets `kinema_session` cookie and returns:
```json
{
  "token": "4f9d8a1c...e2b",
  "user": {
    "id": 1,
    "name": "admin",
    "is_admin": true,
    "has_pin": false,
    "avatar": null
  }
}
```

---

### Profile Sign-In

Authenticates a user via password or 4-digit PIN. Enforces a 60-second lockout after 5 consecutive failures.

```http
POST /api/auth/login
Content-Type: application/json

{
  "user_id": 1,
  "password": "SuperSecretPassword123"
}
```

*For PIN authentication:*
```json
{
  "user_id": 2,
  "pin": "1234"
}
```

#### Response (`200 OK`)
```json
{
  "token": "8a3e1b0c...9f1",
  "user": {
    "id": 1,
    "name": "admin",
    "is_admin": true,
    "has_pin": false,
    "avatar": null
  }
}
```

---

### Get Profiles List

Returns all available user profiles for the profile switcher screen:

```http
GET /api/auth/profiles
```

#### Response (`200 OK`)
```json
[
  {
    "id": 1,
    "name": "Admin",
    "is_admin": true,
    "has_pin": false,
    "has_password": true,
    "avatar": null
  },
  {
    "id": 2,
    "name": "Kids",
    "is_admin": false,
    "has_pin": true,
    "has_password": false,
    "avatar": "avatar_kids.png"
  }
]
```

---

### Sign Out

Invalidates the active session token in the database and clears the session cookie:

```http
POST /api/auth/logout
Authorization: Bearer <token>
```

#### Response (`200 OK`)
```json
{
  "ok": true
}
```
