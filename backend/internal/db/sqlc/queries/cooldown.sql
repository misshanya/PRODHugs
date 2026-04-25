-- name: GetCooldown :one
SELECT giver_id, receiver_id, last_hug_at, cooldown_seconds
FROM hug_cooldowns
WHERE giver_id = $1 AND receiver_id = $2;

-- name: UpsertCooldown :one
INSERT INTO hug_cooldowns (giver_id, receiver_id, last_hug_at, cooldown_seconds)
VALUES ($1, $2, now(), $3)
ON CONFLICT (giver_id, receiver_id)
DO UPDATE SET last_hug_at = now()
RETURNING giver_id, receiver_id, last_hug_at, cooldown_seconds;

-- name: ReduceCooldown :one
UPDATE hug_cooldowns
SET cooldown_seconds = GREATEST(cooldown_seconds - @reduction::int, 300)
WHERE giver_id = $1 AND receiver_id = $2
RETURNING giver_id, receiver_id, last_hug_at, cooldown_seconds;

-- name: GetOrCreateCooldown :one
SELECT giver_id, receiver_id,
       COALESCE(last_hug_at, '1970-01-01 00:00:00+00'::timestamptz) as last_hug_at,
       COALESCE(cooldown_seconds, 3600) as cooldown_seconds
FROM hug_cooldowns
WHERE giver_id = $1 AND receiver_id = $2
LIMIT 1;
