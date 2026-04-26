-- +goose Up
ALTER TABLE users ADD COLUMN gender TEXT;

-- +goose Down
ALTER TABLE users DROP COLUMN gender;
