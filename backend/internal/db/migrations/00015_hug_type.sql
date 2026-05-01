-- +goose Up
ALTER TABLE hugs ADD COLUMN hug_type TEXT NOT NULL DEFAULT 'standard';

-- +goose Down
ALTER TABLE hugs DROP COLUMN IF EXISTS hug_type;