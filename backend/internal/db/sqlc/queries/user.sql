-- name: CreateUser :one
INSERT INTO users (username, password, role, gender)
VALUES (
    $1, $2, $3, $4
)
RETURNING *;

-- name: GetUserByUsername :one
SELECT *
FROM users
WHERE username = $1;

-- name: GetUserByID :one
SELECT *
FROM users
WHERE id = $1;

-- name: SearchUsers :many
SELECT id, username, role, gender
FROM users
WHERE username ILIKE '%' || @query::text || '%'
  AND id NOT IN (
    SELECT blocked_id FROM user_blocks WHERE blocker_id = @viewer_id::uuid
    UNION
    SELECT blocker_id FROM user_blocks WHERE blocked_id = @viewer_id::uuid
  )
ORDER BY username
LIMIT @lim::int OFFSET @off::int;

-- name: ListAllUsers :many
SELECT id, username, role, gender
FROM users
WHERE id NOT IN (
    SELECT blocked_id FROM user_blocks WHERE blocker_id = @viewer_id::uuid
    UNION
    SELECT blocker_id FROM user_blocks WHERE blocked_id = @viewer_id::uuid
  )
ORDER BY username
LIMIT @lim::int OFFSET @off::int;

-- name: GetLeaderboard :many
SELECT
    u.id,
    u.username,
    u.role,
    u.gender,
    COALESCE(given.cnt, 0) + COALESCE(received.cnt, 0) AS total_hugs,
    COALESCE(given.cnt, 0) AS hugs_given,
    COALESCE(received.cnt, 0) AS hugs_received
FROM users u
LEFT JOIN (
    SELECT giver_id, COUNT(*) AS cnt FROM hugs WHERE status = 'completed' GROUP BY giver_id
) given ON given.giver_id = u.id
LEFT JOIN (
    SELECT receiver_id, COUNT(*) AS cnt FROM hugs WHERE status = 'completed' GROUP BY receiver_id
) received ON received.receiver_id = u.id
ORDER BY total_hugs DESC
LIMIT @lim::int OFFSET @off::int;

-- name: GetUserStats :one
SELECT
    COUNT(*) FILTER (WHERE giver_id = @user_id::uuid)::bigint AS hugs_given,
    COUNT(*) FILTER (WHERE receiver_id = @user_id::uuid)::bigint AS hugs_received,
    COUNT(*)::bigint AS total_hugs
FROM hugs
WHERE (giver_id = @user_id::uuid OR receiver_id = @user_id::uuid)
  AND status = 'completed';

-- name: GetRecentHugsFeed :many
SELECT
    h.id,
    h.giver_id,
    h.receiver_id,
    h.created_at,
    g.username AS giver_username,
    r.username AS receiver_username,
    g.gender AS giver_gender
FROM hugs h
JOIN users g ON g.id = h.giver_id
JOIN users r ON r.id = h.receiver_id
WHERE h.status = 'completed'
ORDER BY h.created_at DESC
LIMIT @lim::int;

-- name: UpdateUserSettings :one
UPDATE users
SET gender = $2
WHERE id = $1
RETURNING *;

-- name: UpdateUserPassword :exec
UPDATE users
SET password = $2
WHERE id = $1;

-- name: BanUser :one
UPDATE users
SET banned_at = NOW()
WHERE id = $1 AND role != 'admin'
RETURNING *;

-- name: UnbanUser :one
UPDATE users
SET banned_at = NULL
WHERE id = $1
RETURNING *;

-- name: CountUsers :one
SELECT COUNT(*) FROM users;

-- name: CountBannedUsers :one
SELECT COUNT(*) FROM users WHERE banned_at IS NOT NULL;

-- name: ListUsersAdmin :many
SELECT u.id, u.username, u.role, u.gender, u.banned_at,
       COALESCE(b.amount, 0)::int AS balance
FROM users u
LEFT JOIN balances b ON b.user_id = u.id
ORDER BY u.username
LIMIT @lim::int OFFSET @off::int;

-- name: AdminUpdateUsername :one
UPDATE users
SET username = $2
WHERE id = $1
RETURNING *;

-- name: AdminUpdateGender :one
UPDATE users
SET gender = $2
WHERE id = $1
RETURNING *;

-- name: AdminUpdatePassword :exec
UPDATE users
SET password = $2
WHERE id = $1;

-- name: GetUserSlots :one
SELECT hug_slots FROM users WHERE id = $1;

-- name: IncrementUserSlots :one
UPDATE users
SET hug_slots = hug_slots + 1
WHERE id = $1 AND hug_slots < 5
RETURNING hug_slots;
