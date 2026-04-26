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
	var giverGender *string
	if row.GiverGender.Valid {
		giverGender = &row.GiverGender.String
	}
	return &models.HugFeedItem{
		ID:               row.ID,
		GiverID:          row.GiverID,
		ReceiverID:       row.ReceiverID,
		GiverUsername:    row.GiverUsername,
		ReceiverUsername: row.ReceiverUsername,
		GiverGender:      giverGender,
		CreatedAt:        row.CreatedAt.Time,
	}
}

func toModelHistoryItem(row storage.ListHugsByUserRow) *models.HugFeedItem {
	var giverGender *string
	if row.GiverGender.Valid {
		giverGender = &row.GiverGender.String
	}
	return &models.HugFeedItem{
		ID:               row.ID,
		GiverID:          row.GiverID,
		ReceiverID:       row.ReceiverID,
		GiverUsername:    row.GiverUsername,
		ReceiverUsername: row.ReceiverUsername,
		GiverGender:      giverGender,
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
	var gender *string
	if row.Gender.Valid {
		gender = &row.Gender.String
	}
	return &models.User{
		ID:       row.ID,
		Username: row.Username,
		Role:     row.Role,
		Gender:   gender,
	}
}

func toModelUserListItemFromAll(row storage.ListAllUsersRow) *models.User {
	var gender *string
	if row.Gender.Valid {
		gender = &row.Gender.String
	}
	return &models.User{
		ID:       row.ID,
		Username: row.Username,
		Role:     row.Role,
		Gender:   gender,
	}
}

// placeholder to satisfy uuid import
var _ = uuid.Nil
