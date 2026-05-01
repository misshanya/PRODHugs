package intimacy

import (
	"context"
	"database/sql"
	"errors"

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
	return &repo{q: storage.New(db)}
}

// GetPairIntimacy returns the intimacy data for a user pair. Returns nil if no row exists.
func (r *repo) GetPairIntimacy(ctx context.Context, userA, userB uuid.UUID) (*models.PairIntimacy, error) {
	q := repository.Queries(ctx, r.q)

	row, err := q.GetPairIntimacy(ctx, storage.GetPairIntimacyParams{
		UserA: userA,
		UserB: userB,
	})
	if err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return nil, nil
		}
		return nil, err
	}

	return &models.PairIntimacy{
		UserAID:     row.UserAID,
		UserBID:     row.UserBID,
		RawScore:    int(row.RawScore),
		LastHugAt:   row.LastHugAt.Time,
		LastDecayAt: row.LastDecayAt.Time,
	}, nil
}

// UpsertPairIntimacy increments the intimacy score and updates last_hug_at.
func (r *repo) UpsertPairIntimacy(ctx context.Context, userA, userB uuid.UUID) (*models.PairIntimacy, error) {
	q := repository.Queries(ctx, r.q)

	row, err := q.UpsertPairIntimacy(ctx, storage.UpsertPairIntimacyParams{
		UserA: userA,
		UserB: userB,
	})
	if err != nil {
		return nil, err
	}

	return &models.PairIntimacy{
		UserAID:     row.UserAID,
		UserBID:     row.UserBID,
		RawScore:    int(row.RawScore),
		LastHugAt:   row.LastHugAt.Time,
		LastDecayAt: row.LastDecayAt.Time,
	}, nil
}

// ApplyDecay applies time-based decay to all stale pairs.
func (r *repo) ApplyDecay(ctx context.Context) error {
	q := repository.Queries(ctx, r.q)
	return q.ApplyIntimacyDecay(ctx)
}

// GetUserConnections returns a user's connections ordered by intimacy score.
func (r *repo) GetUserConnections(ctx context.Context, userID uuid.UUID, limit, offset int32) ([]*models.ConnectionItem, error) {
	q := repository.Queries(ctx, r.q)

	rows, err := q.GetUserConnections(ctx, storage.GetUserConnectionsParams{
		UserID: userID,
		Lim:    limit,
		Off:    offset,
	})
	if err != nil {
		return nil, err
	}

	result := make([]*models.ConnectionItem, len(rows))
	for i, row := range rows {
		// Determine which user is the "other" user
		otherID := row.UserBID
		if row.UserBID == userID {
			otherID = row.UserAID
		}

		var gender *string
		if row.Gender.Valid {
			gender = &row.Gender.String
		}
		var displayName *string
		if row.DisplayName.Valid {
			displayName = &row.DisplayName.String
		}

		result[i] = &models.ConnectionItem{
			UserID:      otherID,
			Username:    row.Username,
			Gender:      gender,
			DisplayName: displayName,
			Intimacy:    models.ComputeIntimacyInfo(int(row.RawScore)),
		}
	}
	return result, nil
}

// GetLeaderboard returns the top pairs by intimacy score.
func (r *repo) GetLeaderboard(ctx context.Context, limit, offset int32) ([]*models.LeaderboardPairEntry, error) {
	q := repository.Queries(ctx, r.q)

	rows, err := q.GetIntimacyLeaderboard(ctx, storage.GetIntimacyLeaderboardParams{
		Lim: limit,
		Off: offset,
	})
	if err != nil {
		return nil, err
	}

	result := make([]*models.LeaderboardPairEntry, len(rows))
	for i, row := range rows {
		tier := models.ComputeTier(int(row.RawScore))

		var userADisplayName *string
		if row.UserADisplayName.Valid {
			userADisplayName = &row.UserADisplayName.String
		}
		var userBDisplayName *string
		if row.UserBDisplayName.Valid {
			userBDisplayName = &row.UserBDisplayName.String
		}

		result[i] = &models.LeaderboardPairEntry{
			UserAID:          row.UserAID,
			UserAUsername:    row.UserAUsername,
			UserADisplayName: userADisplayName,
			UserBID:          row.UserBID,
			UserBUsername:    row.UserBUsername,
			UserBDisplayName: userBDisplayName,
			RawScore:         int(row.RawScore),
			Tier:             tier.Level,
			TierName:         tier.Name,
		}
	}
	return result, nil
}
