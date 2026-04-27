package hug

import (
	"context"
	"go-service-template/internal/models"

	"github.com/google/uuid"
)

type service interface {
	SuggestHug(ctx context.Context, giverID, receiverID uuid.UUID) (*models.Hug, error)
	AcceptHug(ctx context.Context, hugID, receiverID uuid.UUID) (*models.Hug, error)
	DeclineHug(ctx context.Context, hugID, receiverID uuid.UUID) error
	CancelHug(ctx context.Context, hugID, giverID uuid.UUID) error
	GetCooldownInfo(ctx context.Context, userA, userB uuid.UUID) (*models.HugCooldown, int32, bool, int32, error)
	UpgradeCooldown(ctx context.Context, payerID, otherUserID uuid.UUID) (*models.HugCooldown, error)
	GetBalance(ctx context.Context, userID uuid.UUID) (*models.Balance, error)
	GetHugHistory(ctx context.Context, userID uuid.UUID) ([]*models.HugFeedItem, error)
	GetRecentFeed(ctx context.Context, limit int32) ([]*models.HugFeedItem, error)
	GetHugActivity(ctx context.Context) ([]*models.HugActivityItem, error)
	GetLeaderboard(ctx context.Context, limit, offset int32) ([]*models.LeaderboardEntry, error)
	GetUserStats(ctx context.Context, userID uuid.UUID) (*models.UserStats, error)
	GetUserProfile(ctx context.Context, userID uuid.UUID, viewerID *uuid.UUID) (*models.User, *models.UserStats, *models.Balance, *models.MutualHugStats, error)
	SearchUsers(ctx context.Context, query string, limit, offset int32) ([]*models.User, error)
	ClaimDailyReward(ctx context.Context, userID uuid.UUID) (int32, int32, int32, bool, error)
	GetPendingInbox(ctx context.Context, userID uuid.UUID) ([]*models.PendingHugInboxItem, error)
	GetOutgoingPendingHug(ctx context.Context, userID uuid.UUID) (*models.OutgoingPendingHug, error)
	GetInboxCount(ctx context.Context, userID uuid.UUID) (int64, error)
}

type HugHandler struct {
	svc service
}

func New(svc service) *HugHandler {
	return &HugHandler{svc: svc}
}
