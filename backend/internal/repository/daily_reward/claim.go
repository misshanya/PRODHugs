package dailyreward

import (
	"context"
	"go-service-template/internal/models"
	"go-service-template/internal/repository"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
)

func (r *repo) GetDailyReward(ctx context.Context, userID uuid.UUID) (*models.DailyReward, error) {
	q := repository.Queries(ctx, r.q)

	d, err := q.GetDailyReward(ctx, userID)
	if err != nil {
		if err == pgx.ErrNoRows {
			return nil, nil // never claimed
		}
		return nil, err
	}

	return toModelDailyReward(d), nil
}

func (r *repo) ClaimDailyReward(ctx context.Context, userID uuid.UUID) (*models.DailyReward, error) {
	q := repository.Queries(ctx, r.q)

	d, err := q.ClaimDailyReward(ctx, userID)
	if err != nil {
		return nil, err
	}

	return toModelDailyReward(d), nil
}
