package hug

import (
	"context"
	"time"

	"go-service-template/internal/db/sqlc/storage"
	"go-service-template/internal/models"
	"go-service-template/internal/repository"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
)

func (r *repo) InsertHug(ctx context.Context, giverID, receiverID uuid.UUID, status string) (*models.Hug, error) {
	q := repository.Queries(ctx, r.q)

	h, err := q.InsertHug(ctx, storage.InsertHugParams{
		GiverID:    giverID,
		ReceiverID: receiverID,
		Status:     status,
	})
	if err != nil {
		return nil, err
	}

	return toModelHug(h), nil
}

func (r *repo) AcceptHug(ctx context.Context, hugID, receiverID uuid.UUID) (*models.Hug, error) {
	q := repository.Queries(ctx, r.q)

	h, err := q.AcceptHug(ctx, storage.AcceptHugParams{
		ID:         hugID,
		ReceiverID: receiverID,
	})
	if err != nil {
		if err == pgx.ErrNoRows {
			return nil, nil
		}
		return nil, err
	}

	return toModelHug(h), nil
}

func (r *repo) DeclineHug(ctx context.Context, hugID, receiverID uuid.UUID) (*models.Hug, error) {
	q := repository.Queries(ctx, r.q)

	h, err := q.DeclineHug(ctx, storage.DeclineHugParams{
		ID:         hugID,
		ReceiverID: receiverID,
	})
	if err != nil {
		if err == pgx.ErrNoRows {
			return nil, nil
		}
		return nil, err
	}

	return toModelHug(h), nil
}

func (r *repo) CancelHug(ctx context.Context, hugID, giverID uuid.UUID) (*models.Hug, error) {
	q := repository.Queries(ctx, r.q)

	h, err := q.CancelHug(ctx, storage.CancelHugParams{
		ID:      hugID,
		GiverID: giverID,
	})
	if err != nil {
		if err == pgx.ErrNoRows {
			return nil, nil
		}
		return nil, err
	}

	return toModelHug(h), nil
}

func (r *repo) GetHugByID(ctx context.Context, hugID uuid.UUID) (*models.Hug, error) {
	q := repository.Queries(ctx, r.q)

	row, err := q.GetHugByID(ctx, hugID)
	if err != nil {
		if err == pgx.ErrNoRows {
			return nil, nil
		}
		return nil, err
	}

	var acceptedAt *time.Time
	if row.AcceptedAt.Valid {
		t := row.AcceptedAt.Time
		acceptedAt = &t
	}

	return &models.Hug{
		ID:         row.ID,
		GiverID:    row.GiverID,
		ReceiverID: row.ReceiverID,
		Status:     row.Status,
		CreatedAt:  row.CreatedAt.Time,
		AcceptedAt: acceptedAt,
	}, nil
}

func (r *repo) GetPendingHugsForUser(ctx context.Context, userID uuid.UUID) ([]*models.PendingHugInboxItem, error) {
	q := repository.Queries(ctx, r.q)

	rows, err := q.GetPendingHugsForUser(ctx, userID)
	if err != nil {
		return nil, err
	}

	result := make([]*models.PendingHugInboxItem, len(rows))
	for i, row := range rows {
		result[i] = toModelPendingInboxItem(row)
	}
	return result, nil
}

func (r *repo) GetOutgoingPendingHug(ctx context.Context, userID uuid.UUID) (*models.OutgoingPendingHug, error) {
	q := repository.Queries(ctx, r.q)

	row, err := q.GetOutgoingPendingHug(ctx, userID)
	if err != nil {
		if err == pgx.ErrNoRows {
			return nil, nil
		}
		return nil, err
	}

	return toModelOutgoingPendingHug(row), nil
}

func (r *repo) CountPendingHugsForUser(ctx context.Context, userID uuid.UUID) (int64, error) {
	q := repository.Queries(ctx, r.q)
	return q.CountPendingHugsForUser(ctx, userID)
}

func (r *repo) HasOutgoingPendingHug(ctx context.Context, userID uuid.UUID) (bool, error) {
	q := repository.Queries(ctx, r.q)
	return q.HasOutgoingPendingHug(ctx, userID)
}

func (r *repo) HasPendingHugForPair(ctx context.Context, giverID, receiverID uuid.UUID) (bool, error) {
	q := repository.Queries(ctx, r.q)
	return q.HasPendingHugForPair(ctx, storage.HasPendingHugForPairParams{
		GiverID:    giverID,
		ReceiverID: receiverID,
	})
}

func (r *repo) ListHugsByUser(ctx context.Context, userID uuid.UUID) ([]*models.HugFeedItem, error) {
	q := repository.Queries(ctx, r.q)

	rows, err := q.ListHugsByUser(ctx, userID)
	if err != nil {
		return nil, err
	}

	result := make([]*models.HugFeedItem, len(rows))
	for i, row := range rows {
		result[i] = toModelHistoryItem(row)
	}
	return result, nil
}

func (r *repo) CountMutualHugs(ctx context.Context, userA, userB uuid.UUID) (*models.MutualHugStats, error) {
	q := repository.Queries(ctx, r.q)

	row, err := q.CountMutualHugs(ctx, storage.CountMutualHugsParams{
		UserA: userA,
		UserB: userB,
	})
	if err != nil {
		return nil, err
	}

	return &models.MutualHugStats{
		Total:    row.MutualTotal,
		Given:    row.MutualGiven,
		Received: row.MutualReceived,
	}, nil
}

func (r *repo) CountHugsGiven(ctx context.Context, userID uuid.UUID) (int64, error) {
	q := repository.Queries(ctx, r.q)
	return q.CountHugsGiven(ctx, userID)
}

func (r *repo) CountHugsReceived(ctx context.Context, userID uuid.UUID) (int64, error) {
	q := repository.Queries(ctx, r.q)
	return q.CountHugsReceived(ctx, userID)
}
