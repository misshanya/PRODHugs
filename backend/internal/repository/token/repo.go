package token

import (
	"context"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgxpool"
)

type repo struct {
	db *pgxpool.Pool
}

func New(db *pgxpool.Pool) *repo {
	return &repo{db: db}
}

func (r *repo) SaveRefreshToken(ctx context.Context, jti string, userID uuid.UUID, expiresAtUnix int64) error {
	_, err := r.db.Exec(ctx, `
		INSERT INTO refresh_tokens (jti, user_id, expires_at)
		VALUES ($1, $2::uuid, to_timestamp($3))
	`, jti, userID, expiresAtUnix)
	return err
}

func (r *repo) IsRefreshTokenActive(ctx context.Context, jti string) (bool, error) {
	var active bool
	err := r.db.QueryRow(ctx, `
		SELECT EXISTS(
			SELECT 1
			FROM refresh_tokens
			WHERE jti = $1
			  AND revoked_at IS NULL
			  AND expires_at > NOW()
		)
	`, jti).Scan(&active)
	return active, err
}

func (r *repo) RevokeRefreshToken(ctx context.Context, jti string) error {
	_, err := r.db.Exec(ctx, `
		UPDATE refresh_tokens
		SET revoked_at = NOW()
		WHERE jti = $1
		  AND revoked_at IS NULL
	`, jti)
	return err
}

func (r *repo) RevokeAllUserRefreshTokens(ctx context.Context, userID uuid.UUID) error {
	_, err := r.db.Exec(ctx, `
		UPDATE refresh_tokens
		SET revoked_at = NOW()
		WHERE user_id = $1::uuid
		  AND revoked_at IS NULL
	`, userID)
	return err
}
