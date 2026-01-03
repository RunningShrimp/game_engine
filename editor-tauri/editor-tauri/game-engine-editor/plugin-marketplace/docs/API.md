# Plugin Marketplace API Documentation

## Base URL

```
Production: https://plugins.gameengine.com
Development: http://localhost:8080
```

## Authentication

Most endpoints require a JWT token in the Authorization header:

```
Authorization: Bearer <token>
```

### Get Auth Token

```http
POST /api/v1/users/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}
```

Response:

```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "username": "username"
    }
  }
}
```

## Plugins

### Search Plugins

```http
GET /api/v1/plugins/search?q=terrain&category=rendering&sort_by=downloads&page=1&limit=20
```

Query Parameters:
- `q` (string): Search query
- `category` (string): Filter by category
- `tags` (string): Comma-separated tags
- `pricing` (string): Filter by pricing type (free, paid, freemium, subscription)
- `min_rating` (number): Minimum rating (0-5)
- `sort_by` (string): Sort by (relevance, downloads, rating, updated, name)
- `page` (number): Page number (default: 1)
- `limit` (number): Results per page (default: 20, max: 100)

Response:

```json
{
  "success": true,
  "data": {
    "plugins": [
      {
        "id": "uuid",
        "name": "Terrain Generator",
        "slug": "terrain-generator",
        "description": "Procedural terrain generation plugin",
        "author": {
          "id": "uuid",
          "name": "John Doe",
          "avatar": "https://..."
        },
        "version": "1.2.0",
        "latest_version": "1.2.0",
        "categories": ["rendering", "tools"],
        "tags": ["terrain", "procedural", "generation"],
        "license": "MIT",
        "homepage": "https://...",
        "repository": "https://github.com/...",
        "screenshots": ["https://...", "https://..."],
        "rating": {
          "average": 4.5,
          "count": 128,
          "distribution": { "5": 80, "4": 30, "3": 15, "2": 2, "1": 1 }
        },
        "downloads": 15000,
        "pricing": {
          "pricing_type": "paid",
          "price": 29.99,
          "currency": "USD",
          "trial_available": true
        },
        "compatibility": {
          "engine_version_min": "1.0.0",
          "platforms": ["windows", "macos", "linux"]
        },
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-15T00:00:00Z"
      }
    ],
    "total": 150,
    "page": 1,
    "limit": 20
  }
}
```

### Get Plugin Details

```http
GET /api/v1/plugins/{plugin_id}
```

Response: Same as search result plugin object

### Get Plugin Versions

```http
GET /api/v1/plugins/{plugin_id}/versions
```

Response:

```json
{
  "success": true,
  "data": {
    "versions": [
      {
        "id": "uuid",
        "version": "1.2.0",
        "changelog": "Added new features...\n\n- Feature 1\n- Feature 2",
        "file_size": 1024000,
        "status": "approved",
        "published_at": "2024-01-15T00:00:00Z"
      }
    ]
  }
}
```

### Get Download URL

```http
GET /api/v1/plugins/{plugin_id}/download?version=1.2.0&platform=macos&engine_version=1.5.0
```

Query Parameters:
- `version` (string): Specific version (optional, defaults to latest)
- `platform` (string): Platform (windows, macos, linux, etc.)
- `engine_version` (string): Engine version

Response:

```json
{
  "success": true,
  "data": {
    "url": "https://s3.amazonaws.com/bucket/plugin.tar.gz",
    "sha256": "abc123...",
    "expires_at": "2024-01-15T01:00:00Z"
  }
}
```

### Create Plugin

```http
POST /api/v1/plugins
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "My Plugin",
  "description": "Plugin description",
  "categories": ["rendering"],
  "tags": ["graphics", "3d"],
  "license": "MIT",
  "repository": "https://github.com/...",
  "manifest": { ... }
}
```

### Update Plugin

```http
PUT /api/v1/plugins/{plugin_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "description": "Updated description",
  "categories": ["rendering", "tools"]
}
```

### Delete Plugin

```http
DELETE /api/v1/plugins/{plugin_id}
Authorization: Bearer <token>
```

## Reviews

### Get Plugin Reviews

```http
GET /api/v1/plugins/{plugin_id}/reviews?page=1&limit=10&sort_by=helpful
```

Query Parameters:
- `page` (number): Page number
- `limit` (number): Results per page
- `sort_by` (string): Sort by (helpful, recent, rating_high, rating_low)

Response:

```json
{
  "success": true,
  "data": {
    "reviews": [
      {
        "id": "uuid",
        "plugin_id": "uuid",
        "user": {
          "id": "uuid",
          "name": "Jane Doe",
          "avatar": "https://..."
        },
        "rating": 5,
        "title": "Excellent plugin!",
        "content": "This plugin saved me hours of work...",
        "helpful_count": 42,
        "created_at": "2024-01-10T00:00:00Z"
      }
    ],
    "total": 128,
    "page": 1,
    "limit": 10
  }
}
```

