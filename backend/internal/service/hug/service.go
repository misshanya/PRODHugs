package hug

import (
	"context"
	"go-service-template/internal/models"

	"github.com/google/uuid"
)

type hugRepo interface {
	InsertHug(ctx context.Context, giverID, receiverID uuid.UUID) (*models.Hug, error)
	ListHugsByUser(ctx context.Context, userID uuid.UUID) ([]*models.HugFeedItem, error)
	GetCooldown(ctx context.Context, giverID, receiverID uuid.UUID) (*models.HugCooldown, error)
	UpsertCooldown(ctx context.Context, giverID, receiverID uuid.UUID, cooldownSeconds int32) (*models.HugCooldown, error)
	ReduceCooldown(ctx context.Context, giverID, receiverID uuid.UUID, reduction int32) (*models.HugCooldown, error)
	GetRecentFeed(ctx context.Context, limit int32) ([]*models.HugFeedItem, error)
	GetHugActivity(ctx context.Context) ([]*models.HugActivityItem, error)
	GetLeaderboard(ctx context.Context, limit, offset int32) ([]*models.LeaderboardEntry, error)
	GetUserStats(ctx context.Context, userID uuid.UUID) (*models.UserStats, error)
	SearchUsers(ctx context.Context, query string, limit, offset int32) ([]*models.User, error)
}

type balanceRepo interface {
	GetBalance(ctx context.Context, userID uuid.UUID) (*models.Balance, error)
	AddBalance(ctx context.Context, userID uuid.UUID, delta int32) (*models.Balance, error)
	DeductBalance(ctx context.Context, userID uuid.UUID, delta int32) (*models.Balance, error)
	EnsureBalance(ctx context.Context, userID uuid.UUID) error
}

type dailyRewardRepo interface {
	GetDailyReward(ctx context.Context, userID uuid.UUID) (*models.DailyReward, error)
	ClaimDailyReward(ctx context.Context, userID uuid.UUID) (*models.DailyReward, error)
}

type userRepo interface {
	GetByID(ctx context.Context, id uuid.UUID) (*models.User, error)
}

// HugEventCallback is called when a hug is sent, for WebSocket broadcasting
type HugEventCallback func(item *models.HugFeedItem)

type service struct {
	hugRepo        hugRepo
	balanceRepo    balanceRepo
	dailyRepo      dailyRewardRepo
	userRepo       userRepo
	onHugCallback  HugEventCallback
}

func New(hugRepo hugRepo, balanceRepo balanceRepo, dailyRepo dailyRewardRepo, userRepo userRepo) *service {
	return &service{
		hugRepo:     hugRepo,
		balanceRepo: balanceRepo,
		dailyRepo:   dailyRepo,
		userRepo:    userRepo,
	}
}

func (s *service) SetHugCallback(cb HugEventCallback) {
	s.onHugCallback = cb
}
