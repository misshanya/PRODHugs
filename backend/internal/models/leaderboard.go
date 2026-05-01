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

// rankDef holds the gender-aware forms for a rank.
type rankDef struct {
	male    string
	female  string
	unknown string
}

var ranks = []struct {
	minHugs int32
	def     rankDef
}{
	{1000, rankDef{"Милашка", "Милашка", "Милашка"}},
	{500, rankDef{"Легенда", "Легенда", "Легенда"}},
	{200, rankDef{"Обнимастер", "Обнимастер", "Обнимастер"}},
	{50, rankDef{"Тактильный", "Тактильная", "Тактильный(ая)"}},
	{10, rankDef{"Неопытный", "Неопытная", "Неопытный(ая)"}},
	{0, rankDef{"Нетактильный", "Нетактильная", "Нетактильный(ая)"}},
}

// GetRank returns a gender-adapted rank name for the given hug count.
func GetRank(totalHugs int32, gender *string) string {
	for _, r := range ranks {
		if totalHugs >= r.minHugs {
			return pickGender(gender, r.def)
		}
	}
	return pickGender(gender, ranks[len(ranks)-1].def)
}

func pickGender(gender *string, def rankDef) string {
	if gender == nil {
		return def.unknown
	}
	switch *gender {
	case "male":
		return def.male
	case "female":
		return def.female
	default:
		return def.unknown
	}
}
