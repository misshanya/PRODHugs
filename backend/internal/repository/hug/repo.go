package hug

import (
	"go-service-template/internal/db/sqlc/storage"
	"go-service-template/internal/models"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgxpool"
)

type repo struct {
	q *storage.Queries
}

func New(db *pgxpool.Pool) *repo {
	return &repo{
		q: storage.New(db),
	}
}

func toModelHug(h storage.Hug) *models.Hug {
	return &models.Hug{
		ID:         h.ID,
		GiverID:    h.GiverID,
		ReceiverID: h.ReceiverID,
		CreatedAt:  h.CreatedAt.Time,
	}
}

func toModelCooldown(c storage.HugCooldown) *models.HugCooldown {
	return &models.HugCooldown{
		GiverID:         c.GiverID,
		ReceiverID:      c.ReceiverID,
		LastHugAt:       c.LastHugAt.Time,
		CooldownSeconds: c.CooldownSeconds,
	}
}

func toModelFeedItem(row storage.GetRecentHugsFeedRow) *models.HugFeedItem {
	return &models.HugFeedItem{
		ID:               row.ID,
		GiverID:          row.GiverID,
		ReceiverID:       row.ReceiverID,
		GiverUsername:    row.GiverUsername,
		ReceiverUsername: row.ReceiverUsername,
		CreatedAt:        row.CreatedAt.Time,
	}
}

func toModelLeaderboardEntry(row storage.GetLeaderboardRow) *models.LeaderboardEntry {
	return &models.LeaderboardEntry{
		UserID:       row.ID,
		Username:     row.Username,
		Role:         row.Role,
		TotalHugs:    row.TotalHugs,
		HugsGiven:    row.HugsGiven,
		HugsReceived: row.HugsReceived,
		Rank:         models.GetRank(int32(row.TotalHugs)),
	}
}

func toModelUserStats(row storage.GetUserStatsRow) *models.UserStats {
	return &models.UserStats{
		HugsGiven:    row.HugsGiven,
		HugsReceived: row.HugsReceived,
		TotalHugs:    row.TotalHugs,
		Rank:         models.GetRank(int32(row.TotalHugs)),
	}
}

func toModelUserListItem(row storage.SearchUsersRow) *models.User {
	return &models.User{
		ID:       row.ID,
		Username: row.Username,
		Role:     row.Role,
	}
}

func toModelUserListItemFromAll(row storage.ListAllUsersRow) *models.User {
	return &models.User{
		ID:       row.ID,
		Username: row.Username,
		Role:     row.Role,
	}
}

// placeholder to satisfy uuid import
var _ = uuid.Nil
