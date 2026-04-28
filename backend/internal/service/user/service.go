package user

import (
	"context"
	"go-service-template/internal/models"

	"github.com/google/uuid"
)

type repo interface {
	Create(ctx context.Context, input *models.CreateUser) (*models.User, error)
	GetByUsername(ctx context.Context, username string) (*models.User, error)
	GetByID(ctx context.Context, id uuid.UUID) (*models.User, error)
	UpdateSettings(ctx context.Context, id uuid.UUID, gender *string) (*models.User, error)
	UpdatePassword(ctx context.Context, id uuid.UUID, hashedPassword string) error
	BanUser(ctx context.Context, id uuid.UUID) (*models.User, error)
	UnbanUser(ctx context.Context, id uuid.UUID) (*models.User, error)
	CountUsers(ctx context.Context) (int64, error)
	CountBannedUsers(ctx context.Context) (int64, error)
	ListUsersAdmin(ctx context.Context, limit, offset int32) ([]*models.AdminUser, error)
	AdminUpdateUsername(ctx context.Context, id uuid.UUID, username string) (*models.User, error)
	AdminUpdateGender(ctx context.Context, id uuid.UUID, gender *string) (*models.User, error)
	AdminUpdatePassword(ctx context.Context, id uuid.UUID, hashedPassword string) error
}

type balanceRepo interface {
	AdminSetBalance(ctx context.Context, userID uuid.UUID, amount int32) (*models.Balance, error)
}

type refreshTokenRepo interface {
	SaveRefreshToken(ctx context.Context, jti string, userID uuid.UUID, expiresAtUnix int64) error
	IsRefreshTokenActive(ctx context.Context, jti string) (bool, error)
	RevokeRefreshToken(ctx context.Context, jti string) error
	RevokeAllUserRefreshTokens(ctx context.Context, userID uuid.UUID) error
}

type jwtManager interface {
	GenerateAccessToken(userID uuid.UUID, role string) (string, int64, error)
	GenerateRefreshToken(userID uuid.UUID) (string, string, int64, error)
}

type service struct {
	repo             repo
	balanceRepo      balanceRepo
	refreshTokenRepo refreshTokenRepo
	jwtManager       jwtManager
}

func New(repo repo, jwtManager jwtManager, opts ...func(*service)) *service {
	s := &service{
		repo:       repo,
		jwtManager: jwtManager,
	}
	for _, opt := range opts {
		opt(s)
	}
	return s
}

// WithBalanceRepo sets the balance repository for admin operations.
func WithBalanceRepo(br balanceRepo) func(*service) {
	return func(s *service) {
		s.balanceRepo = br
	}
}

func WithRefreshTokenRepo(rtr refreshTokenRepo) func(*service) {
	return func(s *service) {
		s.refreshTokenRepo = rtr
	}
}
