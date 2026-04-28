package hug

import (
	"context"
	"go-service-template/internal/models"

	"github.com/google/uuid"
)

func (s *service) GetBalance(ctx context.Context, userID uuid.UUID) (*models.Balance, error) {
	return s.balanceRepo.GetBalance(ctx, userID)
}

func (s *service) GetHugHistory(ctx context.Context, userID uuid.UUID) ([]*models.HugFeedItem, error) {
	return s.hugRepo.ListHugsByUser(ctx, userID)
}

func (s *service) GetRecentFeed(ctx context.Context, limit int32) ([]*models.HugFeedItem, error) {
	return s.hugRepo.GetRecentFeed(ctx, limit)
}

func (s *service) GetHugActivity(ctx context.Context) ([]*models.HugActivityItem, error) {
	return s.hugRepo.GetHugActivity(ctx)
}

func (s *service) GetLeaderboard(ctx context.Context, limit, offset int32) ([]*models.LeaderboardEntry, error) {
	return s.hugRepo.GetLeaderboard(ctx, limit, offset)
}

func (s *service) GetUserStats(ctx context.Context, userID uuid.UUID) (*models.UserStats, error) {
	return s.hugRepo.GetUserStats(ctx, userID)
}

func (s *service) GetUserProfile(ctx context.Context, userID uuid.UUID, viewerID *uuid.UUID) (*models.User, *models.UserStats, *models.Balance, *models.MutualHugStats, error) {
	user, err := s.userRepo.GetByID(ctx, userID)
	if err != nil {
		return nil, nil, nil, nil, err
	}

	stats, err := s.hugRepo.GetUserStats(ctx, userID)
	if err != nil {
		return nil, nil, nil, nil, err
	}

	balance, err := s.balanceRepo.GetBalance(ctx, userID)
	if err != nil {
		return nil, nil, nil, nil, err
	}

	var mutual *models.MutualHugStats
	if viewerID != nil && *viewerID != userID {
		mutual, err = s.hugRepo.CountMutualHugs(ctx, userID, *viewerID)
		if err != nil {
			return nil, nil, nil, nil, err
		}
	}

	return user, stats, balance, mutual, nil
}

func (s *service) SearchUsers(ctx context.Context, query string, limit, offset int32) ([]*models.User, error) {
	return s.hugRepo.SearchUsers(ctx, query, limit, offset)
}

func (s *service) GetPendingInbox(ctx context.Context, userID uuid.UUID) ([]*models.PendingHugInboxItem, error) {
	return s.hugRepo.GetPendingHugsForUser(ctx, userID)
}

func (s *service) GetOutgoingPendingHug(ctx context.Context, userID uuid.UUID) (*models.OutgoingPendingHug, error) {
	return s.hugRepo.GetOutgoingPendingHug(ctx, userID)
}

func (s *service) GetInboxCount(ctx context.Context, userID uuid.UUID) (int64, error) {
	return s.hugRepo.CountPendingHugsForUser(ctx, userID)
}

func (s *service) ClaimDailyReward(ctx context.Context, userID uuid.UUID) (int32, int32, int32, bool, error) {
	var (
		amount       int32
		streakDays   int32
		balAmount    int32
		alreadyClaimed bool
	)

	// Wrap check + claim + balance update in a transaction to prevent double-claiming.
	err := s.tx.RunInTx(ctx, func(txCtx context.Context) error {
		// Check if already claimed today
		existing, err := s.dailyRepo.GetDailyReward(txCtx, userID)
		if err != nil {
			return err
		}

		if existing != nil && existing.LastClaimedAt.UTC().Format("2006-01-02") == models.Today() {
			// Already claimed today
			bal, err := s.balanceRepo.GetBalance(txCtx, userID)
			if err != nil {
				return err
			}
			alreadyClaimed = true
			streakDays = existing.StreakDays
			balAmount = bal.Amount
			return nil
		}

		// Claim
		reward, err := s.dailyRepo.ClaimDailyReward(txCtx, userID)
		if err != nil {
			return err
		}

		// Calculate reward amount: base 5 + min(streak-1, 5)
		bonus := reward.StreakDays - 1
		if bonus > 5 {
			bonus = 5
		}
		amount = int32(5) + bonus

		bal, err := s.balanceRepo.AddBalance(txCtx, userID, amount)
		if err != nil {
			return err
		}

		streakDays = reward.StreakDays
		balAmount = bal.Amount
		return nil
	})
	if err != nil {
		return 0, 0, 0, false, err
	}

	return amount, streakDays, balAmount, alreadyClaimed, nil
}
