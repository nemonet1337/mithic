# Mithic API Documentation

## Overview

Mithic API follows the Mastodon API specification for compatibility with existing clients.

Base URL: `https://your-instance.com/api/v1/`

## Authentication

### Sign In
```
POST /api/v1/signin
Content-Type: application/json

{
  "username": "string",
  "password": "string"
}
```

Response:
```json
{
  "token": "string",
  "actor": {
    "id": "string",
    "username": "string",
    "name": "string",
    "avatar_url": "string"
  }
}
```

### Sign Up
```
POST /api/v1/signup
Content-Type: application/json

{
  "username": "string",
  "password": "string",
  "name": "string",
  "email": "string"
}
```

## Accounts

### Get Current Account
```
GET /api/v1/accounts/verify_credentials
Authorization: Bearer {token}
```

### Get Account
```
GET /api/v1/accounts/{id}
Authorization: Bearer {token}
```

### Follow Account
```
POST /api/v1/accounts/{id}/follow
Authorization: Bearer {token}
```

### Unfollow Account
```
POST /api/v1/accounts/{id}/unfollow
Authorization: Bearer {token}
```

## Timelines

### Home Timeline
```
GET /api/v1/timelines/home?limit={limit}
Authorization: Bearer {token}
```

### Public Timeline
```
GET /api/v1/timelines/public?limit={limit}
```

## Statuses

### Create Status
```
POST /api/v1/statuses
Authorization: Bearer {token}
Content-Type: application/json

{
  "status": "string",
  "in_reply_to_id": "string",
  "sensitive": false,
  "spoiler_text": "string",
  "visibility": "public"
}
```

### Get Status
```
GET /api/v1/statuses/{id}
```

### Delete Status
```
DELETE /api/v1/statuses/{id}
Authorization: Bearer {token}
```

## Error Responses

All errors follow this format:

```json
{
  "error": true,
  "message": "Error description"
}
```

Status codes:
- 400: Bad Request
- 401: Unauthorized
- 403: Forbidden
- 404: Not Found
- 500: Internal Server Error
