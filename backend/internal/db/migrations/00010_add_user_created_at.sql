-- +goose Up
ALTER TABLE users ADD COLUMN created_at TIMESTAMPTZ;

-- +goose Down
ALTER TABLE users DROP COLUMN created_at;
