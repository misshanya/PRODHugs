package hug

import (
	"time"

	"go-service-template/internal/db/sqlc/storage"
	"go-service-template/internal/models"

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
	var acceptedAt *time.Time
	if h.AcceptedAt.Valid {
		t := h.AcceptedAt.Time
		acceptedAt = &t
	}
	return &models.Hug{
		ID:         h.ID,
		GiverID:    h.GiverID,
		ReceiverID: h.ReceiverID,
		Status:     h.Status,
		HugType:    h.HugType,
		CreatedAt:  h.CreatedAt.Time,
		AcceptedAt: acceptedAt,
	}
}

func toModelCooldown(c storage.HugCooldown) *models.HugCooldown {
	var declineCooldownUntil *time.Time
	if c.DeclineCooldownUntil.Valid {
		t := c.DeclineCooldownUntil.Time
		declineCooldownUntil = &t
	}
	return &models.HugCooldown{
		UserAID:              c.UserAID,
		UserBID:              c.UserBID,
		LastHugAt:            c.LastHugAt.Time,
		CooldownSeconds:      c.CooldownSeconds,
		DeclineCooldownUntil: declineCooldownUntil,
	}
}

func toModelFeedItem(row storage.GetRecentHugsFeedRow) *models.HugFeedItem {
	var giverGender *string
	if row.GiverGender.Valid {
		giverGender = &row.GiverGender.String
	}
	var giverDisplayName *string
	if row.GiverDisplayName.Valid {
		giverDisplayName = &row.GiverDisplayName.String
	}
	var receiverDisplayName *string
	if row.ReceiverDisplayName.Valid {
		receiverDisplayName = &row.ReceiverDisplayName.String
	}
	return &models.HugFeedItem{
		ID:                  row.ID,
		GiverID:             row.GiverID,
		ReceiverID:          row.ReceiverID,
		GiverUsername:       row.GiverUsername,
		ReceiverUsername:    row.ReceiverUsername,
		GiverGender:         giverGender,
		GiverDisplayName:    giverDisplayName,
		ReceiverDisplayName: receiverDisplayName,
		HugType:             row.HugType,
		CreatedAt:           row.CreatedAt.Time,
	}
}

func toModelHistoryItem(row storage.ListHugsByUserRow) *models.HugFeedItem {
	var giverGender *string
	if row.GiverGender.Valid {
		giverGender = &row.GiverGender.String
	}
	var giverDisplayName *string
	if row.GiverDisplayName.Valid {
		giverDisplayName = &row.GiverDisplayName.String
	}
	var receiverDisplayName *string
	if row.ReceiverDisplayName.Valid {
		receiverDisplayName = &row.ReceiverDisplayName.String
	}
	return &models.HugFeedItem{
		ID:                  row.ID,
		GiverID:             row.GiverID,
		ReceiverID:          row.ReceiverID,
		GiverUsername:       row.GiverUsername,
		ReceiverUsername:    row.ReceiverUsername,
		GiverGender:         giverGender,
		GiverDisplayName:    giverDisplayName,
		ReceiverDisplayName: receiverDisplayName,
		HugType:             row.HugType,
		CreatedAt:           row.CreatedAt.Time,
	}
}

func toModelLeaderboardEntry(row storage.GetLeaderboardRow) *models.LeaderboardEntry {
	var displayName *string
	if row.DisplayName.Valid {
		displayName = &row.DisplayName.String
	}
	return &models.LeaderboardEntry{
		UserID:       row.ID,
		Username:     row.Username,
		DisplayName:  displayName,
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
		TotalHugs:    int32(row.TotalHugs),
		Rank:         models.GetRank(int32(row.TotalHugs)),
	}
}

func toModelUserListItem(row storage.SearchUsersRow) *models.User {
	var gender *string
	if row.Gender.Valid {
		gender = &row.Gender.String
	}
	var displayName *string
	if row.DisplayName.Valid {
		displayName = &row.DisplayName.String
	}
	return &models.User{
		ID:          row.ID,
		Username:    row.Username,
		Role:        row.Role,
		Gender:      gender,
		DisplayName: displayName,
	}
}

func toModelUserListItemFromAll(row storage.ListAllUsersRow) *models.User {
	var gender *string
	if row.Gender.Valid {
		gender = &row.Gender.String
	}
	var displayName *string
	if row.DisplayName.Valid {
		displayName = &row.DisplayName.String
	}
	return &models.User{
		ID:          row.ID,
		Username:    row.Username,
		Role:        row.Role,
		Gender:      gender,
		DisplayName: displayName,
	}
}

func toModelPendingInboxItem(row storage.GetPendingHugsForUserRow) *models.PendingHugInboxItem {
	var giverGender *string
	if row.GiverGender.Valid {
		giverGender = &row.GiverGender.String
	}
	var giverDisplayName *string
	if row.GiverDisplayName.Valid {
		giverDisplayName = &row.GiverDisplayName.String
	}
	return &models.PendingHugInboxItem{
		ID:               row.ID,
		GiverID:          row.GiverID,
		ReceiverID:       row.ReceiverID,
		GiverUsername:    row.GiverUsername,
		GiverGender:      giverGender,
		GiverDisplayName: giverDisplayName,
		CreatedAt:        row.CreatedAt.Time,
	}
}


