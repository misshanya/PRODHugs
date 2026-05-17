ALTER TABLE sudoku_captchas ADD COLUMN target_id UUID REFERENCES users(id) ON DELETE CASCADE;
UPDATE sudoku_captchas SET target_id = user_id WHERE target_id IS NULL;
ALTER TABLE sudoku_captchas ALTER COLUMN target_id SET NOT NULL;

ALTER TABLE users DROP COLUMN sudoku_cooldown_until;

