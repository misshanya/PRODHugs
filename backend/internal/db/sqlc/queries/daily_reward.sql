-- name: GetDailyReward :one
SELECT user_id, last_claimed_at, streak_days
FROM daily_rewards
WHERE user_id = $1;

-- name: ClaimDailyReward :one
INSERT INTO daily_rewards (user_id, last_claimed_at, streak_days)
VALUES ($1, now(), 1)
ON CONFLICT (user_id)
DO UPDATE SET
    streak_days = CASE
        WHEN daily_rewards.last_claimed_at::date = (now() - interval '1 day')::date
        THEN daily_rewards.streak_days + 1
        WHEN daily_rewards.last_claimed_at::date = now()::date
        THEN daily_rewards.streak_days  -- already claimed today, no change
        ELSE 1  -- streak reset
    END,
    last_claimed_at = CASE
        WHEN daily_rewards.last_claimed_at::date = now()::date
        THEN daily_rewards.last_claimed_at  -- don't update if already claimed today
        ELSE now()
    END
RETURNING user_id, last_claimed_at, streak_days;
