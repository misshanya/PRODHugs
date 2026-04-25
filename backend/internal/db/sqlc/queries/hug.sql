-- name: InsertHug :one
INSERT INTO hugs (giver_id, receiver_id)
VALUES ($1, $2)
RETURNING id, giver_id, receiver_id, created_at;

-- name: GetHugByID :one
SELECT id, giver_id, receiver_id, created_at
FROM hugs
WHERE id = $1;

-- name: ListHugsByUser :many
SELECT id, giver_id, receiver_id, created_at
FROM hugs
WHERE giver_id = $1 OR receiver_id = $1
ORDER BY created_at DESC;

-- name: CountHugsReceived :one
SELECT COUNT(*)
FROM hugs
WHERE receiver_id = $1;

-- name: CountHugsGiven :one
SELECT COUNT(*)
FROM hugs
WHERE giver_id = $1;
