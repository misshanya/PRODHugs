package hug

import (
	"context"
	"time"

	"go-service-template/internal/db/sqlc/storage"
	"go-service-template/internal/models"
	"go-service-template/internal/repository"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgtype"
)

func (r *repo) GetCooldown(ctx context.Context, userA, userB uuid.UUID) (*models.HugCooldown, error) {
	q := repository.Queries(ctx, r.q)

	c, err := q.GetCooldown(ctx, storage.GetCooldownParams{
		Column1: userA,
		Column2: userB,
	})
	if err != nil {
		if err == pgx.ErrNoRows {
			return nil, nil // no cooldown exists yet
		}
		return nil, err
	}

	return toModelCooldown(c), nil
}

func (r *repo) UpsertCooldown(ctx context.Context, userA, userB uuid.UUID, cooldownSeconds int32) (*models.HugCooldown, error) {
	q := repository.Queries(ctx, r.q)

	c, err := q.UpsertCooldown(ctx, storage.UpsertCooldownParams{
		Column1:         userA,
		Column2:         userB,
		CooldownSeconds: cooldownSeconds,
	})
	if err != nil {
		return nil, err
	}

	return toModelCooldown(c), nil
}

func (r *repo) ReduceCooldown(ctx context.Context, userA, userB uuid.UUID, reduction int32) (*models.HugCooldown, error) {
	q := repository.Queries(ctx, r.q)

	c, err := q.ReduceCooldown(ctx, storage.ReduceCooldownParams{
		Column1:   userA,
		Column2:   userB,
		Reduction: reduction,
	})
	if err != nil {
		if err == pgx.ErrNoRows {
			return nil, nil
		}
		return nil, err
	}

	return toModelCooldown(c), nil
}

func (r *repo) SetDeclineCooldown(ctx context.Context, userA, userB uuid.UUID, until time.Time) error {
	q := repository.Queries(ctx, r.q)

	return q.SetDeclineCooldown(ctx, storage.SetDeclineCooldownParams{
		Column1: userA,
		Column2: userB,
		DeclineCooldownUntil: pgtype.Timestamptz{
			Time:  until,
			Valid: true,
		},
	})
}
