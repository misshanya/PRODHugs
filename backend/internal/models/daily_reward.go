package models

import (
	"time"

	"github.com/google/uuid"
)

type DailyReward struct {
	UserID        uuid.UUID
	LastClaimedAt time.Time
	StreakDays    int32
}

// Today returns today's date in UTC as a string (YYYY-MM-DD)
func Today() string {
	return time.Now().UTC().Format("2006-01-02")
}
