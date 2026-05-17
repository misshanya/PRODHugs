-- Partial indexes for hot queries filtering on completed hugs
CREATE INDEX IF NOT EXISTS idx_hugs_giver_completed ON hugs(giver_id) WHERE status = 'completed';
CREATE INDEX IF NOT EXISTS idx_hugs_receiver_completed ON hugs(receiver_id) WHERE status = 'completed';

-- Trigram index for ILIKE search on usernames
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX IF NOT EXISTS idx_users_username_trgm ON users USING GIN (username gin_trgm_ops);

