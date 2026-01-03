# Plugin Marketplace System

A complete plugin marketplace solution for the game engine, including a CLI tool, REST API backend, and web frontend.

## Architecture Overview

```
┌─────────────────┐      ┌──────────────────┐      ┌─────────────────┐
│   CLI Tool      │──────│   Backend API    │──────│   PostgreSQL    │
│   (plugin-cli)  │      │   (Actix/Rust)   │      │   Database      │
└─────────────────┘      └──────────────────┘      └─────────────────┘
                                  │
                                  │
                                  ▼
                         ┌──────────────────┐      ┌─────────────────┐
                         │   Frontend Web   │──────│      S3/CDN     │
                         │   (Next.js)      │      │   File Storage  │
                         └──────────────────┘      └─────────────────┘
```

## Features

### For Plugin Users
- Search and browse plugins
- Filter by category, tags, pricing, rating
- Install, update, and uninstall plugins via CLI
- View ratings, reviews, and screenshots
- Track dependencies and compatibility

### For Plugin Developers
- Publish plugins to the marketplace
- Manage versions and changelogs
- View download statistics and analytics
- Receive user feedback and reviews
- Monetize plugins (paid/subscription)

## Directory Structure

```
plugin-marketplace/
├── backend/                 # Rust backend API
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── models/         # Data models
│   │   ├── routes/         # API routes
│   │   ├── handlers/       # Business logic
│   │   ├── services/       # External services
│   │   └── middleware/     # Auth, logging, etc.
│   ├── migrations/         # Database migrations
│   └── Cargo.toml
│
├── frontend/               # Next.js frontend
│   ├── src/
│   │   ├── components/     # React components
│   │   ├── pages/         # Page components
│   │   ├── services/      # API clients
│   │   ├── hooks/         # Custom hooks
│   │   ├── types/         # TypeScript types
│   │   └── utils/         # Utilities
│   ├── public/            # Static assets
│   └── package.json
│
├── cli/                    # CLI tool (in game_engine/src/plugin/)
│   ├── mod.rs
│   ├── cli.rs
│   ├── marketplace.rs
│   ├── registry.rs
│   └── installer.rs
│
└── docs/                  # Documentation
    ├── API.md
    ├── DEPLOYMENT.md
    └── DEVELOPMENT.md
```

## Quick Start

### Prerequisites

- Rust 1.70+ (for backend and CLI)
- Node.js 20+ (for frontend)
- PostgreSQL 14+
- AWS Account (for S3 storage)

### 1. Database Setup

```bash
# Install PostgreSQL
brew install postgresql  # macOS
sudo apt install postgresql  # Ubuntu

# Create database
createdb plugin_marketplace

# Run migrations
cd backend
sqlx database create
sqlx migrate run
```

### 2. Backend Setup

```bash
cd backend

# Copy environment template
cp .env.example .env

# Edit .env with your configuration
# DATABASE_URL=postgresql://user:password@localhost/plugin_marketplace
# JWT_SECRET=your-secret-key
# S3_BUCKET=your-bucket-name
# AWS_ACCESS_KEY_ID=your-access-key
# AWS_SECRET_ACCESS_KEY=your-secret-key
# AWS_REGION=us-east-1

# Build and run
cargo build --release
cargo run
```

The API will be available at `http://localhost:8080`

### 3. Frontend Setup

```bash
cd frontend

# Install dependencies
npm install

# Copy environment template
cp .env.example .env.local

# Edit .env.local
# NEXT_PUBLIC_API_URL=http://localhost:8080

# Run development server
npm run dev
```

The web interface will be available at `http://localhost:3000`

### 4. CLI Tool Usage

```bash
# Search for plugins
plugin-cli search "terrain"

# Install a plugin
plugin-cli install terrain-generator

# List installed plugins
plugin-cli list --verbose

# Update plugins
plugin-cli update --all

# Check for updates
plugin-cli check-updates

# Uninstall a plugin
plugin-cli uninstall terrain-generator

# Publish a plugin
plugin-cli publish ./my-plugin

# Get plugin information
plugin-cli info terrain-generator

# View marketplace statistics
plugin-cli stats
```

## API Documentation

### Authentication

Most endpoints require authentication. Include the JWT token in the Authorization header:

```
Authorization: Bearer <your-jwt-token>
```

### Endpoints

#### Plugins

- `GET /api/v1/plugins/search` - Search plugins
- `GET /api/v1/plugins/{id}` - Get plugin details
- `GET /api/v1/plugins/{id}/versions` - Get plugin versions
- `GET /api/v1/plugins/{id}/download` - Get download URL
- `POST /api/v1/plugins` - Create plugin (requires auth)
- `PUT /api/v1/plugins/{id}` - Update plugin (requires auth)
- `DELETE /api/v1/plugins/{id}` - Delete plugin (requires auth)

