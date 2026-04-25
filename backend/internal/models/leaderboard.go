package models

import "github.com/google/uuid"

type LeaderboardEntry struct {
	UserID       uuid.UUID
	Username     string
	Role         string
	TotalHugs    int32
	HugsGiven   int64
	HugsReceived int64
	Rank         string
}

type UserStats struct {
	HugsGiven    int64
	HugsReceived int64
	TotalHugs    int32
	Rank         string
}

// Rank thresholds
func GetRank(totalHugs int32) string {
	switch {
	case totalHugs >= 1000:
		return "Бог объятий"
	case totalHugs >= 500:
		return "Легенда"
	case totalHugs >= 200:
		return "Мастер объятий"
	case totalHugs >= 50:
		return "Дружелюбный"
	case totalHugs >= 10:
		return "Обнимашка"
	default:
		return "Новичок"
	}
}
