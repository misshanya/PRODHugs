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
ORDER BY username
LIMIT @lim::int OFFSET @off::int;

-- name: ListAllUsers :many
SELECT id, username, role, gender
FROM users
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
    SELECT giver_id, COUNT(*) AS cnt FROM hugs GROUP BY giver_id
) given ON given.giver_id = u.id
LEFT JOIN (
    SELECT receiver_id, COUNT(*) AS cnt FROM hugs GROUP BY receiver_id
) received ON received.receiver_id = u.id
ORDER BY total_hugs DESC
LIMIT @lim::int OFFSET @off::int;

-- name: GetUserStats :one
SELECT
    (SELECT COUNT(*) FROM hugs WHERE hugs.giver_id = @user_id::uuid)::bigint AS hugs_given,
    (SELECT COUNT(*) FROM hugs WHERE hugs.receiver_id = @user_id::uuid)::bigint AS hugs_received,
    (SELECT COUNT(*) FROM hugs WHERE hugs.giver_id = @user_id::uuid)::bigint +
    (SELECT COUNT(*) FROM hugs WHERE hugs.receiver_id = @user_id::uuid)::bigint AS total_hugs;

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
