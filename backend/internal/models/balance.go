package models

import (
	"time"

	"github.com/google/uuid"
)

type Balance struct {
	UserID    uuid.UUID
	Amount    int32
	UpdatedAt time.Time
}
