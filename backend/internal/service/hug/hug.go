package hug

import (
	"context"
	"time"

	"go-service-template/internal/errorz"
	"go-service-template/internal/models"

	"github.com/google/uuid"
)

const (
	defaultCooldownSeconds      = 3600 // 1 hour
	cooldownReductionPerUpgrade = 600  // 10 minutes
	upgradeCost                 = 5    // balance cost per upgrade
	minCooldownSeconds          = 300  // 5 minutes minimum
	declineCooldownSeconds      = 300  // 5 minutes
)

// SuggestHug creates a pending hug suggestion (replaces old SendHug).
func (s *service) SuggestHug(ctx context.Context, giverID, receiverID uuid.UUID) (*models.Hug, error) {
	if giverID == receiverID {
		return nil, errorz.ErrCannotHugSelf
	}

	// Verify receiver exists (can be done outside tx — user won't disappear)
	_, err := s.userRepo.GetByID(ctx, receiverID)
	if err != nil {
		return nil, err
	}

	var h *models.Hug

	// Wrap all checks + insert in a transaction to prevent TOCTOU races
	// (e.g., two concurrent requests both passing the pending check before either inserts).
	err = s.tx.RunInTx(ctx, func(txCtx context.Context) error {
		// Check if giver already has an outgoing pending hug (global limit of 1)
		hasPending, err := s.hugRepo.HasOutgoingPendingHug(txCtx, giverID)
		if err != nil {
			return err
		}
		if hasPending {
			return errorz.ErrAlreadyHasPendingHug
		}

		// Check if there's already a pending hug for this specific pair
		pairPending, err := s.hugRepo.HasPendingHugForPair(txCtx, giverID, receiverID)
		if err != nil {
			return err
		}
		if pairPending {
			return errorz.ErrPendingHugExists
		}

		// Check if the receiver has already suggested a hug to the giver
		reversePending, err := s.hugRepo.HasPendingHugForPair(txCtx, receiverID, giverID)
		if err != nil {
			return err
		}
		if reversePending {
			return errorz.ErrReversePendingHugExists
		}

		// Check shared cooldown
		cooldown, err := s.hugRepo.GetCooldown(txCtx, giverID, receiverID)
		if err != nil {
			return err
		}

		if cooldown != nil {
			// Check decline cooldown first
			if cooldown.DeclineCooldownUntil != nil && cooldown.DeclineCooldownUntil.After(time.Now()) {
				return errorz.ErrDeclineCooldownActive
			}

			// Check regular cooldown
			elapsed := time.Since(cooldown.LastHugAt)
			if elapsed < time.Duration(cooldown.CooldownSeconds)*time.Second {
				return errorz.ErrHugCooldownActive
			}
		}

		// Insert the pending hug
		h, err = s.hugRepo.InsertHug(txCtx, giverID, receiverID, models.HugStatusPending)
		return err
	})
	if err != nil {
		return nil, err
	}

	// Fire WebSocket suggestion notification to receiver (outside tx — fire-and-forget)
	if s.onHugSuggestion != nil {
		giver, _ := s.userRepo.GetByID(ctx, giverID)
		giverUsername := ""
		var giverGender *string
		if giver != nil {
			giverUsername = giver.Username
			giverGender = giver.Gender
		}
		s.onHugSuggestion(receiverID, &models.PendingHugInboxItem{
			ID:            h.ID,
			GiverID:       h.GiverID,
			ReceiverID:    h.ReceiverID,
			GiverUsername: giverUsername,
			GiverGender:   giverGender,
			CreatedAt:     h.CreatedAt,
		})
	}

	return h, nil
}

// AcceptHug accepts a pending hug suggestion.
func (s *service) AcceptHug(ctx context.Context, hugID, receiverID uuid.UUID) (*models.Hug, error) {
	var acceptedHug *models.Hug

	err := s.tx.RunInTx(ctx, func(txCtx context.Context) error {
		h, err := s.hugRepo.AcceptHug(txCtx, hugID, receiverID)
		if err != nil {
			return err
		}
		if h == nil {
			// Check why it failed — hug might not exist or might have expired
			existing, lookupErr := s.hugRepo.GetHugByID(txCtx, hugID)
			if lookupErr != nil {
				return lookupErr
			}
			if existing == nil {
				return errorz.ErrHugNotFound
			}
			if existing.Status != models.HugStatusPending {
				return errorz.ErrHugNotPending
			}
			// It was pending but the 24h window passed
			return errorz.ErrHugExpired
		}

		// +1 coin to initiator (giver)
		_, err = s.balanceRepo.AddBalance(txCtx, h.GiverID, 1)
		if err != nil {
			return err
		}

		// +1 coin to acceptor (receiver)
		_, err = s.balanceRepo.AddBalance(txCtx, h.ReceiverID, 1)
		if err != nil {
			return err
		}

		// Start shared cooldown
		cd := int32(defaultCooldownSeconds)
		cooldown, _ := s.hugRepo.GetCooldown(txCtx, h.GiverID, h.ReceiverID)
		if cooldown != nil {
			cd = cooldown.CooldownSeconds
		}
		_, err = s.hugRepo.UpsertCooldown(txCtx, h.GiverID, h.ReceiverID, cd)
		if err != nil {
			return err
		}

		acceptedHug = h
		return nil
	})
	if err != nil {
		return nil, err
	}

	// Fire WebSocket hug_completed broadcast
	if s.onHugCompleted != nil && acceptedHug != nil {
		giver, _ := s.userRepo.GetByID(ctx, acceptedHug.GiverID)
		receiver, _ := s.userRepo.GetByID(ctx, acceptedHug.ReceiverID)
		giverName := ""
		receiverName := ""
		var giverGender *string
		if giver != nil {
			giverName = giver.Username
			giverGender = giver.Gender
		}
		if receiver != nil {
			receiverName = receiver.Username
		}
		s.onHugCompleted(&models.HugFeedItem{
			ID:               acceptedHug.ID,
			GiverID:          acceptedHug.GiverID,
			ReceiverID:       acceptedHug.ReceiverID,
			GiverUsername:    giverName,
			ReceiverUsername: receiverName,
			GiverGender:      giverGender,
			CreatedAt:        acceptedHug.CreatedAt,
		})
	}

	return acceptedHug, nil
}

