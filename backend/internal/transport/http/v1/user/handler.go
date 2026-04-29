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
	GetByUsername(ctx context.Context, username string) (*models.User, error)
	UpdateSettings(ctx context.Context, id uuid.UUID, gender *string, displayName *string) (*models.User, error)
	ChangePassword(ctx context.Context, id uuid.UUID, oldPassword, newPassword string) error
	SendTelegramCode(ctx context.Context, userID uuid.UUID, telegramID int64) error
	VerifyTelegramCode(ctx context.Context, userID uuid.UUID, telegramID int64, code string) (*models.User, error)
	UnlinkTelegram(ctx context.Context, userID uuid.UUID) (*models.User, error)
	SaveRefreshToken(ctx context.Context, jti string, userID uuid.UUID, expiresAtUnix int64) error
	IsRefreshTokenActive(ctx context.Context, jti string) (bool, error)
	RevokeRefreshToken(ctx context.Context, jti string) error
	RevokeAllUserRefreshTokens(ctx context.Context, userID uuid.UUID) error
}

type UserHandler struct {
	svc          service
	jwtManager   *jwt.Manager
	cookieSecure bool
}

func New(svc service, jwtManager *jwt.Manager, cookieSecure bool) *UserHandler {
	return &UserHandler{svc: svc, jwtManager: jwtManager, cookieSecure: cookieSecure}
}

func toV1User(u *models.User) v1.User {
	user := v1.User{
		Id:          u.ID,
		Username:    u.Username,
		Role:        v1.UserRole(u.Role),
		DisplayName: u.DisplayName,
		TelegramId:  u.TelegramID,
	}
	if u.Gender != nil {
		g := v1.Gender(*u.Gender)
		user.Gender = &g
	}
	return user
}
