-- +goose Up
ALTER TABLE hugs ADD COLUMN streak_tier TEXT NOT NULL DEFAULT '';

-- +goose Down
ALTER TABLE hugs DROP COLUMN IF EXISTS streak_tier;
