-- +goose Up
ALTER TABLE users ADD COLUMN telegram_id BIGINT;
CREATE UNIQUE INDEX users_telegram_id_key ON users (telegram_id) WHERE telegram_id IS NOT NULL;

-- +goose Down
DROP INDEX IF EXISTS users_telegram_id_key;
ALTER TABLE users DROP COLUMN IF EXISTS telegram_id;
