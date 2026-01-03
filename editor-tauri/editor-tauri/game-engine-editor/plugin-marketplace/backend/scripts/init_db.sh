#!/bin/bash

# Database initialization script
# This script sets up the PostgreSQL database and runs migrations

set -e

echo "🚀 Initializing Plugin Marketplace Database..."

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
else
    echo "❌ .env file not found!"
    exit 1
fi

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "❌ DATABASE_URL not set in .env file!"
    exit 1
fi

echo "📊 Database URL: ${DATABASE_URL%:*}:****@${DATABASE_URL##*@}"

# Install sqlx-cli if not already installed
if ! command -v sqlx &> /dev/null; then
    echo "📦 Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features rustls,postgres
fi

# Create database if it doesn't exist
echo "🔧 Creating database..."
sqlx database create --database-url "$DATABASE_URL" || echo "Database already exists"

# Run migrations
echo "🔄 Running migrations..."
sqlx migrate run --database-url "$DATABASE_URL" --source ./migrations

# Verify migrations
echo "✅ Verifying migrations..."
sqlx migrate info --database-url "$DATABASE_URL" --source ./migrations

echo "✨ Database initialization complete!"
echo ""
echo "📝 Next steps:"
echo "  1. Review the database schema"
echo "  2. Seed initial data (optional): ./scripts/seed_db.sh"
echo "  3. Start the backend server: cargo run"
