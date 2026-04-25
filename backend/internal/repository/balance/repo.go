package balance

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

func toModelBalance(b storage.Balance) *models.Balance {
	return &models.Balance{
		UserID:    b.UserID,
		Amount:    b.Amount,
		UpdatedAt: b.UpdatedAt.Time,
	}
}
