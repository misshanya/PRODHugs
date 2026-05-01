-- name: GetPairIntimacy :one
SELECT user_a_id, user_b_id, raw_score, last_hug_at, last_decay_at
FROM pair_intimacy
WHERE user_a_id = LEAST(@user_a::uuid, @user_b::uuid)
  AND user_b_id = GREATEST(@user_a::uuid, @user_b::uuid);

-- name: UpsertPairIntimacy :one
INSERT INTO pair_intimacy (user_a_id, user_b_id, raw_score, last_hug_at, last_decay_at)
VALUES (
    LEAST(@user_a::uuid, @user_b::uuid),
    GREATEST(@user_a::uuid, @user_b::uuid),
    1, NOW(), NOW()
)
ON CONFLICT (user_a_id, user_b_id) DO UPDATE
SET raw_score = pair_intimacy.raw_score + 1,
    last_hug_at = NOW()
RETURNING *;

-- name: ApplyIntimacyDecay :exec
UPDATE pair_intimacy
SET raw_score = GREATEST(
        raw_score - FLOOR(EXTRACT(EPOCH FROM (NOW() - last_decay_at)) / 259200)::int,
        0
    ),
    last_decay_at = NOW()
WHERE last_decay_at < NOW() - INTERVAL '3 days'
  AND raw_score > 0;

-- name: GetUserConnections :many
SELECT
    pi.user_a_id,
    pi.user_b_id,
    pi.raw_score,
    pi.last_hug_at,
    pi.last_decay_at,
    u.username,
    u.gender,
    u.display_name
FROM pair_intimacy pi
JOIN users u ON u.id = CASE
    WHEN pi.user_a_id = @user_id::uuid THEN pi.user_b_id
    ELSE pi.user_a_id
END
WHERE (pi.user_a_id = @user_id::uuid OR pi.user_b_id = @user_id::uuid)
  AND pi.raw_score > 0
ORDER BY pi.raw_score DESC
LIMIT @lim OFFSET @off;

-- name: GetIntimacyLeaderboard :many
SELECT
    pi.user_a_id,
    pi.user_b_id,
    pi.raw_score,
    ua.username AS user_a_username,
    ua.display_name AS user_a_display_name,
    ub.username AS user_b_username,
    ub.display_name AS user_b_display_name
FROM pair_intimacy pi
JOIN users ua ON ua.id = pi.user_a_id
JOIN users ub ON ub.id = pi.user_b_id
WHERE pi.raw_score > 0
ORDER BY pi.raw_score DESC
LIMIT @lim OFFSET @off;