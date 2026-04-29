package telegram

import (
	"crypto/rand"
	"fmt"
	"sync"
	"time"

	"github.com/google/uuid"
)

const (
	codeTTL    = 5 * time.Minute
	codeLength = 6
)

// verifyKey uniquely identifies a pending verification for a (user, telegramID) pair.
type verifyKey struct {
	UserID     uuid.UUID
	TelegramID int64
}

type verifyEntry struct {
	Code      string
	ExpiresAt time.Time
}

// VerifyStore holds pending Telegram verification codes in memory.
// Codes expire after 5 minutes. Thread-safe.
type VerifyStore struct {
	mu      sync.Mutex
	entries map[verifyKey]verifyEntry
}

// NewVerifyStore creates a new in-memory verification code store.
func NewVerifyStore() *VerifyStore {
	return &VerifyStore{
		entries: make(map[verifyKey]verifyEntry),
	}
}

// GenerateCode creates a new 6-digit verification code for the given user+telegramID pair.
// Any previous code for the same pair is overwritten.
func (s *VerifyStore) GenerateCode(userID uuid.UUID, telegramID int64) (string, error) {
	code, err := randomDigits(codeLength)
	if err != nil {
		return "", fmt.Errorf("generate verification code: %w", err)
	}

	key := verifyKey{UserID: userID, TelegramID: telegramID}

	s.mu.Lock()
	s.entries[key] = verifyEntry{
		Code:      code,
		ExpiresAt: time.Now().Add(codeTTL),
	}
	s.mu.Unlock()

	return code, nil
}

// CheckCode verifies the code for the given user+telegramID pair.
// Returns true on match (and deletes the entry so it can't be reused).
// Returns false if the code is wrong. Returns an error string hint:
// "expired" if the entry existed but expired, "invalid" otherwise.
func (s *VerifyStore) CheckCode(userID uuid.UUID, telegramID int64, code string) (ok bool, reason string) {
	key := verifyKey{UserID: userID, TelegramID: telegramID}

	s.mu.Lock()
	defer s.mu.Unlock()

	entry, exists := s.entries[key]
	if !exists {
		return false, "expired" // no entry = treat as expired
	}

	if time.Now().After(entry.ExpiresAt) {
		delete(s.entries, key)
		return false, "expired"
	}

	if entry.Code != code {
		return false, "invalid"
	}

	// Success — consume the code
	delete(s.entries, key)
	return true, ""
}

// randomDigits generates a string of n cryptographically random decimal digits.
func randomDigits(n int) (string, error) {
	b := make([]byte, n)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	for i := range b {
		b[i] = '0' + b[i]%10
	}
	return string(b), nil
}
