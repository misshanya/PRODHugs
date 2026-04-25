package balance

import (
	"context"
	"go-service-template/internal/db/sqlc/storage"
	"go-service-template/internal/models"
	"go-service-template/internal/repository"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
)

func (r *repo) GetBalance(ctx context.Context, userID uuid.UUID) (*models.Balance, error) {
	q := repository.Queries(ctx, r.q)

	b, err := q.GetBalance(ctx, userID)
	if err != nil {
		if err == pgx.ErrNoRows {
			// Create balance on first access
			created, err := q.CreateBalance(ctx, storage.CreateBalanceParams{
				UserID: userID,
				Amount: 0,
			})
			if err != nil {
				return nil, err
			}
			return toModelBalance(created), nil
		}
		return nil, err
	}

	return toModelBalance(b), nil
}

func (r *repo) AddBalance(ctx context.Context, userID uuid.UUID, delta int32) (*models.Balance, error) {
	q := repository.Queries(ctx, r.q)

	// Ensure balance row exists
	_, _ = q.CreateBalance(ctx, storage.CreateBalanceParams{
		UserID: userID,
		Amount: 0,
	})

	b, err := q.AddBalance(ctx, storage.AddBalanceParams{
		UserID: userID,
		Delta:  delta,
	})
	if err != nil {
		return nil, err
	}

	return toModelBalance(b), nil
}

func (r *repo) DeductBalance(ctx context.Context, userID uuid.UUID, delta int32) (*models.Balance, error) {
	q := repository.Queries(ctx, r.q)

	b, err := q.DeductBalance(ctx, storage.DeductBalanceParams{
		UserID: userID,
		Delta:  delta,
	})
	if err != nil {
		if err == pgx.ErrNoRows {
			return nil, nil // insufficient balance
		}
		return nil, err
	}

	return toModelBalance(b), nil
}

func (r *repo) EnsureBalance(ctx context.Context, userID uuid.UUID) error {
	q := repository.Queries(ctx, r.q)
	_, _ = q.CreateBalance(ctx, storage.CreateBalanceParams{
		UserID: userID,
		Amount: 0,
	})
	return nil
}
