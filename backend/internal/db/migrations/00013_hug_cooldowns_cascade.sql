-- +goose Up
-- +goose StatementBegin

-- Add ON DELETE CASCADE to hug_cooldowns foreign keys so user deletion works.
ALTER TABLE hug_cooldowns
    DROP CONSTRAINT hug_cooldowns_user_a_id_fkey,
    ADD CONSTRAINT hug_cooldowns_user_a_id_fkey
        FOREIGN KEY (user_a_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE hug_cooldowns
    DROP CONSTRAINT hug_cooldowns_user_b_id_fkey,
    ADD CONSTRAINT hug_cooldowns_user_b_id_fkey
        FOREIGN KEY (user_b_id) REFERENCES users(id) ON DELETE CASCADE;

-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin

ALTER TABLE hug_cooldowns
    DROP CONSTRAINT hug_cooldowns_user_a_id_fkey,
    ADD CONSTRAINT hug_cooldowns_user_a_id_fkey
        FOREIGN KEY (user_a_id) REFERENCES users(id);

ALTER TABLE hug_cooldowns
    DROP CONSTRAINT hug_cooldowns_user_b_id_fkey,
    ADD CONSTRAINT hug_cooldowns_user_b_id_fkey
        FOREIGN KEY (user_b_id) REFERENCES users(id);

-- +goose StatementEnd
