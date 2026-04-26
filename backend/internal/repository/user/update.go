package user

import (
	"context"
	"go-service-template/internal/db/sqlc/storage"
	"go-service-template/internal/models"
	"go-service-template/internal/repository"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgtype"
)

func (r *repo) UpdateSettings(ctx context.Context, id uuid.UUID, gender *string) (*models.User, error) {
	q := repository.Queries(ctx, r.q)

	var g pgtype.Text
	if gender != nil {
		g = pgtype.Text{String: *gender, Valid: true}
	}

	u, err := q.UpdateUserSettings(ctx, storage.UpdateUserSettingsParams{
		ID:     id,
		Gender: g,
	})
	if err != nil {
		return nil, err
	}

	return toModelUser(u), nil
}

func (r *repo) UpdatePassword(ctx context.Context, id uuid.UUID, hashedPassword string) error {
	q := repository.Queries(ctx, r.q)

	return q.UpdateUserPassword(ctx, storage.UpdateUserPasswordParams{
		ID:       id,
		Password: hashedPassword,
	})
}
