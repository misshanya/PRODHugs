package models

import "github.com/google/uuid"

type LeaderboardEntry struct {
	UserID       uuid.UUID
	Username     string
	DisplayName  *string
	Role         string
	TotalHugs    int32
	HugsGiven    int64
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
		return "Милашка"
	case totalHugs >= 500:
		return "Легенда"
	case totalHugs >= 200:
		return "Обнимастер"
	case totalHugs >= 50:
		return "Тактильный"
	case totalHugs >= 10:
		return "Неопытный"
	default:
		return "Нетактильный"
	}
}
