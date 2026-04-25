package hug

import (
	"context"
	"go-service-template/internal/db/sqlc/storage"
	"go-service-template/internal/models"
	"go-service-template/internal/repository"

	"github.com/google/uuid"
)

func (r *repo) GetRecentFeed(ctx context.Context, limit int32) ([]*models.HugFeedItem, error) {
	q := repository.Queries(ctx, r.q)

	rows, err := q.GetRecentHugsFeed(ctx, limit)
	if err != nil {
		return nil, err
	}

	result := make([]*models.HugFeedItem, len(rows))
	for i, row := range rows {
		result[i] = toModelFeedItem(row)
	}
	return result, nil
}

func (r *repo) GetLeaderboard(ctx context.Context, limit, offset int32) ([]*models.LeaderboardEntry, error) {
	q := repository.Queries(ctx, r.q)

	rows, err := q.GetLeaderboard(ctx, storage.GetLeaderboardParams{
		Lim: limit,
		Off: offset,
	})
	if err != nil {
		return nil, err
	}

	result := make([]*models.LeaderboardEntry, len(rows))
	for i, row := range rows {
		result[i] = toModelLeaderboardEntry(row)
	}
	return result, nil
}

func (r *repo) GetUserStats(ctx context.Context, userID uuid.UUID) (*models.UserStats, error) {
	q := repository.Queries(ctx, r.q)

	row, err := q.GetUserStats(ctx, userID)
	if err != nil {
		return nil, err
	}

	return toModelUserStats(row), nil
}

func (r *repo) SearchUsers(ctx context.Context, query string, limit, offset int32) ([]*models.User, error) {
	q := repository.Queries(ctx, r.q)

	if query == "" {
		rows, err := q.ListAllUsers(ctx, storage.ListAllUsersParams{
			Lim: limit,
			Off: offset,
		})
		if err != nil {
			return nil, err
		}
		result := make([]*models.User, len(rows))
		for i, row := range rows {
			result[i] = toModelUserListItemFromAll(row)
		}
		return result, nil
	}

	rows, err := q.SearchUsers(ctx, storage.SearchUsersParams{
		Query: query,
		Lim:   limit,
		Off:   offset,
	})
	if err != nil {
		return nil, err
	}

	result := make([]*models.User, len(rows))
	for i, row := range rows {
		result[i] = toModelUserListItem(row)
	}
	return result, nil
}
