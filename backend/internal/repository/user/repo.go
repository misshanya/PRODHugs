package user

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

func toModelUser(u storage.User) *models.User {
	var gender *string
	if u.Gender.Valid {
		gender = &u.Gender.String
	}
	return &models.User{
		ID:             u.ID,
		Username:       u.Username,
		Role:           u.Role,
		HashedPassword: u.Password,
		Gender:         gender,
	}
}
