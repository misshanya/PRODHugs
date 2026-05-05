-- +goose Up
ALTER TABLE users ADD COLUMN special_tag TEXT;
ALTER TABLE users ADD CONSTRAINT users_special_tag_length CHECK (char_length(special_tag) <= 20);

-- +goose Down
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_special_tag_length;
ALTER TABLE users DROP COLUMN IF EXISTS special_tag;