// DeclineHug declines a pending hug suggestion.
func (s *service) DeclineHug(ctx context.Context, hugID, receiverID uuid.UUID) error {
	h, err := s.hugRepo.DeclineHug(ctx, hugID, receiverID)
	if err != nil {
		return err
	}
	if h == nil {
		// Check why
		existing, lookupErr := s.hugRepo.GetHugByID(ctx, hugID)
		if lookupErr != nil {
			return lookupErr
		}
		if existing == nil {
			return errorz.ErrHugNotFound
		}
		if existing.Status != models.HugStatusPending {
			return errorz.ErrHugNotPending
		}
		return errorz.ErrHugExpired
	}

	// Set 5-minute decline cooldown on the pair
	declineUntil := time.Now().Add(time.Duration(declineCooldownSeconds) * time.Second)
	_ = s.hugRepo.SetDeclineCooldown(ctx, h.GiverID, h.ReceiverID, declineUntil)

	// Fire WebSocket hug_declined to giver
	if s.onHugDeclined != nil {
		s.onHugDeclined(h.GiverID, hugID, h.ReceiverID)
	}

	return nil
}

// CancelHug cancels the giver's own outgoing pending hug.
func (s *service) CancelHug(ctx context.Context, hugID, giverID uuid.UUID) error {
	h, err := s.hugRepo.CancelHug(ctx, hugID, giverID)
	if err != nil {
		return err
	}
	if h == nil {
		existing, lookupErr := s.hugRepo.GetHugByID(ctx, hugID)
		if lookupErr != nil {
			return lookupErr
		}
		if existing == nil {
			return errorz.ErrHugNotFound
		}
		if existing.Status != models.HugStatusPending {
			return errorz.ErrHugNotPending
		}
		return errorz.ErrHugExpired
	}

	// Fire WebSocket hug_cancelled to receiver
	if s.onHugCancelled != nil {
		s.onHugCancelled(h.ReceiverID, hugID)
	}

	return nil
}

// GetCooldownInfo returns cooldown details for a pair of users.
func (s *service) GetCooldownInfo(ctx context.Context, userA, userB uuid.UUID) (*models.HugCooldown, int32, bool, int32, error) {
	cooldown, err := s.hugRepo.GetCooldown(ctx, userA, userB)
	if err != nil {
		return nil, 0, true, 0, err
	}

	if cooldown == nil {
		// No cooldown exists yet = can hug
		return &models.HugCooldown{
			UserAID:         userA,
			UserBID:         userB,
			CooldownSeconds: defaultCooldownSeconds,
		}, 0, true, 0, nil
	}

	elapsed := time.Since(cooldown.LastHugAt)
	remaining := time.Duration(cooldown.CooldownSeconds)*time.Second - elapsed
	if remaining < 0 {
		remaining = 0
	}
	canHug := remaining <= 0

	// Calculate decline cooldown remaining
	var declineRemaining int32
	if cooldown.DeclineCooldownUntil != nil {
		dr := time.Until(*cooldown.DeclineCooldownUntil)
		if dr > 0 {
			declineRemaining = int32(dr.Seconds())
			canHug = false
		}
	}

	return cooldown, int32(remaining.Seconds()), canHug, declineRemaining, nil
}

// UpgradeCooldown allows either user in a pair to pay to reduce the shared cooldown.
func (s *service) UpgradeCooldown(ctx context.Context, payerID, otherUserID uuid.UUID) (*models.HugCooldown, error) {
	// Deduct balance
	b, err := s.balanceRepo.DeductBalance(ctx, payerID, int32(upgradeCost))
	if err != nil {
		return nil, err
	}
	if b == nil {
		return nil, errorz.ErrInsufficientBalance
	}

	// Ensure cooldown row exists
	cooldown, err := s.hugRepo.GetCooldown(ctx, payerID, otherUserID)
	if err != nil {
		return nil, err
	}
	if cooldown == nil {
		// Create one with default then reduce
		_, err = s.hugRepo.UpsertCooldown(ctx, payerID, otherUserID, defaultCooldownSeconds)
		if err != nil {
			return nil, err
		}
	}

	reduced, err := s.hugRepo.ReduceCooldown(ctx, payerID, otherUserID, cooldownReductionPerUpgrade)
	if err != nil {
		return nil, err
	}
	if reduced == nil {
		return nil, errorz.ErrCooldownNotFound
	}

	return reduced, nil
}
