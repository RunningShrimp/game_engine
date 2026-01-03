#!/bin/bash

# Database seeding script
# This script populates the database with initial sample data

set -e

echo "🌱 Seeding Plugin Marketplace Database..."

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

# Insert sample categories
echo "📁 Inserting categories..."
psql "$DATABASE_URL" << EOF
-- Clear existing data
TRUNCATE TABLE download_events, view_events, review_votes, reviews, plugin_versions, plugins, categories CASCADE;

-- Insert categories
INSERT INTO categories (name, slug, description, icon, display_order) VALUES
('Rendering', 'rendering', 'Plugins for graphics, rendering, and visual effects', '🎨', 1),
('Physics', 'physics', 'Physics simulation and collision detection plugins', '⚡', 2),
('AI', 'ai', 'Artificial intelligence and behavior systems', '🤖', 3),
('Audio', 'audio', 'Audio processing, music, and sound effects', '🔊', 4),
('Tools', 'tools', 'Developer tools and utilities', '🛠️', 5),
('UI', 'ui', 'User interface components and systems', '🖼️', 6),
('Networking', 'networking', 'Multiplayer and networking functionality', '🌐', 7),
('Animation', 'animation', 'Animation and skeletal systems', '🎬', 8),
('Input', 'input', 'Input handling and device support', '🎮', 9),
('File Formats', 'formats', 'Asset loading and file format support', '📄', 10);
EOF

# Insert sample users
echo "👤 Inserting users..."
psql "$DATABASE_URL" << EOF
-- Insert admin user
INSERT INTO users (email, username, password_hash, role) VALUES
('admin@gameengine.com', 'admin', '\$2b\$12\$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU9iKXKBuJqW', 'admin'),
('john@example.com', 'johndoe', '\$2b\$12\$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU9iKXKBuJqW', 'developer'),
('jane@example.com', 'janedoe', '\$2b\$12\$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU9iKXKBuJqW', 'developer');
EOF

# Insert sample plugins
echo "🔌 Inserting plugins..."
psql "$DATABASE_URL" << EOF
-- Insert sample plugins
INSERT INTO plugins (id, name, slug, description, author_id, version, latest_version, categories, tags, license, screenshots, rating_average, rating_count, downloads, pricing_type, manifest, status) VALUES
(gen_random_uuid(), 'Terrain Generator', 'terrain-generator', 'Procedural terrain generation with support for multiple biomes and erosion simulation', (SELECT id FROM users WHERE username = 'johndoe'), '1.2.0', '1.2.0', ARRAY['rendering', 'tools'], ARRAY['terrain', 'procedural', 'generation', 'biome'], 'MIT', ARRAY['https://example.com/screenshot1.png'], 4.5, 128, 15000, 'free', '{"name": "terrain-generator", "version": "1.2.0"}'::jsonb, 'approved'),

(gen_random_uuid(), 'Advanced Physics', 'advanced-physics', 'High-fidelity physics simulation with soft body dynamics and fluid simulation', (SELECT id FROM users WHERE username = 'janedoe'), '2.0.0', '2.0.0', ARRAY['physics'], ARRAY['physics', 'simulation', 'soft-body', 'fluid'], 'MIT', ARRAY['https://example.com/screenshot2.png'], 4.8, 256, 25000, 'paid', '{"name": "advanced-physics", "version": "2.0.0"}'::jsonb, 'approved'),

(gen_random_uuid(), 'AI Behavior Tree', 'ai-behavior-tree', 'Visual behavior tree editor for AI character logic', (SELECT id FROM users WHERE username = 'johndoe'), '1.0.0', '1.0.0', ARRAY['ai', 'tools'], ARRAY['ai', 'behavior-tree', 'visual-editor'], 'Apache-2.0', ARRAY['https://example.com/screenshot3.png'], 4.2, 89, 8500, 'free', '{"name": "ai-behavior-tree", "version": "1.0.0"}'::jsonb, 'approved'),

(gen_random_uuid(), 'Audio Manager', 'audio-manager', 'Advanced audio system with 3D spatial audio and real-time effects', (SELECT id FROM users WHERE username = 'janedoe'), '1.5.0', '1.5.0', ARRAY['audio'], ARRAY['audio', 'spatial', '3d', 'effects'], 'MIT', ARRAY['https://example.com/screenshot4.png'], 4.6, 167, 12000, 'free', '{"name": "audio-manager", "version": "1.5.0"}'::jsonb, 'approved');

-- Insert sample reviews
INSERT INTO reviews (plugin_id, user_id, rating, title, content) VALUES
((SELECT id FROM plugins WHERE slug = 'terrain-generator'), (SELECT id FROM users WHERE username = 'janedoe'), 5, 'Excellent plugin!', 'This plugin saved me hours of work. Highly recommended!'),
((SELECT id FROM plugins WHERE slug = 'terrain-generator'), (SELECT id FROM users WHERE username = 'admin'), 4, 'Great but needs documentation', 'The plugin works well but documentation could be better.'),
((SELECT id FROM plugins WHERE slug = 'advanced-physics'), (SELECT id FROM users WHERE username = 'johndoe'), 5, 'Best physics engine!', 'Incredible realistic physics simulation. Worth every penny.');

-- Refresh materialized view
REFRESH MATERIALIZED VIEW CONCURRENTLY category_plugin_counts;
EOF

echo "✨ Database seeding complete!"
echo ""
echo "📊 Sample data created:"
echo "  - 10 categories"
echo "  - 3 users (1 admin, 2 developers)"
echo "  - 4 plugins"
echo "  - 3 reviews"
echo ""
echo "🔐 Admin credentials:"
echo "  Email: admin@gameengine.com"
echo "  Password: admin123"
echo ""
echo "👤 Developer credentials:"
echo "  Email: john@example.com / jane@example.com"
echo "  Password: user123"
