-- +goose Up
-- +goose StatementBegin

-- Add status and accepted_at to hugs table
ALTER TABLE hugs ADD COLUMN status TEXT NOT NULL DEFAULT 'completed'
    CHECK (status IN ('pending', 'completed', 'declined', 'expired', 'cancelled'));
ALTER TABLE hugs ADD COLUMN accepted_at TIMESTAMPTZ;

-- Backfill: all existing hugs are completed, accepted_at = created_at
UPDATE hugs SET accepted_at = created_at WHERE status = 'completed';

-- Index for inbox queries (pending hugs for a receiver)
CREATE INDEX idx_hugs_receiver_status ON hugs(receiver_id, status) WHERE status = 'pending';

-- Index for outgoing pending hug check
CREATE INDEX idx_hugs_giver_status ON hugs(giver_id, status) WHERE status = 'pending';

-- Rework hug_cooldowns to be symmetric (shared per pair)
-- Drop old table and recreate with canonical pair ordering
DROP TABLE hug_cooldowns;

CREATE TABLE hug_cooldowns (
    user_a_id UUID NOT NULL REFERENCES users(id),
    user_b_id UUID NOT NULL REFERENCES users(id),
    last_hug_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    cooldown_seconds INTEGER NOT NULL DEFAULT 3600,
    decline_cooldown_until TIMESTAMPTZ,
    PRIMARY KEY (user_a_id, user_b_id),
    CHECK (user_a_id < user_b_id)
);

-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin

ALTER TABLE hugs DROP COLUMN status;
ALTER TABLE hugs DROP COLUMN accepted_at;
DROP TABLE hug_cooldowns;
CREATE TABLE hug_cooldowns (
    giver_id UUID NOT NULL REFERENCES users(id),
    receiver_id UUID NOT NULL REFERENCES users(id),
    last_hug_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    cooldown_seconds INTEGER NOT NULL DEFAULT 3600,
    PRIMARY KEY (giver_id, receiver_id)
);

-- +goose StatementEnd
