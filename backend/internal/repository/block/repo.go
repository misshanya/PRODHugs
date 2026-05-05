package block

import (
	"context"

	"go-service-template/internal/db/sqlc/storage"
	"go-service-template/internal/models"
	"go-service-template/internal/repository"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgxpool"
)

type repo struct {
	q *storage.Queries
}

func New(db *pgxpool.Pool) *repo {
	return &repo{
		q: storage.New(db),
	}
}

func (r *repo) Block(ctx context.Context, blockerID, blockedID uuid.UUID) error {
	q := repository.Queries(ctx, r.q)
	return q.BlockUser(ctx, storage.BlockUserParams{
		BlockerID: blockerID,
		BlockedID: blockedID,
	})
}

func (r *repo) Unblock(ctx context.Context, blockerID, blockedID uuid.UUID) error {
	q := repository.Queries(ctx, r.q)
	return q.UnblockUser(ctx, storage.UnblockUserParams{
		BlockerID: blockerID,
		BlockedID: blockedID,
	})
}

func (r *repo) GetBlockedUsers(ctx context.Context, userID uuid.UUID) ([]*models.BlockedUser, error) {
	q := repository.Queries(ctx, r.q)

	rows, err := q.GetBlockedUsers(ctx, userID)
	if err != nil {
		return nil, err
	}

	result := make([]*models.BlockedUser, len(rows))
	for i, row := range rows {
		var gender *string
		if row.Gender.Valid {
			gender = &row.Gender.String
		}
		var displayName *string
		if row.DisplayName.Valid {
			displayName = &row.DisplayName.String
		}
		var tag *string
		if row.Tag.Valid {
			tag = &row.Tag.String
		}
		var specialTag *string
		if row.SpecialTag.Valid {
			specialTag = &row.SpecialTag.String
		}
		result[i] = &models.BlockedUser{
			ID:          row.ID,
			Username:    row.Username,
			Gender:      gender,
			DisplayName: displayName,
			Tag:         tag,
			SpecialTag:  specialTag,
			CreatedAt:   row.CreatedAt.Time,
		}
	}
	return result, nil
}

func (r *repo) IsBlockedByEither(ctx context.Context, userA, userB uuid.UUID) (bool, error) {
	q := repository.Queries(ctx, r.q)
	return q.IsBlockedByEither(ctx, storage.IsBlockedByEitherParams{
		BlockerID: userA,
		BlockedID: userB,
	})
}
