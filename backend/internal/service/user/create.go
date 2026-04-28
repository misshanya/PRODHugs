package user

import (
	"context"
	"fmt"
	"go-service-template/internal/models"
	"go-service-template/pkg/crypto"
)

func (s *service) Create(ctx context.Context, input *models.CreateUser) (*models.User, string, string, error) {
	hash, err := crypto.GenerateHash(input.Password)
	if err != nil {
		return nil, "", "", fmt.Errorf("failed to hash password: %w", err)
	}
	input.HashedPassword = hash

	u, err := s.repo.Create(ctx, input)
	if err != nil {
		return nil, "", "", err
	}

	accessToken, _, err := s.jwtManager.GenerateAccessToken(u.ID, u.Role)
	if err != nil {
		return nil, "", "", fmt.Errorf("failed to generate access token: %w", err)
	}

	refreshToken, jti, expUnix, err := s.jwtManager.GenerateRefreshToken(u.ID)
	if err != nil {
		return nil, "", "", fmt.Errorf("failed to generate refresh token: %w", err)
	}

	if err := s.refreshTokenRepo.SaveRefreshToken(ctx, jti, u.ID, expUnix); err != nil {
		return nil, "", "", fmt.Errorf("failed to persist refresh token: %w", err)
	}

	return u, accessToken, refreshToken, nil
}
