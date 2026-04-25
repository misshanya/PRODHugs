package errorz

import "errors"

var (
	ErrHugCooldownActive    = errors.New("hug cooldown is still active")
	ErrCannotHugSelf        = errors.New("cannot hug yourself")
	ErrInsufficientBalance  = errors.New("insufficient balance")
	ErrDailyRewardAlreadyClaimed = errors.New("daily reward already claimed today")
	ErrCooldownNotFound     = errors.New("cooldown not found for this pair")
)
