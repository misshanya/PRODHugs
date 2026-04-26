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

func (s *service) GetUserProfile(ctx context.Context, userID uuid.UUID) (*models.User, *models.UserStats, *models.Balance, error) {
	user, err := s.userRepo.GetByID(ctx, userID)
	if err != nil {
		return nil, nil, nil, err
	}

	stats, err := s.hugRepo.GetUserStats(ctx, userID)
	if err != nil {
		return nil, nil, nil, err
	}

	balance, err := s.balanceRepo.GetBalance(ctx, userID)
	if err != nil {
		return nil, nil, nil, err
	}

	return user, stats, balance, nil
}

func (s *service) SearchUsers(ctx context.Context, query string, limit, offset int32) ([]*models.User, error) {
	return s.hugRepo.SearchUsers(ctx, query, limit, offset)
}

func (s *service) ClaimDailyReward(ctx context.Context, userID uuid.UUID) (int32, int32, int32, bool, error) {
	// Check if already claimed today
	existing, err := s.dailyRepo.GetDailyReward(ctx, userID)
	if err != nil {
		return 0, 0, 0, false, err
	}

	if existing != nil && existing.LastClaimedAt.UTC().Format("2006-01-02") == models.Today() {
		// Already claimed today
		bal, err := s.balanceRepo.GetBalance(ctx, userID)
		if err != nil {
			return 0, 0, 0, true, err
		}
		return 0, existing.StreakDays, bal.Amount, true, nil
	}

	// Claim
	reward, err := s.dailyRepo.ClaimDailyReward(ctx, userID)
	if err != nil {
		return 0, 0, 0, false, err
	}

	// Calculate reward amount: base 5 + min(streak-1, 5)
	bonus := reward.StreakDays - 1
	if bonus > 5 {
		bonus = 5
	}
	amount := int32(5) + bonus

	bal, err := s.balanceRepo.AddBalance(ctx, userID, amount)
	if err != nil {
		return 0, 0, 0, false, err
	}

	return amount, reward.StreakDays, bal.Amount, false, nil
}
