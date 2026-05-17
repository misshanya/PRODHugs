ALTER TABLE users ADD COLUMN telegram_id BIGINT;
CREATE UNIQUE INDEX users_telegram_id_key ON users (telegram_id) WHERE telegram_id IS NOT NULL;

