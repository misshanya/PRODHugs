package hug

import (
	"context"

	"go-service-template/internal/transport/http/middleware"
	v1 "go-service-template/internal/transport/http/v1"

	"github.com/google/uuid"
)

func (h *HugHandler) GetPairIntimacy(ctx context.Context, req v1.GetPairIntimacyRequestObject) (v1.GetPairIntimacyResponseObject, error) {
	userA := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)
	userB := req.UserId

	info, err := h.svc.GetPairIntimacy(ctx, userA, userB)
	if err != nil {
		return nil, err
	}

	hugTypes := make([]v1.HugType, len(info.AvailableHugTypes))
	for i, ht := range info.AvailableHugTypes {
		hugTypes[i] = v1.HugType(ht)
	}

	return v1.GetPairIntimacy200JSONResponse{
		RawScore:             info.RawScore,
		Tier:                 info.Tier,
		TierName:             info.TierName,
		NextTierAt:           info.NextTierAt,
		CooldownReductionPct: info.CooldownReductionPct,
		AvailableHugTypes:    hugTypes,
		BonusCoins:           info.BonusCoins,
	}, nil
}

func (h *HugHandler) GetConnections(ctx context.Context, req v1.GetConnectionsRequestObject) (v1.GetConnectionsResponseObject, error) {
	userID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)

	limit := int32(20)
	offset := int32(0)
	if req.Params.Limit != nil {
		limit = int32(*req.Params.Limit)
	}
	if req.Params.Offset != nil {
		offset = int32(*req.Params.Offset)
	}

	connections, err := h.svc.GetUserConnections(ctx, userID, limit, offset)
	if err != nil {
		return nil, err
	}

	result := make(v1.GetConnections200JSONResponse, len(connections))
	for i, conn := range connections {
		hugTypes := make([]v1.HugType, len(conn.Intimacy.AvailableHugTypes))
		for j, ht := range conn.Intimacy.AvailableHugTypes {
			hugTypes[j] = v1.HugType(ht)
		}

		item := v1.ConnectionItem{
			UserId:   conn.UserID,
			Username: conn.Username,
			Intimacy: v1.IntimacyInfo{
				RawScore:             conn.Intimacy.RawScore,
				Tier:                 conn.Intimacy.Tier,
				TierName:             conn.Intimacy.TierName,
				NextTierAt:           conn.Intimacy.NextTierAt,
				CooldownReductionPct: conn.Intimacy.CooldownReductionPct,
				AvailableHugTypes:    hugTypes,
				BonusCoins:           conn.Intimacy.BonusCoins,
			},
		}
		if conn.DisplayName != nil {
			item.DisplayName = conn.DisplayName
		}
		if conn.Gender != nil {
			g := v1.Gender(*conn.Gender)
			item.Gender = &g
		}
		result[i] = item
	}

	return result, nil
}

func (h *HugHandler) GetIntimacyLeaderboard(ctx context.Context, req v1.GetIntimacyLeaderboardRequestObject) (v1.GetIntimacyLeaderboardResponseObject, error) {
	limit := int32(20)
	offset := int32(0)
	if req.Params.Limit != nil {
		limit = int32(*req.Params.Limit)
	}
	if req.Params.Offset != nil {
		offset = int32(*req.Params.Offset)
	}

	entries, err := h.svc.GetIntimacyLeaderboard(ctx, limit, offset)
	if err != nil {
		return nil, err
	}

	result := make(v1.GetIntimacyLeaderboard200JSONResponse, len(entries))
	for i, e := range entries {
		result[i] = v1.IntimacyLeaderboardEntry{
			UserAId:          e.UserAID,
			UserAUsername:    e.UserAUsername,
			UserADisplayName: e.UserADisplayName,
			UserBId:          e.UserBID,
			UserBUsername:    e.UserBUsername,
			UserBDisplayName: e.UserBDisplayName,
			RawScore:         e.RawScore,
			Tier:             e.Tier,
			TierName:         e.TierName,
		}
	}

	return result, nil
}
