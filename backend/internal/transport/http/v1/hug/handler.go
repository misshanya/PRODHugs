package hug

import (
	"context"
	"go-service-template/internal/models"

	"github.com/google/uuid"
)

type service interface {
	SuggestHug(ctx context.Context, giverID, receiverID uuid.UUID) (*models.Hug, *models.User, error)
	AcceptHug(ctx context.Context, hugID, receiverID uuid.UUID) (*models.Hug, error)
	DeclineHug(ctx context.Context, hugID, receiverID uuid.UUID) error
	CancelHug(ctx context.Context, hugID, giverID uuid.UUID) error
	GetCooldownInfo(ctx context.Context, userA, userB uuid.UUID) (*models.HugCooldown, int32, bool, int32, error)
	UpgradeCooldown(ctx context.Context, payerID, otherUserID uuid.UUID) (*models.HugCooldown, error)
	GetBalance(ctx context.Context, userID uuid.UUID) (*models.Balance, error)
	GetOutgoingHugs(ctx context.Context, userID uuid.UUID) ([]*models.OutgoingPendingHug, *models.SlotInfo, error)
	BuyHugSlot(ctx context.Context, userID uuid.UUID) (*models.SlotInfo, int32, error)
	GetHugHistory(ctx context.Context, userID uuid.UUID, limit, offset int32) ([]*models.HugFeedItem, error)
	GetRecentFeed(ctx context.Context, limit int32) ([]*models.HugFeedItem, error)
	GetHugActivity(ctx context.Context) ([]*models.HugActivityItem, error)
	GetLeaderboard(ctx context.Context, limit, offset int32) ([]*models.LeaderboardEntry, error)
	GetUserStats(ctx context.Context, userID uuid.UUID) (*models.UserStats, error)
	GetUserProfile(ctx context.Context, userID uuid.UUID, viewerID *uuid.UUID) (*models.User, *models.UserStats, *models.Balance, *models.MutualHugStats, bool, error)
	SearchUsers(ctx context.Context, query string, viewerID uuid.UUID, limit, offset int32) ([]*models.User, error)
	ClaimDailyReward(ctx context.Context, userID uuid.UUID) (int32, int32, int32, bool, error)
	GetPendingInbox(ctx context.Context, userID uuid.UUID) ([]*models.PendingHugInboxItem, error)
	GetInboxCount(ctx context.Context, userID uuid.UUID) (int64, error)
	BlockUser(ctx context.Context, blockerID, blockedID uuid.UUID) error
	UnblockUser(ctx context.Context, blockerID, blockedID uuid.UUID) error
	GetBlockedUsers(ctx context.Context, userID uuid.UUID) ([]*models.BlockedUser, error)
}

type HugHandler struct {
	svc service
}

func New(svc service) *HugHandler {
	return &HugHandler{svc: svc}
}
