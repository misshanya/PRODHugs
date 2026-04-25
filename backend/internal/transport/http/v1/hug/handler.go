package hug

import (
	"context"
	"go-service-template/internal/models"

	"github.com/google/uuid"
)

type service interface {
	SendHug(ctx context.Context, giverID, receiverID uuid.UUID) (*models.Hug, error)
	GetCooldownInfo(ctx context.Context, giverID, receiverID uuid.UUID) (*models.HugCooldown, int32, bool, error)
	UpgradeCooldown(ctx context.Context, giverID, receiverID uuid.UUID) (*models.HugCooldown, error)
	GetBalance(ctx context.Context, userID uuid.UUID) (*models.Balance, error)
	GetHugHistory(ctx context.Context, userID uuid.UUID) ([]*models.Hug, error)
	GetRecentFeed(ctx context.Context, limit int32) ([]*models.HugFeedItem, error)
	GetLeaderboard(ctx context.Context, limit, offset int32) ([]*models.LeaderboardEntry, error)
	GetUserStats(ctx context.Context, userID uuid.UUID) (*models.UserStats, error)
	GetUserProfile(ctx context.Context, userID uuid.UUID) (*models.User, *models.UserStats, *models.Balance, error)
	SearchUsers(ctx context.Context, query string, limit, offset int32) ([]*models.User, error)
	ClaimDailyReward(ctx context.Context, userID uuid.UUID) (int32, int32, int32, bool, error)
}

type HugHandler struct {
	svc service
}

func New(svc service) *HugHandler {
	return &HugHandler{svc: svc}
}
