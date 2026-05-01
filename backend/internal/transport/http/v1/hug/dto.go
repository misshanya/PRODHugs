package hug

import (
	"go-service-template/internal/models"
	"time"
)

// HugFeedItemDTO is the WebSocket/transport representation of a hug feed event.
type HugFeedItemDTO struct {
	ID                  string  `json:"id"`
	GiverID             string  `json:"giver_id"`
	ReceiverID          string  `json:"receiver_id"`
	GiverUsername       string  `json:"giver_username"`
	ReceiverUsername    string  `json:"receiver_username"`
	GiverGender         *string `json:"giver_gender,omitempty"`
	GiverDisplayName    *string `json:"giver_display_name,omitempty"`
	ReceiverDisplayName *string `json:"receiver_display_name,omitempty"`
	HugType             string  `json:"hug_type"`
	CreatedAt           string  `json:"created_at"`
}

func ToFeedItemDTO(item *models.HugFeedItem) HugFeedItemDTO {
	return HugFeedItemDTO{
		ID:                  item.ID.String(),
		GiverID:             item.GiverID.String(),
		ReceiverID:          item.ReceiverID.String(),
		GiverUsername:       item.GiverUsername,
		ReceiverUsername:    item.ReceiverUsername,
		GiverGender:         item.GiverGender,
		GiverDisplayName:    item.GiverDisplayName,
		ReceiverDisplayName: item.ReceiverDisplayName,
		HugType:             item.HugType,
		CreatedAt:           item.CreatedAt.Format(time.RFC3339),
	}
}
