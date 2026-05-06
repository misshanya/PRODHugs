package hug

import (
	"context"
	"time"

	"go-service-template/internal/db/sqlc/storage"
	"go-service-template/internal/models"
	"go-service-template/internal/repository"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgtype"
)

func (r *repo) GetPairStreak(ctx context.Context, userA, userB uuid.UUID) (*models.PairStreak, error) {
	q := repository.Queries(ctx, r.q)

	row, err := q.GetPairStreak(ctx, storage.GetPairStreakParams{
		UserA: userA,
		UserB: userB,
	})
	if err != nil {
		if err == pgx.ErrNoRows {
			return nil, nil
		}
		return nil, err
	}

	return toModelPairStreak(row.UserAID, row.UserBID, row.CurrentStreak, row.BestStreak,
		row.LastStreakDate, row.AHuggedToday, row.BHuggedToday, row.TodayDate), nil
}

func (r *repo) UpsertPairStreak(ctx context.Context, streak *models.PairStreak) (*models.PairStreak, error) {
	q := repository.Queries(ctx, r.q)

	params := storage.UpsertPairStreakParams{
		UserA:         streak.UserAID,
		UserB:         streak.UserBID,
		CurrentStreak: streak.CurrentStreak,
		BestStreak:    streak.BestStreak,
		AHuggedToday:  streak.AHuggedToday,
		BHuggedToday:  streak.BHuggedToday,
		TodayDate:     pgtype.Date{Time: streak.TodayDate, Valid: true},
	}
	if streak.LastStreakDate != nil {
		params.LastStreakDate = pgtype.Date{Time: *streak.LastStreakDate, Valid: true}
	}

	row, err := q.UpsertPairStreak(ctx, params)
	if err != nil {
		return nil, err
	}

	return toModelPairStreak(row.UserAID, row.UserBID, row.CurrentStreak, row.BestStreak,
		row.LastStreakDate, row.AHuggedToday, row.BHuggedToday, row.TodayDate), nil
}

func (r *repo) GetUserTopStreaks(ctx context.Context, userID uuid.UUID, limit int32) ([]*models.TopStreakEntry, error) {
	q := repository.Queries(ctx, r.q)

	rows, err := q.GetUserTopStreaks(ctx, storage.GetUserTopStreaksParams{
		UserID: userID,
		Lim:    limit,
	})
	if err != nil {
		return nil, err
	}

	today := time.Now().UTC().Truncate(24 * time.Hour)

	result := make([]*models.TopStreakEntry, 0, len(rows))
	for _, row := range rows {
		streak := evaluateStreakFreshness(row.CurrentStreak, row.BestStreak,
			row.LastStreakDate, row.AHuggedToday, row.BHuggedToday, row.TodayDate, today)

		// Skip entries that are no longer active after freshness evaluation
		if streak.CurrentStreak <= 0 {
			continue
		}

		tier := models.ComputeStreakTier(streak.CurrentStreak)

		var otherUserID uuid.UUID
		if row.UserAID == userID {
			otherUserID = row.UserBID
		} else {
			otherUserID = row.UserAID
		}

		var displayName *string
		if row.OtherDisplayName.Valid {
			displayName = &row.OtherDisplayName.String
		}
		var gender *string
		if row.OtherGender.Valid {
			gender = &row.OtherGender.String
		}

		result = append(result, &models.TopStreakEntry{
			UserID:        otherUserID,
			Username:      row.OtherUsername,
			DisplayName:   displayName,
			Gender:        gender,
			CurrentStreak: streak.CurrentStreak,
			BestStreak:    streak.BestStreak,
			TierLevel:     tier.Level,
			TierName:      tier.Name,
			TierKey:       tier.Key,
		})
	}

	return result, nil
}

func (r *repo) GetPairStreakCalendar(ctx context.Context, userA, userB uuid.UUID, since time.Time) ([]*models.StreakCalendarDay, error) {
	q := repository.Queries(ctx, r.q)

	rows, err := q.GetPairStreakCalendar(ctx, storage.GetPairStreakCalendarParams{
		UserA: userA,
		UserB: userB,
		Since: pgtype.Timestamptz{Time: since, Valid: true},
	})
	if err != nil {
		return nil, err
	}

	result := make([]*models.StreakCalendarDay, len(rows))
	for i, row := range rows {
		result[i] = &models.StreakCalendarDay{
			Date:      row.HugDate.Time,
			HugCount:  row.HugCount,
			Completed: row.Completed,
		}
	}
	return result, nil
}

// toModelPairStreak converts DB types to domain PairStreak.
func toModelPairStreak(userAID, userBID uuid.UUID, currentStreak, bestStreak int32,
	lastStreakDate pgtype.Date, aHuggedToday, bHuggedToday bool, todayDate pgtype.Date) *models.PairStreak {
	ps := &models.PairStreak{
		UserAID:       userAID,
		UserBID:       userBID,
		CurrentStreak: currentStreak,
		BestStreak:    bestStreak,
		AHuggedToday:  aHuggedToday,
		BHuggedToday:  bHuggedToday,
		TodayDate:     todayDate.Time,
	}
	if lastStreakDate.Valid {
		t := lastStreakDate.Time
		ps.LastStreakDate = &t
	}
	return ps
}

// evaluateStreakFreshness checks if the streak data is stale and computes the effective current values.
// This implements lazy evaluation: if today_date in the DB is not today, we figure out what really happened.
func evaluateStreakFreshness(currentStreak, bestStreak int32, lastStreakDate pgtype.Date,
	aHuggedToday, bHuggedToday bool, todayDate pgtype.Date, today time.Time) struct {
	CurrentStreak int32
	BestStreak    int32
} {
	dbToday := todayDate.Time
	// If the record's today_date is actually today, return as-is
	if dbToday.Equal(today) {
		return struct {
			CurrentStreak int32
			BestStreak    int32
		}{currentStreak, bestStreak}
	}

	// The record is stale. Check if the previous day (todayDate) was completed.
	yesterday := today.AddDate(0, 0, -1)
	if aHuggedToday && bHuggedToday && dbToday.Equal(yesterday) {
		// Previous day was completed and it was yesterday — streak continues
		return struct {
			CurrentStreak int32
			BestStreak    int32
		}{currentStreak, bestStreak}
	}

	// If last_streak_date was yesterday, the streak is still valid (just no hugs today yet)
	if lastStreakDate.Valid && lastStreakDate.Time.Equal(yesterday) {
		return struct {
			CurrentStreak int32
			BestStreak    int32
		}{currentStreak, bestStreak}
	}

	// If last_streak_date was today, streak is valid
	if lastStreakDate.Valid && lastStreakDate.Time.Equal(today) {
		return struct {
			CurrentStreak int32
			BestStreak    int32
		}{currentStreak, bestStreak}
	}

	// Otherwise streak is broken
	return struct {
		CurrentStreak int32
		BestStreak    int32
	}{0, bestStreak}
}
