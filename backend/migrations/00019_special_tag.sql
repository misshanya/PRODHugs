ALTER TABLE users ADD COLUMN special_tag TEXT;
ALTER TABLE users ADD CONSTRAINT users_special_tag_length CHECK (char_length(special_tag) <= 20);

