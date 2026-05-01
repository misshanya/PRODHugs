-- +goose Up
INSERT INTO pair_intimacy (user_a_id, user_b_id, raw_score, last_hug_at, last_decay_at)
SELECT
    LEAST(giver_id, receiver_id),
    GREATEST(giver_id, receiver_id),
    COUNT(*)::int,
    MAX(accepted_at),
    NOW()
FROM hugs
WHERE status = 'completed' AND accepted_at IS NOT NULL
GROUP BY LEAST(giver_id, receiver_id), GREATEST(giver_id, receiver_id)
ON CONFLICT (user_a_id, user_b_id) DO NOTHING;

-- +goose Down
-- No rollback — backfill is additive and 00014 DOWN drops the table anyway.