-- name: InsertHug :one
INSERT INTO hugs (giver_id, receiver_id, status, hug_type, comment)
VALUES ($1, $2, $3, $4, $5)
RETURNING *;

-- name: GetHugByID :one
SELECT id, giver_id, receiver_id, status, hug_type, created_at, accepted_at, comment
FROM hugs
WHERE id = $1;

-- name: AcceptHug :one
UPDATE hugs SET status = 'completed', accepted_at = now()
WHERE id = $1 AND receiver_id = $2 AND status = 'pending'
RETURNING *;

-- name: DeclineHug :one
UPDATE hugs SET status = 'declined'
WHERE id = $1 AND receiver_id = $2 AND status = 'pending'
RETURNING *;

-- name: CancelHug :one
UPDATE hugs SET status = 'cancelled'
WHERE id = $1 AND giver_id = $2 AND status = 'pending'
RETURNING *;

-- name: GetPendingHugsForUser :many
SELECT h.id, h.giver_id, h.receiver_id, h.status, h.created_at, h.accepted_at,
       h.hug_type, h.comment,
       g.username AS giver_username, g.gender AS giver_gender,
       g.display_name AS giver_display_name
FROM hugs h
JOIN users g ON g.id = h.giver_id
WHERE h.receiver_id = $1
  AND h.status = 'pending'
  AND h.created_at > now() - INTERVAL '24 hours'
ORDER BY h.created_at DESC;

-- name: GetOutgoingPendingHugs :many
SELECT h.id, h.giver_id, h.receiver_id, h.status, h.created_at, h.accepted_at,
       h.hug_type, h.comment,
       r.username AS receiver_username, r.gender AS receiver_gender,
       r.display_name AS receiver_display_name
FROM hugs h
JOIN users r ON r.id = h.receiver_id
WHERE h.giver_id = $1
  AND h.status = 'pending'
  AND h.created_at > now() - INTERVAL '24 hours'
ORDER BY h.created_at DESC;

-- name: CountPendingHugsForUser :one
SELECT COUNT(*) FROM hugs
WHERE receiver_id = $1
  AND status = 'pending'
  AND created_at > now() - INTERVAL '24 hours';

-- name: HasOutgoingPendingHug :one
SELECT EXISTS(
    SELECT 1 FROM hugs
    WHERE giver_id = $1
      AND status = 'pending'
      AND created_at > now() - INTERVAL '24 hours'
) AS has_pending;

-- name: HasPendingHugForPair :one
SELECT EXISTS(
    SELECT 1 FROM hugs
    WHERE giver_id = $1
      AND receiver_id = $2
      AND status = 'pending'
      AND created_at > now() - INTERVAL '24 hours'
) AS has_pending;

-- name: CheckSuggestEligibility :one
SELECT
    (SELECT COUNT(*) FROM hugs
     WHERE giver_id = @giver_id::uuid
       AND status = 'pending'
       AND created_at > now() - INTERVAL '24 hours'
    )::int AS outgoing_count,
    EXISTS(
        SELECT 1 FROM hugs
        WHERE giver_id = @giver_id::uuid
          AND receiver_id = @receiver_id::uuid
          AND status = 'pending'
          AND created_at > now() - INTERVAL '24 hours'
    ) AS pair_pending,
    EXISTS(
        SELECT 1 FROM hugs
        WHERE giver_id = @receiver_id::uuid
          AND receiver_id = @giver_id::uuid
          AND status = 'pending'
          AND created_at > now() - INTERVAL '24 hours'
    ) AS reverse_pending;

-- name: ExpirePendingHugs :exec
UPDATE hugs SET status = 'expired'
WHERE status = 'pending'
  AND created_at <= now() - INTERVAL '24 hours';

-- name: ListHugsByUser :many
SELECT
    h.id,
    h.giver_id,
    h.receiver_id,
    COALESCE(h.accepted_at, h.created_at) AS created_at,
    h.hug_type,
    (h.comment IS NOT NULL)::bool AS has_comment,
    g.username AS giver_username,
    r.username AS receiver_username,
    g.gender AS giver_gender,
    g.display_name AS giver_display_name,
    r.display_name AS receiver_display_name
FROM hugs h
JOIN users g ON g.id = h.giver_id
JOIN users r ON r.id = h.receiver_id
WHERE (h.giver_id = @user_id::uuid OR h.receiver_id = @user_id::uuid)
  AND h.status = 'completed'
ORDER BY COALESCE(h.accepted_at, h.created_at) DESC
LIMIT @lim::int OFFSET @off::int;

-- name: CountHugsReceived :one
SELECT COUNT(*)
FROM hugs
WHERE receiver_id = $1
  AND status = 'completed';

-- name: CountHugsGiven :one
SELECT COUNT(*)
FROM hugs
WHERE giver_id = $1
  AND status = 'completed';

-- name: CountMutualHugs :one
SELECT
    COUNT(*)::bigint AS mutual_total,
    COUNT(*) FILTER (WHERE giver_id = @user_a AND receiver_id = @user_b)::bigint AS mutual_given,
    COUNT(*) FILTER (WHERE giver_id = @user_b AND receiver_id = @user_a)::bigint AS mutual_received
FROM hugs
WHERE ((giver_id = @user_a AND receiver_id = @user_b)
   OR (giver_id = @user_b AND receiver_id = @user_a))
  AND status = 'completed';

-- name: GetHugDetail :one
SELECT
    h.id,
    h.giver_id,
    h.receiver_id,
    h.status,
    h.hug_type,
    h.comment,
    h.created_at,
    h.accepted_at,
    g.username AS giver_username,
    g.gender AS giver_gender,
    g.display_name AS giver_display_name,
    r.username AS receiver_username,
    r.gender AS receiver_gender,
    r.display_name AS receiver_display_name
FROM hugs h
JOIN users g ON g.id = h.giver_id
JOIN users r ON r.id = h.receiver_id
WHERE h.id = $1;

-- name: GetHugActivity :many
SELECT
    bucket::timestamptz AS bucket_time,
    COALESCE(COUNT(h.id), 0)::bigint AS hug_count
FROM generate_series(
    DATE_TRUNC('hour', NOW() - INTERVAL '23 hours'),
    DATE_TRUNC('hour', NOW()),
    '1 hour'::interval
) AS bucket
LEFT JOIN hugs h
    ON h.created_at >= bucket
   AND h.created_at < bucket + '1 hour'::interval
   AND h.status = 'completed'
GROUP BY bucket
ORDER BY bucket;
