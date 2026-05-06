-- name: GetPairStreak :one
SELECT user_a_id, user_b_id, current_streak, best_streak,
       last_streak_date, a_hugged_today, b_hugged_today, today_date
FROM pair_streaks
WHERE user_a_id = LEAST(@user_a::uuid, @user_b::uuid)
  AND user_b_id = GREATEST(@user_a::uuid, @user_b::uuid);

-- name: UpsertPairStreak :one
INSERT INTO pair_streaks (user_a_id, user_b_id, current_streak, best_streak,
                          last_streak_date, a_hugged_today, b_hugged_today, today_date)
VALUES (
    LEAST(@user_a::uuid, @user_b::uuid),
    GREATEST(@user_a::uuid, @user_b::uuid),
    @current_streak::int,
    @best_streak::int,
    @last_streak_date::date,
    @a_hugged_today::bool,
    @b_hugged_today::bool,
    @today_date::date
)
ON CONFLICT (user_a_id, user_b_id) DO UPDATE
SET current_streak = @current_streak::int,
    best_streak = @best_streak::int,
    last_streak_date = @last_streak_date::date,
    a_hugged_today = @a_hugged_today::bool,
    b_hugged_today = @b_hugged_today::bool,
    today_date = @today_date::date
RETURNING *;

-- name: GetUserTopStreaks :many
SELECT
    ps.user_a_id,
    ps.user_b_id,
    ps.current_streak,
    ps.best_streak,
    ps.last_streak_date,
    ps.a_hugged_today,
    ps.b_hugged_today,
    ps.today_date,
    u.username AS other_username,
    u.display_name AS other_display_name,
    u.gender AS other_gender
FROM pair_streaks ps
JOIN users u ON u.id = CASE
    WHEN ps.user_a_id = @user_id::uuid THEN ps.user_b_id
    ELSE ps.user_a_id
END
WHERE (ps.user_a_id = @user_id::uuid OR ps.user_b_id = @user_id::uuid)
  AND ps.current_streak > 0
ORDER BY ps.current_streak DESC
LIMIT @lim::int;

-- name: GetPairStreakCalendar :many
SELECT
    (COALESCE(h.accepted_at, h.created_at))::date AS hug_date,
    COUNT(*)::bigint AS hug_count,
    (COUNT(*) FILTER (WHERE h.giver_id = @user_a::uuid) > 0
     AND COUNT(*) FILTER (WHERE h.giver_id = @user_b::uuid) > 0)::bool AS completed
FROM hugs h
WHERE ((h.giver_id = @user_a::uuid AND h.receiver_id = @user_b::uuid)
   OR (h.giver_id = @user_b::uuid AND h.receiver_id = @user_a::uuid))
  AND h.status = 'completed'
  AND COALESCE(h.accepted_at, h.created_at) >= @since::timestamptz
GROUP BY hug_date
ORDER BY hug_date;
