package dailyreward

import (
	"go-service-template/internal/db/sqlc/storage"
	"go-service-template/internal/models"

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

func toModelDailyReward(d storage.DailyReward) *models.DailyReward {
	return &models.DailyReward{
		UserID:        d.UserID,
		LastClaimedAt: d.LastClaimedAt.Time,
		StreakDays:    d.StreakDays,
	}
}
