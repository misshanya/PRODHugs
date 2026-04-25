package hug

import (
	"go-service-template/internal/models"
	"time"
)

// HugFeedItemDTO is the WebSocket/transport representation of a hug feed event.
type HugFeedItemDTO struct {
	ID               string `json:"id"`
	GiverID          string `json:"giver_id"`
	ReceiverID       string `json:"receiver_id"`
	GiverUsername    string `json:"giver_username"`
	ReceiverUsername string `json:"receiver_username"`
	CreatedAt        string `json:"created_at"`
}

// ToFeedItemDTO maps a domain HugFeedItem to its transport DTO.
func ToFeedItemDTO(item *models.HugFeedItem) HugFeedItemDTO {
	return HugFeedItemDTO{
		ID:               item.ID.String(),
		GiverID:          item.GiverID.String(),
		ReceiverID:       item.ReceiverID.String(),
		GiverUsername:    item.GiverUsername,
		ReceiverUsername: item.ReceiverUsername,
		CreatedAt:        item.CreatedAt.Format(time.RFC3339),
	}
}
