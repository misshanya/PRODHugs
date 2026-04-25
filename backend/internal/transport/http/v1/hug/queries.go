package hug

import (
	"context"
	"errors"
	"go-service-template/internal/errorz"
	"go-service-template/internal/transport/http/middleware"
	v1 "go-service-template/internal/transport/http/v1"

	"github.com/google/uuid"
)

func (h *HugHandler) GetBalance(ctx context.Context, req v1.GetBalanceRequestObject) (v1.GetBalanceResponseObject, error) {
	userID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)

	bal, err := h.svc.GetBalance(ctx, userID)
	if err != nil {
		return nil, err
	}

	return v1.GetBalance200JSONResponse{
		UserId: bal.UserID,
		Amount: int(bal.Amount),
	}, nil
}

func (h *HugHandler) ClaimDailyReward(ctx context.Context, req v1.ClaimDailyRewardRequestObject) (v1.ClaimDailyRewardResponseObject, error) {
	userID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)

	amount, streak, newBalance, alreadyClaimed, err := h.svc.ClaimDailyReward(ctx, userID)
	if err != nil {
		return nil, err
	}

	return v1.ClaimDailyReward200JSONResponse{
		Amount:         int(amount),
		StreakDays:     int(streak),
		NewBalance:     int(newBalance),
		AlreadyClaimed: &alreadyClaimed,
	}, nil
}

func (h *HugHandler) GetHugHistory(ctx context.Context, req v1.GetHugHistoryRequestObject) (v1.GetHugHistoryResponseObject, error) {
	userID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)

	hugs, err := h.svc.GetHugHistory(ctx, userID)
	if err != nil {
		return nil, err
	}

	result := make(v1.GetHugHistory200JSONResponse, len(hugs))
	for i, hg := range hugs {
		result[i] = v1.Hug{
			Id:         hg.ID,
			GiverId:    hg.GiverID,
			ReceiverId: hg.ReceiverID,
			CreatedAt:  hg.CreatedAt,
		}
	}

	return result, nil
}

func (h *HugHandler) GetHugsFeed(ctx context.Context, req v1.GetHugsFeedRequestObject) (v1.GetHugsFeedResponseObject, error) {
	limit := int32(50)
	if req.Params.Limit != nil && *req.Params.Limit > 0 {
		limit = int32(*req.Params.Limit)
	}

	items, err := h.svc.GetRecentFeed(ctx, limit)
	if err != nil {
		return nil, err
	}

	result := make(v1.GetHugsFeed200JSONResponse, len(items))
	for i, item := range items {
		result[i] = v1.HugFeedItem{
			Id:               item.ID,
			GiverId:          item.GiverID,
			ReceiverId:       item.ReceiverID,
			GiverUsername:    item.GiverUsername,
			ReceiverUsername: item.ReceiverUsername,
			CreatedAt:        item.CreatedAt,
		}
	}

	return result, nil
}

func (h *HugHandler) GetLeaderboard(ctx context.Context, req v1.GetLeaderboardRequestObject) (v1.GetLeaderboardResponseObject, error) {
	limit := int32(20)
	offset := int32(0)
	if req.Params.Limit != nil && *req.Params.Limit > 0 {
		limit = int32(*req.Params.Limit)
	}
	if req.Params.Offset != nil && *req.Params.Offset >= 0 {
		offset = int32(*req.Params.Offset)
	}

	entries, err := h.svc.GetLeaderboard(ctx, limit, offset)
	if err != nil {
		return nil, err
	}

	result := make(v1.GetLeaderboard200JSONResponse, len(entries))
	for i, e := range entries {
		result[i] = v1.LeaderboardEntry{
			UserId:       e.UserID,
			Username:     e.Username,
			TotalHugs:    int(e.TotalHugs),
			HugsGiven:    int(e.HugsGiven),
			HugsReceived: int(e.HugsReceived),
			Rank:         e.Rank,
		}
	}

	return result, nil
}

func (h *HugHandler) GetUserProfile(ctx context.Context, req v1.GetUserProfileRequestObject) (v1.GetUserProfileResponseObject, error) {
	user, stats, bal, err := h.svc.GetUserProfile(ctx, req.UserId)
	if err != nil {
		if errors.Is(err, errorz.ErrUserNotFound) {
			return v1.GetUserProfile404JSONResponse{
				NotFoundJSONResponse: v1.NotFoundJSONResponse{
					Code:    v1.USERNOTFOUND,
					Message: "User not found",
				},
			}, nil
		}
		return nil, err
	}

	balAmount := int(bal.Amount)
	return v1.GetUserProfile200JSONResponse{
		Id:           user.ID,
		Username:     user.Username,
		Role:         user.Role,
		HugsGiven:    int(stats.HugsGiven),
		HugsReceived: int(stats.HugsReceived),
		TotalHugs:    int(stats.TotalHugs),
		Rank:         stats.Rank,
		Balance:      &balAmount,
	}, nil
}

func (h *HugHandler) SearchUsers(ctx context.Context, req v1.SearchUsersRequestObject) (v1.SearchUsersResponseObject, error) {
	query := ""
	limit := int32(20)
	offset := int32(0)

	if req.Params.Q != nil {
		query = *req.Params.Q
	}
	if req.Params.Limit != nil && *req.Params.Limit > 0 {
		limit = int32(*req.Params.Limit)
	}
	if req.Params.Offset != nil && *req.Params.Offset >= 0 {
		offset = int32(*req.Params.Offset)
	}

	users, err := h.svc.SearchUsers(ctx, query, limit, offset)
	if err != nil {
		return nil, err
	}

	result := make(v1.SearchUsers200JSONResponse, len(users))
	for i, u := range users {
		result[i] = v1.UserListItem{
			Id:       u.ID,
			Username: u.Username,
			Role:     u.Role,
		}
	}

	return result, nil
}
