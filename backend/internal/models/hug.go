package models

import (
	"time"

	"github.com/google/uuid"
)

// Hug status constants
const (
	HugStatusPending   = "pending"
	HugStatusCompleted = "completed"
	HugStatusDeclined  = "declined"
	HugStatusExpired   = "expired"
	HugStatusCancelled = "cancelled"
)

type Hug struct {
	ID         uuid.UUID
	GiverID    uuid.UUID
	ReceiverID uuid.UUID
	Status     string
	CreatedAt  time.Time
	AcceptedAt *time.Time
}

type HugFeedItem struct {
	ID               uuid.UUID
	GiverID          uuid.UUID
	ReceiverID       uuid.UUID
	GiverUsername    string
	ReceiverUsername string
	GiverGender      *string
	CreatedAt        time.Time
}

type HugActivityItem struct {
	Timestamp time.Time
	Count     int64
}

type MutualHugStats struct {
	Total    int64
	Given    int64
	Received int64
}

type HugCooldown struct {
	UserAID              uuid.UUID  // LEAST of the pair
	UserBID              uuid.UUID  // GREATEST of the pair
	LastHugAt            time.Time
	CooldownSeconds      int32
	DeclineCooldownUntil *time.Time
}

// New models for pending hug inbox
type PendingHugInboxItem struct {
	ID            uuid.UUID
	GiverID       uuid.UUID
	ReceiverID    uuid.UUID
	GiverUsername string
	GiverGender   *string
	CreatedAt     time.Time
}

type OutgoingPendingHug struct {
	ID               uuid.UUID
	GiverID          uuid.UUID
	ReceiverID       uuid.UUID
	ReceiverUsername  string
	ReceiverGender   *string
	CreatedAt        time.Time
}
