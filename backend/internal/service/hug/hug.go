package hug

import (
	"context"
	"go-service-template/internal/errorz"
	"go-service-template/internal/models"
	"time"

	"github.com/google/uuid"
)

const (
	defaultCooldownSeconds = 3600 // 1 hour
	cooldownReductionPerUpgrade = 600 // 10 minutes
	upgradeCost = 5 // balance cost per upgrade
	minCooldownSeconds = 300 // 5 minutes minimum
)

func (s *service) SendHug(ctx context.Context, giverID, receiverID uuid.UUID) (*models.Hug, error) {
	if giverID == receiverID {
		return nil, errorz.ErrCannotHugSelf
	}

	// Verify receiver exists
	_, err := s.userRepo.GetByID(ctx, receiverID)
	if err != nil {
		return nil, err
	}

	// Check cooldown
	cooldown, err := s.hugRepo.GetCooldown(ctx, giverID, receiverID)
	if err != nil {
		return nil, err
	}

	if cooldown != nil {
		elapsed := time.Since(cooldown.LastHugAt)
		if elapsed < time.Duration(cooldown.CooldownSeconds)*time.Second {
			return nil, errorz.ErrHugCooldownActive
		}
	}

	// Insert the hug
	h, err := s.hugRepo.InsertHug(ctx, giverID, receiverID)
	if err != nil {
		return nil, err
	}

	// Update cooldown
	cd := int32(defaultCooldownSeconds)
	if cooldown != nil {
		cd = cooldown.CooldownSeconds
	}
	_, err = s.hugRepo.UpsertCooldown(ctx, giverID, receiverID, cd)
	if err != nil {
		return nil, err
	}

	// Give receiver +1 balance
	_, err = s.balanceRepo.AddBalance(ctx, receiverID, 1)
	if err != nil {
		return nil, err
	}

	// Fire WebSocket callback
	if s.onHugCallback != nil {
		giver, _ := s.userRepo.GetByID(ctx, giverID)
		receiver, _ := s.userRepo.GetByID(ctx, receiverID)
		giverName := ""
		receiverName := ""
		if giver != nil {
			giverName = giver.Username
		}
		if receiver != nil {
			receiverName = receiver.Username
		}
		s.onHugCallback(&models.HugFeedItem{
			ID:               h.ID,
			GiverID:          h.GiverID,
			ReceiverID:       h.ReceiverID,
			GiverUsername:    giverName,
			ReceiverUsername: receiverName,
			CreatedAt:        h.CreatedAt,
		})
	}

	return h, nil
}

func (s *service) GetCooldownInfo(ctx context.Context, giverID, receiverID uuid.UUID) (*models.HugCooldown, int32, bool, error) {
	cooldown, err := s.hugRepo.GetCooldown(ctx, giverID, receiverID)
	if err != nil {
		return nil, 0, true, err
	}

	if cooldown == nil {
		// No cooldown exists yet = can hug
		return &models.HugCooldown{
			GiverID:         giverID,
			ReceiverID:      receiverID,
			CooldownSeconds: defaultCooldownSeconds,
		}, 0, true, nil
	}

	elapsed := time.Since(cooldown.LastHugAt)
	remaining := time.Duration(cooldown.CooldownSeconds)*time.Second - elapsed
	if remaining < 0 {
		remaining = 0
	}
	canHug := remaining <= 0

	return cooldown, int32(remaining.Seconds()), canHug, nil
}

func (s *service) UpgradeCooldown(ctx context.Context, giverID, receiverID uuid.UUID) (*models.HugCooldown, error) {
	// Deduct balance
	b, err := s.balanceRepo.DeductBalance(ctx, giverID, int32(upgradeCost))
	if err != nil {
		return nil, err
	}
	if b == nil {
		return nil, errorz.ErrInsufficientBalance
	}

	// Ensure cooldown row exists
	cooldown, err := s.hugRepo.GetCooldown(ctx, giverID, receiverID)
	if err != nil {
		return nil, err
	}
	if cooldown == nil {
		// Create one with default then reduce
		_, err = s.hugRepo.UpsertCooldown(ctx, giverID, receiverID, defaultCooldownSeconds)
		if err != nil {
			return nil, err
		}
	}

	reduced, err := s.hugRepo.ReduceCooldown(ctx, giverID, receiverID, cooldownReductionPerUpgrade)
	if err != nil {
		return nil, err
	}
	if reduced == nil {
		return nil, errorz.ErrCooldownNotFound
	}

	return reduced, nil
}
