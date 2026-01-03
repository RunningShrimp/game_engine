# Plugin Marketplace - Quick Start Guide

Get the plugin marketplace system up and running in 5 minutes!

## Prerequisites

Make sure you have installed:
- Docker and Docker Compose
- Git
- (Optional) Rust 1.70+ for CLI tool

## Option 1: Docker Compose (Recommended)

### 1. Clone and Setup

```bash
# Clone the repository
git clone <repository-url>
cd plugin-marketplace

# Copy environment files
cp backend/.env.example backend/.env
cp frontend/.env.example frontend/.env.local
```

### 2. Start Services

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f
```

Services will be available at:
- **Frontend**: http://localhost:3000
- **Backend API**: http://localhost:8080
- **Database**: localhost:5432
- **pgAdmin**: http://localhost:5050 (admin@admin.com / admin)

### 3. Seed Database (Optional)

```bash
# Add sample data
docker-compose exec backend bash -c "cd /app && ./scripts/seed_db.sh"
```

### 4. Test the System

```bash
# Test API
curl http://localhost:8080/api/v1/health

# Search plugins
curl http://localhost:8080/api/v1/plugins/search?q=terrain

# View in browser
open http://localhost:3000
```

## Option 2: Manual Setup

### 1. Database Setup

```bash
# Start PostgreSQL
docker run -d \
  --name plugin-marketplace-db \
  -e POSTGRES_USER=admin \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=plugin_marketplace \
  -p 5432:5432 \
  postgres:14-alpine

# Wait for DB to be ready
sleep 5

# Run migrations
cd backend
cargo install sqlx-cli
sqlx database create
sqlx migrate run
```

### 2. Backend Setup

```bash
cd backend

# Set environment
export DATABASE_URL="postgresql://admin:password@localhost:5432/plugin_marketplace"
export JWT_SECRET="your-secret-key"
export S3_BUCKET="plugin-marketplace"

# Run server
cargo run
```

Backend will be available at http://localhost:8080

### 3. Frontend Setup

```bash
cd frontend

# Install dependencies
npm install

# Set environment
echo "NEXT_PUBLIC_API_URL=http://localhost:8080" > .env.local

# Run development server
npm run dev
```

Frontend will be available at http://localhost:3000

## Using the CLI Tool

### Installation

```bash
# From game_engine directory
cd game_engine

# Build CLI
cargo build --release

# (Optional) Install globally
cargo install --path .
```

### Basic Commands

```bash
# Search for plugins
plugin-cli search "terrain"

# Install a plugin
plugin-cli install terrain-generator

# List installed plugins
plugin-cli list --verbose

# Update all plugins
plugin-cli update --all

# Check for updates
plugin-cli check-updates

# Get plugin details
plugin-cli info terrain-generator

# View marketplace stats
plugin-cli stats

# Login (for publishing)
plugin-cli login

# Publish a plugin
plugin-cli publish ./my-plugin
```

## Testing the System

### 1. Create a Test Plugin

```bash
mkdir test-plugin
cd test-plugin

# Create plugin.json
cat > plugin.json << 'EOF'
{
  "name": "test-plugin",
  "display_name": "Test Plugin",
  "version": "1.0.0",
  "description": "A simple test plugin",
  "entry_point": "src/lib.rs",
  "permissions": [],
  "resources": [],
  "commands": [],
  "settings": []
}
EOF

# Create source file
mkdir src
cat > src/lib.rs << 'EOF'
use game_engine::plugin::*;

pub struct TestPlugin;

impl Plugin for TestPlugin {
    fn on_load(&mut self) -> Result<(), PluginError> {
        println!("Test plugin loaded!");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        println!("Test plugin unloaded!");
        Ok(())
    }
}

game_engine_export_plugin!(TestPlugin);
EOF
```

### 2. Build and Package

```bash
# Build
cargo build --release

# Package
plugin-cli package ./test-plugin

# Output: test-plugin-1.0.0.tar.gz
```

### 3. Publish Plugin

```bash
# Login first
plugin-cli login

# Publish
plugin-cli publish ./test-plugin
```

## Development Workflow

### Backend Development

```bash
cd backend

# Watch mode
cargo watch -x run

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

### Frontend Development

```bash
cd frontend

# Development server
npm run dev

# Type check
npm run type-check

# Lint
npm run lint

# Build
npm run build
```

### Database Migrations

```bash
cd backend

# Create new migration
sqlx migrate add create_new_table

# Run migrations
sqlx migrate run

# Rollback
sqlx migrate revert
```

## Common Tasks

### Add New API Endpoint

1. Define route in `backend/src/routes/plugins.rs`
2. Implement handler in `backend/src/handlers/`
3. Add types in `backend/src/models/mod.rs`
4. Test with curl or Postman

### Add New Frontend Page

1. Create page in `frontend/src/pages/`
2. Add components in `frontend/src/components/`
3. Add API service in `frontend/src/services/`
4. Add types in `frontend/src/types/`

### Add New CLI Command

1. Add command variant in `game_engine/src/plugin/cli.rs`
2. Implement handler in same file
3. Test with `plugin-cli <command>`

## Troubleshooting

### Database Connection Failed

```bash
# Check database is running
docker ps | grep plugin-marketplace-db

# View database logs
docker logs plugin-marketplace-db

# Test connection
psql postgresql://admin:password@localhost:5432/plugin_marketplace
```

### Port Already in Use

```bash
# Find process using port
lsof -i :8080  # Backend
lsof -i :3000  # Frontend
lsof -i :5432  # Database

# Kill process
kill -9 <PID>
```

### Build Errors

```bash
# Clean build
cd backend
cargo clean
cargo build

# Update dependencies
cargo update
```

### Frontend Errors

```bash
cd frontend

# Clear cache
rm -rf .next node_modules
npm install

# Rebuild
npm run dev
```

## Next Steps

1. **Read the Documentation**
   - [README.md](./README.md) - Complete overview
   - [API.md](./docs/API.md) - API reference
   - [DEPLOYMENT.md](./docs/DEPLOYMENT.md) - Deployment guide

2. **Configure for Production**
   - Update `.env` files with production values
   - Set up SSL certificates
   - Configure CDN
   - Set up monitoring

3. **Customize**
   - Add your branding
   - Configure authentication
   - Set up payment processing
   - Customize categories

4. **Deploy**
   - Choose deployment option (Docker/Kubernetes/Manual)
   - Follow deployment guide
   - Set up CI/CD
   - Monitor performance

## Getting Help

- **Documentation**: Check the `docs/` folder
- **Issues**: Report bugs on GitHub
- **Community**: Join our Discord server
- **Email**: support@gameengine.com

## Tips

- Use `docker-compose` for easiest setup
- Seed the database to see sample data
- Test CLI tool before publishing plugins
- Check logs when troubleshooting
- Keep dependencies updated
- Backup database regularly

Enjoy using the Plugin Marketplace! 🚀
