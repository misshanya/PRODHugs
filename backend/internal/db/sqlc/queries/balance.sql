-- name: GetBalance :one
SELECT user_id, amount, updated_at
FROM balances
WHERE user_id = $1;

-- name: CreateBalance :one
INSERT INTO balances (user_id, amount)
VALUES ($1, $2)
RETURNING user_id, amount, updated_at;

-- name: AddBalance :one
UPDATE balances
SET amount = amount + @delta::int, updated_at = now()
WHERE user_id = $1
RETURNING user_id, amount, updated_at;

-- name: DeductBalance :one
UPDATE balances
SET amount = amount - @delta::int, updated_at = now()
WHERE user_id = $1 AND amount >= @delta::int
RETURNING user_id, amount, updated_at;

-- name: EnsureBalance :one
INSERT INTO balances (user_id, amount)
VALUES ($1, 0)
ON CONFLICT (user_id) DO NOTHING
RETURNING user_id, amount, updated_at;
