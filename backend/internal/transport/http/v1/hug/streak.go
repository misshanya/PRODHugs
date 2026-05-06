package hug

import (
	"context"

	"go-service-template/internal/transport/http/middleware"
	v1 "go-service-template/internal/transport/http/v1"

	openapi_types "github.com/oapi-codegen/runtime/types"

	"github.com/google/uuid"
)

func (h *HugHandler) GetPairStreak(ctx context.Context, req v1.GetPairStreakRequestObject) (v1.GetPairStreakResponseObject, error) {
	userA := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)
	userB := uuid.UUID(req.UserId)

	streak, err := h.svc.GetPairStreak(ctx, userA, userB)
	if err != nil {
		return nil, err
	}

	calendar, err := h.svc.GetPairStreakCalendar(ctx, userA, userB)
	if err != nil {
		return nil, err
	}

	calendarItems := make([]v1.StreakCalendarDay, len(calendar))
	for i, day := range calendar {
		calendarItems[i] = v1.StreakCalendarDay{
			Date:      openapi_types.Date{Time: day.Date},
			HugCount:  int(day.HugCount),
			Completed: day.Completed,
		}
	}

	return v1.GetPairStreak200JSONResponse{
		Streak: v1.StreakInfo{
			CurrentStreak: int(streak.CurrentStreak),
			BestStreak:    int(streak.BestStreak),
			TierLevel:     streak.TierLevel,
			TierName:      streak.TierName,
			TierKey:       streak.TierKey,
			NextTierAt:    streak.NextTierAt,
			AHuggedToday:  streak.AHuggedToday,
			BHuggedToday:  streak.BHuggedToday,
		},
		Calendar: calendarItems,
	}, nil
}

func (h *HugHandler) GetTopStreaks(ctx context.Context, _ v1.GetTopStreaksRequestObject) (v1.GetTopStreaksResponseObject, error) {
	userID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)

	entries, err := h.svc.GetUserTopStreaks(ctx, userID, 3)
	if err != nil {
		return nil, err
	}

	result := make(v1.GetTopStreaks200JSONResponse, len(entries))
	for i, e := range entries {
		item := v1.TopStreakEntry{
			UserId:        e.UserID,
			Username:      e.Username,
			DisplayName:   e.DisplayName,
			CurrentStreak: int(e.CurrentStreak),
			BestStreak:    int(e.BestStreak),
			TierLevel:     e.TierLevel,
			TierName:      e.TierName,
			TierKey:       e.TierKey,
		}
		if e.Gender != nil {
			g := v1.Gender(*e.Gender)
			item.Gender = &g
		}
		result[i] = item
	}

	return result, nil
}
