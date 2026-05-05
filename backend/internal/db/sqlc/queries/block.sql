-- name: BlockUser :exec
INSERT INTO user_blocks (blocker_id, blocked_id)
VALUES ($1, $2)
ON CONFLICT DO NOTHING;

-- name: UnblockUser :exec
DELETE FROM user_blocks
WHERE blocker_id = $1 AND blocked_id = $2;

-- name: GetBlockedUsers :many
SELECT u.id, u.username, u.gender, u.display_name, u.tag, u.special_tag, ub.created_at
FROM user_blocks ub
JOIN users u ON u.id = ub.blocked_id
WHERE ub.blocker_id = $1
ORDER BY ub.created_at DESC;

-- name: IsBlockedByEither :one
SELECT EXISTS(
    SELECT 1 FROM user_blocks
    WHERE (blocker_id = $1 AND blocked_id = $2)
       OR (blocker_id = $2 AND blocked_id = $1)
) AS is_blocked;
