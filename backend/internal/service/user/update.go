package user

import (
	"context"
	"fmt"
	"go-service-template/internal/errorz"
	"go-service-template/internal/models"
	"go-service-template/pkg/crypto"

	"github.com/google/uuid"
)

func (s *service) UpdateSettings(ctx context.Context, id uuid.UUID, gender *string) (*models.User, error) {
	return s.repo.UpdateSettings(ctx, id, gender)
}

func (s *service) ChangePassword(ctx context.Context, id uuid.UUID, oldPassword, newPassword string) error {
	u, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return err
	}

	ok, err := crypto.ComparePasswordAndHash(oldPassword, u.HashedPassword)
	if err != nil {
		return fmt.Errorf("failed to compare password: %w", err)
	}
	if !ok {
		return errorz.ErrWrongPassword
	}

	hash, err := crypto.GenerateHash(newPassword)
	if err != nil {
		return fmt.Errorf("failed to hash new password: %w", err)
	}

	return s.repo.UpdatePassword(ctx, id, hash)
}

func (s *service) SaveRefreshToken(ctx context.Context, jti string, userID uuid.UUID, expiresAtUnix int64) error {
	return s.refreshTokenRepo.SaveRefreshToken(ctx, jti, userID, expiresAtUnix)
}

func (s *service) IsRefreshTokenActive(ctx context.Context, jti string) (bool, error) {
	return s.refreshTokenRepo.IsRefreshTokenActive(ctx, jti)
}

func (s *service) RevokeRefreshToken(ctx context.Context, jti string) error {
	return s.refreshTokenRepo.RevokeRefreshToken(ctx, jti)
}

func (s *service) RevokeAllUserRefreshTokens(ctx context.Context, userID uuid.UUID) error {
	return s.refreshTokenRepo.RevokeAllUserRefreshTokens(ctx, userID)
}
