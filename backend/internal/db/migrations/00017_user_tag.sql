-- +goose Up
ALTER TABLE users ADD COLUMN tag TEXT;

-- +goose Down
ALTER TABLE users DROP COLUMN tag;