#### Reviews

- `GET /api/v1/plugins/{id}/reviews` - Get plugin reviews
- `POST /api/v1/plugins/{id}/reviews` - Create review (requires auth)
- `PUT /api/v1/reviews/{id}` - Update review (requires auth)
- `DELETE /api/v1/reviews/{id}` - Delete review (requires auth)

#### Users

- `POST /api/v1/users/register` - Register user
- `POST /api/v1/users/login` - Login user
- `GET /api/v1/users/me` - Get current user (requires auth)
- `PUT /api/v1/users/me` - Update profile (requires auth)

#### Analytics

- `GET /api/v1/stats` - Get marketplace statistics
- `GET /api/v1/plugins/{id}/stats` - Get plugin statistics
- `POST /api/v1/analytics/download` - Track download
- `POST /api/v1/analytics/view` - Track view

See [API.md](docs/API.md) for detailed API documentation.

## Deployment

### Backend Deployment

#### Docker Deployment

```bash
# Build Docker image
docker build -t plugin-marketplace-backend .

# Run container
docker run -p 8080:8080 \
  -e DATABASE_URL=postgresql://... \
  -e JWT_SECRET=... \
  -e S3_BUCKET=... \
  plugin-marketplace-backend
```

#### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: plugin-marketplace-backend
spec:
  replicas: 3
  selector:
    matchLabels:
      app: backend
  template:
    metadata:
      labels:
        app: backend
    spec:
      containers:
      - name: backend
        image: plugin-marketplace-backend:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-secrets
              key: url
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: jwt-secret
              key: secret
```

### Frontend Deployment

#### Vercel Deployment

```bash
# Install Vercel CLI
npm i -g vercel

# Deploy
vercel --prod
```

#### Docker Deployment

```dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM node:20-alpine
WORKDIR /app
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./package.json
EXPOSE 3000
CMD ["npm", "start"]
```

### Database Migration in Production

```bash
# Run migrations
sqlx migrate run --database-url $DATABASE_URL

# Or use the CLI
plugin-cli migrate --production
```

## Development

### Backend Development

```bash
cd backend

# Watch mode for development
cargo watch -x run

# Run tests
cargo test

# Run with auto-reload
cargo install cargo-watch
cargo watch -x 'run'
```

### Frontend Development

```bash
cd frontend

# Development server
npm run dev

# Type checking
npm run type-check

# Linting
npm run lint

# Build
npm run build
```

### Adding New Features

1. **Backend**: Add models, routes, and handlers
2. **Frontend**: Add components, pages, and API services
3. **CLI**: Add new commands in `cli.rs`
4. **Database**: Create new migration files

## Testing

### Backend Tests

```bash
cd backend
cargo test
```

### Frontend Tests

```bash
cd frontend
npm test
```

### Integration Tests

```bash
# Run end-to-end tests
npm run test:e2e
```

## Performance Optimization

### Backend

- Database indexing on frequently queried fields
- Materialized views for category counts
- Connection pooling with `sqlx`
- Response compression with actix-compress
- CDN caching for static assets

### Frontend

- Static generation for static pages
- Image optimization with Next.js Image
- Code splitting and lazy loading
- React Query for data caching
- Infinite scroll for pagination

## Security

- JWT-based authentication
- bcrypt password hashing
- CORS configuration
- Rate limiting (to be implemented)
- Input validation with validator
- SQL injection prevention with sqlx
- XSS protection with React

## Monitoring and Logging

### Backend Logging

```rust
use log::{info, warn, error};

info!("Plugin installed: {}", plugin_id);
warn!("Rate limit exceeded for IP: {}", ip);
error!("Database connection failed: {}", error);
```

### Frontend Analytics

```typescript
import { trackEvent } from '@/services/analytics';

trackEvent('plugin_viewed', { plugin_id: plugin.id });
trackEvent('plugin_downloaded', { plugin_id: plugin.id, version: '1.0.0' });
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see LICENSE file for details

## Support

For issues and questions:
- GitHub Issues: [repository-url]/issues
- Documentation: [repository-url]/wiki
- Discord: [invite-link]

## Roadmap

### Phase 1 (Current)
- ✅ Basic CRUD operations
- ✅ Search and filtering
- ✅ CLI tool
- ✅ Web interface

### Phase 2 (Q1 2025)
- [ ] Plugin dependencies management
- [ ] Automatic updates
- [ ] Payment integration (Stripe)
- [ ] Plugin analytics dashboard

### Phase 3 (Q2 2025)
- [ ] Plugin sandboxing
- [ ] WebAssembly plugins
- [ ] Real-time notifications
- [ ] Plugin marketplace mobile app
