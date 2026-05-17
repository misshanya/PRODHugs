ALTER TABLE hugs ADD COLUMN comment TEXT;
ALTER TABLE hugs ADD CONSTRAINT hugs_comment_length CHECK (char_length(comment) <= 140);