### Create Review

```http
POST /api/v1/plugins/{plugin_id}/reviews
Authorization: Bearer <token>
Content-Type: application/json

{
  "rating": 5,
  "title": "Great plugin",
  "content": "Detailed review..."
}
```

### Update Review

```http
PUT /api/v1/reviews/{review_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "rating": 4,
  "title": "Updated title",
  "content": "Updated content"
}
```

### Delete Review

```http
DELETE /api/v1/reviews/{review_id}
Authorization: Bearer <token>
```

### Vote Review Helpful

```http
POST /api/v1/reviews/{review_id}/vote
Authorization: Bearer <token>
```

## Users

### Register

```http
POST /api/v1/users/register
Content-Type: application/json

{
  "email": "user@example.com",
  "username": "username",
  "password": "password123"
}
```

Response:

```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "email": "user@example.com",
    "username": "username",
    "role": "user"
  }
}
```

### Login

```http
POST /api/v1/users/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}
```

Response:

```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "username": "username",
      "avatar": "https://...",
      "role": "user"
    }
  }
}
```

### Get Current User

```http
GET /api/v1/users/me
Authorization: Bearer <token>
```

### Update Profile

```http
PUT /api/v1/users/me
Authorization: Bearer <token>
Content-Type: application/json

{
  "bio": "Game developer and plugin creator",
  "website": "https://mywebsite.com"
}
```

## Categories

### List Categories

```http
GET /api/v1/categories
```

Response:

```json
{
  "success": true,
  "data": {
    "categories": [
      {
        "id": "uuid",
        "name": "Rendering",
        "slug": "rendering",
        "description": "Plugins for graphics and rendering",
        "icon": "https://...",
        "plugin_count": 45,
        "display_order": 1
      }
    ]
  }
}
```

## Statistics

### Get Marketplace Stats

```http
GET /api/v1/stats
```

Response:

```json
{
  "success": true,
  "data": {
    "total_plugins": 500,
    "total_downloads": 5000000,
    "active_developers": 150,
    "categories": {
      "rendering": 45,
      "physics": 30,
      "ai": 25
    }
  }
}
```

### Get Plugin Statistics

```http
GET /api/v1/plugins/{plugin_id}/stats?days=30
```

Response:

```json
{
  "success": true,
  "data": {
    "downloads_last_30_days": 1500,
    "views_last_30_days": 5000,
    "downloads_by_version": {
      "1.2.0": 800,
      "1.1.0": 500,
      "1.0.0": 200
    },
    "downloads_by_platform": {
      "windows": 700,
      "macos": 500,
      "linux": 300
    },
    "daily_downloads": [
      { "date": "2024-01-01", "downloads": 50 },
      { "date": "2024-01-02", "downloads": 45 }
    ]
  }
}
```

## Analytics

### Track Download

```http
POST /api/v1/analytics/download
Content-Type: application/json

{
  "plugin_id": "uuid",
  "version": "1.2.0",
  "platform": "macos",
  "engine_version": "1.5.0"
}
```

### Track View

```http
POST /api/v1/analytics/view
Content-Type: application/json

{
  "plugin_id": "uuid",
  "session_id": "optional-session-id",
  "referrer": "https://google.com"
}
```

## Error Responses

All endpoints may return error responses:

```json
{
  "success": false,
  "error": "Error message describing what went wrong",
  "message": "Additional context or suggestion"
}
```

HTTP Status Codes:
- 200: Success
- 201: Created
- 400: Bad Request
- 401: Unauthorized
- 403: Forbidden
- 404: Not Found
- 429: Rate Limit Exceeded
- 500: Internal Server Error

## Rate Limiting

API requests are rate limited:
- Unauthenticated: 100 requests/hour
- Authenticated: 1000 requests/hour

Rate limit headers are included in responses:

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1640995200
```

## Pagination

List endpoints support pagination:

Query Parameters:
- `page`: Page number (default: 1)
- `limit`: Items per page (default: 20, max: 100)

Response includes pagination info:

```json
{
  "success": true,
  "data": { ... },
  "pagination": {
    "total": 500,
    "page": 1,
    "limit": 20,
    "pages": 25
  }
}
```

## Webhooks

Webhooks can be configured to receive notifications about events.

### Configure Webhook

```http
POST /api/v1/webhooks
Authorization: Bearer <token>
Content-Type: application/json

{
  "url": "https://your-server.com/webhook",
  "events": ["plugin.installed", "plugin.updated", "review.created"]
}
```

### Webhook Payload

```json
{
  "event": "plugin.installed",
  "timestamp": "2024-01-15T00:00:00Z",
  "data": {
    "plugin_id": "uuid",
    "version": "1.2.0",
    "user_id": "uuid"
  }
}
```
