-- Rollback initial schema

DROP TRIGGER IF EXISTS refresh_category_counts_trigger ON plugins;
DROP FUNCTION IF EXISTS refresh_category_counts();

DROP TRIGGER IF EXISTS update_plugin_rating_trigger ON reviews;
DROP FUNCTION IF EXISTS update_plugin_rating();

DROP TRIGGER IF EXISTS update_reviews_updated_at ON reviews;
DROP TRIGGER IF EXISTS update_plugins_updated_at ON plugins;
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
DROP FUNCTION IF EXISTS update_updated_at_column();

DROP MATERIALIZED VIEW IF EXISTS category_plugin_counts;

DROP TABLE IF EXISTS view_events;
DROP TABLE IF EXISTS download_events;
DROP TABLE IF EXISTS review_votes;
DROP TABLE IF EXISTS reviews;
DROP TABLE IF EXISTS plugin_versions;
DROP TABLE IF EXISTS plugins;
DROP TABLE IF EXISTS categories;
DROP TABLE IF EXISTS users;

DROP TYPE IF EXISTS plugin_status;
DROP TYPE IF EXISTS pricing_type;
DROP TYPE IF EXISTS user_role;
