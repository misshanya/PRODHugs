-- +goose Up
-- +goose StatementBegin

-- Balance for each user (earned by receiving hugs + daily rewards)
CREATE TABLE IF NOT EXISTS balances (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    amount  INTEGER NOT NULL DEFAULT 0 CHECK (amount >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Hug history
CREATE TABLE IF NOT EXISTS hugs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    giver_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    receiver_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (giver_id <> receiver_id)
);

CREATE INDEX idx_hugs_giver_id ON hugs(giver_id);
CREATE INDEX idx_hugs_receiver_id ON hugs(receiver_id);
CREATE INDEX idx_hugs_created_at ON hugs(created_at DESC);

-- Cooldown tracking per user pair (directional: A -> B)
CREATE TABLE IF NOT EXISTS hug_cooldowns (
    giver_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    receiver_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    last_hug_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    cooldown_seconds INTEGER NOT NULL DEFAULT 3600, -- 1 hour default
    PRIMARY KEY (giver_id, receiver_id)
);

-- Daily reward tracking
CREATE TABLE IF NOT EXISTS daily_rewards (
    user_id        UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    last_claimed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    streak_days    INTEGER NOT NULL DEFAULT 1
);

-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
DROP TABLE IF EXISTS daily_rewards;
DROP TABLE IF EXISTS hug_cooldowns;
DROP TABLE IF EXISTS hugs;
DROP TABLE IF EXISTS balances;
-- +goose StatementEnd
