package models

import (
	"time"

	"github.com/google/uuid"
)

type CreateUser struct {
	Username       string
	Password       string
	HashedPassword string
	Role           string
	Gender         *string
}

type User struct {
	ID             uuid.UUID
	Username       string
	Role           string
	HashedPassword string
	Gender         *string
	BannedAt       *time.Time
	CreatedAt      *time.Time
}

type AdminUser struct {
	ID        uuid.UUID
	Username  string
	Role      string
	Gender    *string
	BannedAt  *time.Time
	CreatedAt *time.Time
	Balance   int32
}

type AdminStats struct {
	TotalUsers  int64
	BannedUsers int64
}

type BlockedUser struct {
	ID        uuid.UUID
	Username  string
	Gender    *string
	CreatedAt time.Time
}
