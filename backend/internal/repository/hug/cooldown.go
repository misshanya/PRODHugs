package hug

import (
	"context"
	"go-service-template/internal/db/sqlc/storage"
	"go-service-template/internal/models"
	"go-service-template/internal/repository"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
)

func (r *repo) GetCooldown(ctx context.Context, giverID, receiverID uuid.UUID) (*models.HugCooldown, error) {
	q := repository.Queries(ctx, r.q)

	c, err := q.GetCooldown(ctx, storage.GetCooldownParams{
		GiverID:    giverID,
		ReceiverID: receiverID,
	})
	if err != nil {
		if err == pgx.ErrNoRows {
			return nil, nil // no cooldown exists yet
		}
		return nil, err
	}

	return toModelCooldown(c), nil
}

func (r *repo) UpsertCooldown(ctx context.Context, giverID, receiverID uuid.UUID, cooldownSeconds int32) (*models.HugCooldown, error) {
	q := repository.Queries(ctx, r.q)

	c, err := q.UpsertCooldown(ctx, storage.UpsertCooldownParams{
		GiverID:         giverID,
		ReceiverID:      receiverID,
		CooldownSeconds: cooldownSeconds,
	})
	if err != nil {
		return nil, err
	}

	return toModelCooldown(c), nil
}

func (r *repo) ReduceCooldown(ctx context.Context, giverID, receiverID uuid.UUID, reduction int32) (*models.HugCooldown, error) {
	q := repository.Queries(ctx, r.q)

	c, err := q.ReduceCooldown(ctx, storage.ReduceCooldownParams{
		GiverID:    giverID,
		ReceiverID: receiverID,
		Reduction:  reduction,
	})
	if err != nil {
		if err == pgx.ErrNoRows {
			return nil, nil
		}
		return nil, err
	}

	return toModelCooldown(c), nil
}
