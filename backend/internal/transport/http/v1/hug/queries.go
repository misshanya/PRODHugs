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

	// Default: last 100 hugs. The endpoint doesn't currently expose pagination
	// params in the OpenAPI spec, but the service/repo now support it.
	hugs, err := h.svc.GetHugHistory(ctx, userID, 100, 0)
	if err != nil {
		return nil, err
	}

	result := make(v1.GetHugHistory200JSONResponse, len(hugs))
	for i, hg := range hugs {
		item := v1.HugFeedItem{
			Id:               hg.ID,
			GiverId:          hg.GiverID,
			ReceiverId:       hg.ReceiverID,
			GiverUsername:    hg.GiverUsername,
			ReceiverUsername: hg.ReceiverUsername,
			CreatedAt:        hg.CreatedAt,
		}
		if hg.GiverGender != nil {
			g := v1.Gender(*hg.GiverGender)
			item.GiverGender = &g
		}
		result[i] = item
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
	for i, it := range items {
		fi := v1.HugFeedItem{
			Id:               it.ID,
			GiverId:          it.GiverID,
			ReceiverId:       it.ReceiverID,
			GiverUsername:    it.GiverUsername,
			ReceiverUsername: it.ReceiverUsername,
			CreatedAt:        it.CreatedAt,
		}
		if it.GiverGender != nil {
			g := v1.Gender(*it.GiverGender)
			fi.GiverGender = &g
		}
		result[i] = fi
	}

	return result, nil
}

func (h *HugHandler) GetHugActivity(ctx context.Context, _ v1.GetHugActivityRequestObject) (v1.GetHugActivityResponseObject, error) {
	items, err := h.svc.GetHugActivity(ctx)
	if err != nil {
		return nil, err
	}

	result := make(v1.GetHugActivity200JSONResponse, len(items))
	for i, item := range items {
		result[i] = v1.HugActivityItem{
			Timestamp: item.Timestamp,
			Count:     int(item.Count),
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
	viewerID, _ := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)
	var viewerPtr *uuid.UUID
	if viewerID != uuid.Nil {
		viewerPtr = &viewerID
	}

	user, stats, bal, mutual, isBlocked, err := h.svc.GetUserProfile(ctx, req.UserId, viewerPtr)
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
	resp := v1.GetUserProfile200JSONResponse{
		Id:           user.ID,
		Username:     user.Username,
		Role:         user.Role,
		HugsGiven:    int(stats.HugsGiven),
		HugsReceived: int(stats.HugsReceived),
		TotalHugs:    int(stats.TotalHugs),
		Rank:         stats.Rank,
		Balance:      &balAmount,
	}
	if user.Gender != nil {
		g := v1.Gender(*user.Gender)
		resp.Gender = &g
	}
	if mutual != nil {
		mt := int(mutual.Total)
		mg := int(mutual.Given)
		mr := int(mutual.Received)
		resp.MutualTotal = &mt
		resp.MutualGiven = &mg
		resp.MutualReceived = &mr
	}
	if isBlocked {
		resp.IsBlocked = &isBlocked
	}
	return resp, nil
}

func (h *HugHandler) SearchUsers(ctx context.Context, req v1.SearchUsersRequestObject) (v1.SearchUsersResponseObject, error) {
	viewerID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)
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

	users, err := h.svc.SearchUsers(ctx, query, viewerID, limit, offset)
	if err != nil {
		return nil, err
	}

	result := make(v1.SearchUsers200JSONResponse, len(users))
	for i, u := range users {
		item := v1.UserListItem{
			Id:       u.ID,
			Username: u.Username,
			Role:     u.Role,
		}
		if u.Gender != nil {
			g := v1.Gender(*u.Gender)
			item.Gender = &g
		}
		result[i] = item
	}

	return result, nil
}
