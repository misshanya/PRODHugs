-- +goose Up
ALTER TABLE hugs ADD COLUMN comment TEXT;
ALTER TABLE hugs ADD CONSTRAINT hugs_comment_length CHECK (char_length(comment) <= 140);

-- +goose Down
ALTER TABLE hugs DROP CONSTRAINT IF EXISTS hugs_comment_length;
ALTER TABLE hugs DROP COLUMN IF EXISTS comment;
