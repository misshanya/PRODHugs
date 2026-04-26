package user

import (
	"context"
	"go-service-template/internal/jwt"
	"go-service-template/internal/models"
	v1 "go-service-template/internal/transport/http/v1"

	"github.com/google/uuid"
)

type service interface {
	Create(ctx context.Context, input *models.CreateUser) (*models.User, string, string, error)
	Login(ctx context.Context, username string, password string) (*models.User, string, string, error)
	GetByID(ctx context.Context, id uuid.UUID) (*models.User, error)
	UpdateSettings(ctx context.Context, id uuid.UUID, gender *string) (*models.User, error)
	ChangePassword(ctx context.Context, id uuid.UUID, oldPassword, newPassword string) error
}

type UserHandler struct {
	svc        service
	jwtManager *jwt.Manager
}

func New(svc service, jwtManager *jwt.Manager) *UserHandler {
	return &UserHandler{svc: svc, jwtManager: jwtManager}
}

func toV1User(u *models.User) v1.User {
	user := v1.User{
		Id:       u.ID,
		Username: u.Username,
		Role:     v1.UserRole(u.Role),
	}
	if u.Gender != nil {
		g := v1.Gender(*u.Gender)
		user.Gender = &g
	}
	return user
}
