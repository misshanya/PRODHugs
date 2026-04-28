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
// Returns the created hug and the receiver's user data (for the response).
func (s *service) SuggestHug(ctx context.Context, giverID, receiverID uuid.UUID) (*models.Hug, *models.User, error) {
	if giverID == receiverID {
		return nil, nil, errorz.ErrCannotHugSelf
	}

	// Check if either user has blocked the other
	blocked, err := s.blockRepo.IsBlockedByEither(ctx, giverID, receiverID)
	if err != nil {
		return nil, nil, err
	}
	if blocked {
		return nil, nil, errorz.ErrUserBlocked
	}

	// Verify receiver exists (can be done outside tx — user won't disappear)
	receiver, err := s.userRepo.GetByID(ctx, receiverID)
	if err != nil {
		return nil, nil, err
	}

	var h *models.Hug

	// Wrap all checks + insert in a transaction to prevent TOCTOU races
	// (e.g., two concurrent requests both passing the pending check before either inserts).
	err = s.tx.RunInTx(ctx, func(txCtx context.Context) error {
		// Combined eligibility check — count + 2 EXISTS in a single DB round-trip.
		outgoingCount, pairPending, reversePending, err := s.hugRepo.CheckSuggestEligibility(txCtx, giverID, receiverID)
		if err != nil {
			return err
		}

		// Check slot capacity
		slots, err := s.userRepo.GetUserSlots(txCtx, giverID)
		if err != nil {
			return err
		}
		if outgoingCount >= slots {
			return errorz.ErrAlreadyHasPendingHug
		}
		if pairPending {
			return errorz.ErrPendingHugExists
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
		return nil, nil, err
	}

	// Fire WebSocket notification asynchronously to avoid blocking the HTTP response.
	if s.onHugSuggestion != nil {
		hugCopy := *h
		go func() {
			giver, _ := s.userRepo.GetByID(context.WithoutCancel(ctx), giverID)
			giverUsername := ""
			var giverGender *string
			if giver != nil {
				giverUsername = giver.Username
				giverGender = giver.Gender
			}
			s.onHugSuggestion(receiverID, &models.PendingHugInboxItem{
				ID:            hugCopy.ID,
				GiverID:       hugCopy.GiverID,
				ReceiverID:    hugCopy.ReceiverID,
				GiverUsername: giverUsername,
				GiverGender:   giverGender,
				CreatedAt:     hugCopy.CreatedAt,
			})
		}()
	}

	return h, receiver, nil
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

		// Start/refresh shared cooldown. UpsertCooldown's ON CONFLICT only updates
		// last_hug_at and preserves the existing cooldown_seconds, so the default
		// value is only used for the initial INSERT.
		_, err = s.hugRepo.UpsertCooldown(txCtx, h.GiverID, h.ReceiverID, int32(defaultCooldownSeconds))
		if err != nil {
			return err
		}

		acceptedHug = h
		return nil
	})
	if err != nil {
		return nil, err
	}

	// Invalidate leaderboard cache since a hug was just completed.
	s.leaderboardCache.InvalidateAll()

	// Fire WebSocket broadcast asynchronously to avoid blocking the HTTP response.
	if s.onHugCompleted != nil && acceptedHug != nil {
		hugCopy := *acceptedHug
		go func() {
			bgCtx := context.WithoutCancel(ctx)
			giver, _ := s.userRepo.GetByID(bgCtx, hugCopy.GiverID)
			receiver, _ := s.userRepo.GetByID(bgCtx, hugCopy.ReceiverID)
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
				ID:               hugCopy.ID,
				GiverID:          hugCopy.GiverID,
				ReceiverID:       hugCopy.ReceiverID,
				GiverUsername:    giverName,
				ReceiverUsername: receiverName,
				GiverGender:      giverGender,
				CreatedAt:        hugCopy.CreatedAt,
			})
		}()
	}

	return acceptedHug, nil
}

// DeclineHug declines a pending hug suggestion.
func (s *service) DeclineHug(ctx context.Context, hugID, receiverID uuid.UUID) error {
	var h *models.Hug

	// Wrap decline + cooldown set in a transaction so the cooldown is always applied
	// when the hug is declined (prevents giver from immediately re-sending).
	err := s.tx.RunInTx(ctx, func(txCtx context.Context) error {
		var err error
		h, err = s.hugRepo.DeclineHug(txCtx, hugID, receiverID)
		if err != nil {
			return err
		}
		if h == nil {
			// Check why
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
			return errorz.ErrHugExpired
		}

		// Set 5-minute decline cooldown on the pair
		declineUntil := time.Now().Add(time.Duration(declineCooldownSeconds) * time.Second)
		return s.hugRepo.SetDeclineCooldown(txCtx, h.GiverID, h.ReceiverID, declineUntil)
	})
	if err != nil {
		return err
	}

	// Fire WebSocket hug_declined to giver (outside tx — fire-and-forget)
	if s.onHugDeclined != nil && h != nil {
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
	var reduced *models.HugCooldown

	// Wrap deduct + cooldown reduction in a transaction so balance is rolled back
	// if the cooldown reduction fails.
	err := s.tx.RunInTx(ctx, func(txCtx context.Context) error {
		// Deduct balance
		b, err := s.balanceRepo.DeductBalance(txCtx, payerID, int32(upgradeCost))
		if err != nil {
			return err
		}
		if b == nil {
			return errorz.ErrInsufficientBalance
		}

		// Ensure cooldown row exists
		cooldown, err := s.hugRepo.GetCooldown(txCtx, payerID, otherUserID)
		if err != nil {
			return err
		}
		if cooldown == nil {
			// Create one with default then reduce
			_, err = s.hugRepo.UpsertCooldown(txCtx, payerID, otherUserID, defaultCooldownSeconds)
			if err != nil {
				return err
			}
		}

		reduced, err = s.hugRepo.ReduceCooldown(txCtx, payerID, otherUserID, cooldownReductionPerUpgrade)
		if err != nil {
			return err
		}
		if reduced == nil {
			return errorz.ErrCooldownNotFound
		}

		return nil
	})
	if err != nil {
		return nil, err
	}

	return reduced, nil
}
