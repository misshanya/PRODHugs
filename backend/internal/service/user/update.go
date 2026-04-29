package user

import (
	"context"
	"fmt"
	"strings"

	"go-service-template/internal/errorz"
	"go-service-template/internal/models"
	"go-service-template/pkg/crypto"

	"github.com/google/uuid"
)

func (s *service) UpdateSettings(ctx context.Context, id uuid.UUID, gender *string, displayName *string) (*models.User, error) {
	return s.repo.UpdateSettings(ctx, id, gender, displayName)
}

func (s *service) GetTelegramID(ctx context.Context, userID uuid.UUID) (*int64, error) {
	return s.repo.GetTelegramID(ctx, userID)
}

// SendTelegramCode validates the Telegram ID, checks uniqueness, generates a code, and sends it.
func (s *service) SendTelegramCode(ctx context.Context, userID uuid.UUID, telegramID int64) error {
	// Check if the Telegram ID is already taken by another user.
	taken, err := s.repo.IsTelegramIDTaken(ctx, telegramID, userID)
	if err != nil {
		return err
	}
	if taken {
		return errorz.ErrTelegramIDTaken
	}

	// Validate the bot can reach this chat.
	if s.telegramClient != nil && s.telegramClient.Enabled() {
		if err := s.telegramClient.GetChat(telegramID); err != nil {
			return errorz.ErrInvalidTelegramID
		}
	}

	// Generate and send the code.
	code, err := s.telegramVerify.GenerateCode(userID, telegramID)
	if err != nil {
		return fmt.Errorf("generate telegram code: %w", err)
	}

	msg := fmt.Sprintf("🔑 Ваш код подтверждения: <b>%s</b>\n\nКод действителен 5 минут.", code)
	if s.telegramClient != nil {
		if err := s.telegramClient.SendMessage(telegramID, msg); err != nil {
			return errorz.ErrInvalidTelegramID
		}
	}

	return nil
}

// VerifyTelegramCode checks the code and, on success, links the Telegram ID.
func (s *service) VerifyTelegramCode(ctx context.Context, userID uuid.UUID, telegramID int64, code string) (*models.User, error) {
	ok, reason := s.telegramVerify.CheckCode(userID, telegramID, code)
	if !ok {
		if reason == "expired" {
			return nil, errorz.ErrTelegramCodeExpired
		}
		return nil, errorz.ErrTelegramCodeInvalid
	}

	u, err := s.repo.SetTelegramID(ctx, userID, telegramID)
	if err != nil {
		if isDuplicateTelegramID(err) {
			return nil, errorz.ErrTelegramIDTaken
		}
		return nil, err
	}
	return u, nil
}

// UnlinkTelegram removes the Telegram ID from the user.
func (s *service) UnlinkTelegram(ctx context.Context, userID uuid.UUID) (*models.User, error) {
	return s.repo.ClearTelegramID(ctx, userID)
}

func isDuplicateTelegramID(err error) bool {
	return err != nil && strings.Contains(err.Error(), "users_telegram_id_key")
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
