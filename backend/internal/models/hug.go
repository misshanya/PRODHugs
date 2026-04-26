package models

import (
	"time"

	"github.com/google/uuid"
)

type Hug struct {
	ID         uuid.UUID
	GiverID    uuid.UUID
	ReceiverID uuid.UUID
	CreatedAt  time.Time
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

type HugCooldown struct {
	GiverID         uuid.UUID
	ReceiverID      uuid.UUID
	LastHugAt       time.Time
	CooldownSeconds int32
}
