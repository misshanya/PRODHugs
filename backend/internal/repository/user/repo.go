package user

import (
	"time"

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
	var bannedAt *time.Time
	if u.BannedAt.Valid {
		bannedAt = &u.BannedAt.Time
	}
	var createdAt *time.Time
	if u.CreatedAt.Valid {
		createdAt = &u.CreatedAt.Time
	}
	return &models.User{
		ID:             u.ID,
		Username:       u.Username,
		Role:           u.Role,
		HashedPassword: u.Password,
		Gender:         gender,
		BannedAt:       bannedAt,
		CreatedAt:      createdAt,
	}
}

func toAdminUser(u storage.ListUsersAdminRow) *models.AdminUser {
	var gender *string
	if u.Gender.Valid {
		gender = &u.Gender.String
	}
	var bannedAt *time.Time
	if u.BannedAt.Valid {
		bannedAt = &u.BannedAt.Time
	}
	var createdAt *time.Time
	if u.CreatedAt.Valid {
		createdAt = &u.CreatedAt.Time
	}
	return &models.AdminUser{
		ID:        u.ID,
		Username:  u.Username,
		Role:      u.Role,
		Gender:    gender,
		BannedAt:  bannedAt,
		CreatedAt: createdAt,
		Balance:   u.Balance,
	}
}
