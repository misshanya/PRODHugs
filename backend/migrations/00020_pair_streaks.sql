CREATE TABLE pair_streaks (
    user_a_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    user_b_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    current_streak INT NOT NULL DEFAULT 0,
    best_streak INT NOT NULL DEFAULT 0,
    last_streak_date DATE,
    a_hugged_today BOOLEAN NOT NULL DEFAULT FALSE,
    b_hugged_today BOOLEAN NOT NULL DEFAULT FALSE,
    today_date DATE NOT NULL DEFAULT CURRENT_DATE,
    PRIMARY KEY (user_a_id, user_b_id),
    CHECK (user_a_id < user_b_id)
);

CREATE INDEX idx_pair_streaks_current ON pair_streaks (current_streak DESC);

