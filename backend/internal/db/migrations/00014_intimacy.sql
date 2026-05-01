-- +goose Up
CREATE TABLE pair_intimacy (
    user_a_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    user_b_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    raw_score INT NOT NULL DEFAULT 0,
    last_hug_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_decay_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_a_id, user_b_id),
    CHECK (user_a_id < user_b_id)
);

CREATE INDEX idx_pair_intimacy_score ON pair_intimacy (raw_score DESC);

-- +goose Down
DROP INDEX IF EXISTS idx_pair_intimacy_score;
DROP TABLE IF EXISTS pair_intimacy;