package user

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"errors"
	"fmt"
	"regexp"
	"strings"
	"unicode"

	"go-service-template/internal/errorz"
	"go-service-template/internal/models"
	"go-service-template/internal/telegram"
	"go-service-template/pkg/crypto"
)

var usernamePattern = regexp.MustCompile(`^[a-zA-Z][a-zA-Z0-9_]*$`)

// LoginViaTelegram authenticates an existing user by Telegram ID or auto-registers a new one.
// Returns the user (without tokens — the handler/bot caller generates tokens separately).
func (s *service) LoginViaTelegram(ctx context.Context, info *telegram.TelegramUserInfo) (*models.User, error) {
	// Try to find an existing user with this Telegram ID
	u, err := s.repo.GetByTelegramID(ctx, info.TelegramID)
	if err == nil {
		// Found existing user
		if u.BannedAt != nil {
			return nil, errorz.ErrUserBanned
		}
		return u, nil
	}
	if !errors.Is(err, errorz.ErrUserNotFound) {
		return nil, fmt.Errorf("lookup by telegram ID: %w", err)
	}

	// No existing user — auto-register
	username, err := s.generateUniqueUsername(ctx, info)
	if err != nil {
		return nil, fmt.Errorf("generate username: %w", err)
	}

	// Generate a random password (user won't use it directly; they log in via Telegram)
	randomPassword, err := generateRandomPassword()
	if err != nil {
		return nil, fmt.Errorf("generate random password: %w", err)
	}

	hash, err := crypto.GenerateHash(randomPassword)
	if err != nil {
		return nil, fmt.Errorf("hash password: %w", err)
	}

	input := &models.CreateUser{
		Username:       username,
		Password:       randomPassword,
		HashedPassword: hash,
		Role:           "user",
	}

	u, err = s.repo.Create(ctx, input)
	if err != nil {
		return nil, fmt.Errorf("create user: %w", err)
	}

	// Set telegram_id
	u, err = s.repo.SetTelegramID(ctx, u.ID, info.TelegramID)
	if err != nil {
		return nil, fmt.Errorf("set telegram ID: %w", err)
	}

	// Set display name from Telegram first+last name
	displayName := buildDisplayName(info.FirstName, info.LastName)
	if displayName != "" {
		u, err = s.repo.UpdateSettings(ctx, u.ID, nil, &displayName, nil)
		if err != nil {
			// Non-critical, log but don't fail
			return u, nil
		}
	}

	return u, nil
}

// generateUniqueUsername creates a unique username from Telegram user info.
// Priority: Telegram username -> first name based -> random.
func (s *service) generateUniqueUsername(ctx context.Context, info *telegram.TelegramUserInfo) (string, error) {
	// Try Telegram username first
	if info.Username != "" {
		base := sanitizeUsername(info.Username)
		if base != "" {
			username, err := s.findAvailableUsername(ctx, base)
			if err == nil {
				return username, nil
			}
		}
	}

	// Try first name
	if info.FirstName != "" {
		base := sanitizeUsername(info.FirstName)
		if base != "" {
			username, err := s.findAvailableUsername(ctx, base)
			if err == nil {
				return username, nil
			}
		}
	}

	// Fallback: user_ + random hex
	suffix, err := randomHex(4)
	if err != nil {
		return "", err
	}
	return "user_" + suffix, nil
}

// findAvailableUsername tries the base username, then base_1, base_2, ..., base_99,
// then base_{random} until it finds one that doesn't exist.
func (s *service) findAvailableUsername(ctx context.Context, base string) (string, error) {
	// Enforce length limits: username must be 3-32 chars
	if len(base) < 3 {
		base = base + "__"
	}
	if len(base) > 28 { // leave room for suffixes
		base = base[:28]
	}

	// Try the base name first
	if s.isUsernameAvailable(ctx, base) {
		return base, nil
	}

	// Try numbered suffixes
	for i := 1; i <= 99; i++ {
		candidate := fmt.Sprintf("%s_%d", base, i)
		if len(candidate) <= 32 && s.isUsernameAvailable(ctx, candidate) {
			return candidate, nil
		}
	}

	// Random suffix
	suffix, err := randomHex(4)
	if err != nil {
		return "", err
	}
	candidate := fmt.Sprintf("%s_%s", base, suffix)
	if len(candidate) > 32 {
		candidate = candidate[:32]
	}
	return candidate, nil
}

func (s *service) isUsernameAvailable(ctx context.Context, username string) bool {
	_, err := s.repo.GetByUsername(ctx, username)
	return errors.Is(err, errorz.ErrUserNotFound)
}

// sanitizeUsername converts a string to a valid username matching ^[a-zA-Z][a-zA-Z0-9_]*$.
// Keeps only ASCII letters, digits, and underscores. Ensures it starts with a letter.
func sanitizeUsername(raw string) string {
	raw = strings.ToLower(strings.TrimSpace(raw))
	var b strings.Builder
	for _, r := range raw {
		if unicode.IsLetter(r) && r < 128 {
			b.WriteRune(r)
		} else if unicode.IsDigit(r) {
			b.WriteRune(r)
		} else if r == '_' {
			b.WriteRune(r)
		}
		// skip non-ASCII letters and other characters
	}
	result := b.String()
	if result == "" {
		return ""
	}
	// Ensure starts with a letter
	if !unicode.IsLetter(rune(result[0])) {
		result = "u" + result
	}
	if !usernamePattern.MatchString(result) {
		return ""
	}
	return result
}

func buildDisplayName(firstName, lastName string) string {
	parts := []string{}
	if firstName != "" {
		parts = append(parts, strings.TrimSpace(firstName))
	}
	if lastName != "" {
		parts = append(parts, strings.TrimSpace(lastName))
	}
	return strings.Join(parts, " ")
}

func generateRandomPassword() (string, error) {
	b := make([]byte, 32)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return hex.EncodeToString(b), nil
}

func randomHex(n int) (string, error) {
	b := make([]byte, n)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return hex.EncodeToString(b), nil
}
