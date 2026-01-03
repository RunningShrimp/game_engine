-- Initial schema for plugin marketplace

-- Users table
CREATE TYPE user_role AS ENUM ('user', 'developer', 'admin');

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    avatar_url TEXT,
    website TEXT,
    bio TEXT,
    role user_role NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);

-- Categories table
CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    icon TEXT,
    parent_id UUID REFERENCES categories(id) ON DELETE SET NULL,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_categories_slug ON categories(slug);
CREATE INDEX idx_categories_parent_id ON categories(parent_id);

-- Plugins table
CREATE TYPE pricing_type AS ENUM ('free', 'paid', 'freemium', 'subscription');
CREATE TYPE plugin_status AS ENUM ('draft', 'pending_review', 'approved', 'rejected', 'archived');

CREATE TABLE plugins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    slug VARCHAR(200) UNIQUE NOT NULL,
    description TEXT NOT NULL,
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    version VARCHAR(50) NOT NULL,
    latest_version VARCHAR(50) NOT NULL,
    categories TEXT[] NOT NULL DEFAULT '{}',
    tags TEXT[] NOT NULL DEFAULT '{}',
    license VARCHAR(50) NOT NULL,
    homepage TEXT,
    repository TEXT,
    documentation TEXT,
    screenshots TEXT[] NOT NULL DEFAULT '{}',
    videos JSONB NOT NULL DEFAULT '[]',
    rating_average DECIMAL(3,2) NOT NULL DEFAULT 0.0,
    rating_count INTEGER NOT NULL DEFAULT 0,
    downloads BIGINT NOT NULL DEFAULT 0,
    pricing_type pricing_type NOT NULL DEFAULT 'free',
    price DECIMAL(10,2),
    currency VARCHAR(3),
    trial_available BOOLEAN NOT NULL DEFAULT FALSE,
    manifest JSONB NOT NULL,
    status plugin_status NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_plugins_slug ON plugins(slug);
CREATE INDEX idx_plugins_author_id ON plugins(author_id);
CREATE INDEX idx_plugins_status ON plugins(status);
CREATE INDEX idx_plugins_categories ON plugins USING GIN(categories);
CREATE INDEX idx_plugins_tags ON plugins USING GIN(tags);
CREATE INDEX idx_plugins_rating ON plugins(rating_average DESC, rating_count DESC);
CREATE INDEX idx_plugins_downloads ON plugins(downloads DESC);
CREATE INDEX idx_plugins_created_at ON plugins(created_at DESC);
CREATE INDEX idx_plugins_search ON plugins USING GIN(to_tsvector('english', name || ' ' || description));

-- Plugin versions
CREATE TABLE plugin_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plugin_id UUID NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    version VARCHAR(50) NOT NULL,
    changelog TEXT NOT NULL,
    download_url TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    sha256 VARCHAR(64) NOT NULL UNIQUE,
    status plugin_status NOT NULL DEFAULT 'approved',
    published_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(plugin_id, version)
);

CREATE INDEX idx_plugin_versions_plugin_id ON plugin_versions(plugin_id);
CREATE INDEX idx_plugin_versions_version ON plugin_versions(version);

-- Reviews
CREATE TABLE reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plugin_id UUID NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    title VARCHAR(200) NOT NULL,
    content TEXT NOT NULL,
    helpful_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(plugin_id, user_id)
);

CREATE INDEX idx_reviews_plugin_id ON reviews(plugin_id);
CREATE INDEX idx_reviews_user_id ON reviews(user_id);
CREATE INDEX idx_reviews_rating ON reviews(rating);
CREATE INDEX idx_reviews_created_at ON reviews(created_at DESC);

-- Review helpful votes
CREATE TABLE review_votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    review_id UUID NOT NULL REFERENCES reviews(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(review_id, user_id)
);

CREATE INDEX idx_review_votes_review_id ON review_votes(review_id);
CREATE INDEX idx_review_votes_user_id ON review_votes(user_id);

-- Download analytics
CREATE TABLE download_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plugin_id UUID NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    version VARCHAR(50) NOT NULL,
    platform VARCHAR(50) NOT NULL,
    engine_version VARCHAR(50),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_download_events_plugin_id ON download_events(plugin_id);
CREATE INDEX idx_download_events_created_at ON download_events(created_at DESC);
CREATE INDEX idx_download_events_platform ON download_events(platform);

-- View analytics
CREATE TABLE view_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plugin_id UUID NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    session_id VARCHAR(255),
    referrer TEXT,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_view_events_plugin_id ON view_events(plugin_id);
CREATE INDEX idx_view_events_created_at ON view_events(created_at DESC);

-- Category plugin counts (materialized view for performance)
CREATE MATERIALIZED VIEW category_plugin_counts AS
SELECT
    c.id AS category_id,
    c.name AS category_name,
    COUNT(p.id) AS plugin_count
FROM categories c
LEFT JOIN plugins p ON p.status = 'approved' AND c.slug = ANY(p.categories)
GROUP BY c.id, c.name
ORDER BY plugin_count DESC;

CREATE UNIQUE INDEX idx_category_plugin_counts_id ON category_plugin_counts(category_id);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for updated_at
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_plugins_updated_at BEFORE UPDATE ON plugins
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_reviews_updated_at BEFORE UPDATE ON reviews
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to update plugin ratings
CREATE OR REPLACE FUNCTION update_plugin_rating()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE plugins
    SET
        rating_average = (
            SELECT COALESCE(AVG(rating), 0)
            FROM reviews
            WHERE plugin_id = NEW.plugin_id
        ),
        rating_count = (
            SELECT COUNT(*)
            FROM reviews
            WHERE plugin_id = NEW.plugin_id
        )
    WHERE id = NEW.plugin_id;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_plugin_rating_trigger
AFTER INSERT OR UPDATE OR DELETE ON reviews
FOR EACH ROW EXECUTE FUNCTION update_plugin_rating();

-- Function to refresh category counts
CREATE OR REPLACE FUNCTION refresh_category_counts()
RETURNS TRIGGER AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY category_plugin_counts;
    RETURN NULL;
END;
$$ language 'plpgsql';

CREATE TRIGGER refresh_category_counts_trigger
AFTER INSERT OR UPDATE OR DELETE ON plugins
FOR EACH STATEMENT EXECUTE FUNCTION refresh_category_counts();
