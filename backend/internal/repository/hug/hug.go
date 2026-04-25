package hug

import (
	"context"
	"go-service-template/internal/models"
	"go-service-template/internal/repository"
	"go-service-template/internal/db/sqlc/storage"

	"github.com/google/uuid"
)

func (r *repo) InsertHug(ctx context.Context, giverID, receiverID uuid.UUID) (*models.Hug, error) {
	q := repository.Queries(ctx, r.q)

	h, err := q.InsertHug(ctx, storage.InsertHugParams{
		GiverID:    giverID,
		ReceiverID: receiverID,
	})
	if err != nil {
		return nil, err
	}

	return toModelHug(h), nil
}

func (r *repo) ListHugsByUser(ctx context.Context, userID uuid.UUID) ([]*models.Hug, error) {
	q := repository.Queries(ctx, r.q)

	rows, err := q.ListHugsByUser(ctx, userID)
	if err != nil {
		return nil, err
	}

	result := make([]*models.Hug, len(rows))
	for i, row := range rows {
		result[i] = toModelHug(row)
	}
	return result, nil
}

func (r *repo) CountHugsGiven(ctx context.Context, userID uuid.UUID) (int64, error) {
	q := repository.Queries(ctx, r.q)
	return q.CountHugsGiven(ctx, userID)
}

func (r *repo) CountHugsReceived(ctx context.Context, userID uuid.UUID) (int64, error) {
	q := repository.Queries(ctx, r.q)
	return q.CountHugsReceived(ctx, userID)
}
