package errorz

import "errors"

var (
	ErrUserAlreadyExists  = errors.New("user already exists")
	ErrUserNotFound       = errors.New("user not found")
	ErrInvalidCredentials = errors.New("invalid credentials")
	ErrWrongPassword      = errors.New("wrong password")
	ErrUserBanned         = errors.New("user is banned")
	ErrCannotBanAdmin     = errors.New("cannot ban admin")
	ErrUserBlocked        = errors.New("user is blocked")
	ErrCannotBlockSelf    = errors.New("cannot block yourself")
	ErrInvalidTelegramID    = errors.New("invalid telegram ID")
	ErrTelegramIDTaken      = errors.New("telegram ID already linked to another account")
	ErrTelegramCodeInvalid  = errors.New("invalid telegram verification code")
	ErrTelegramCodeExpired  = errors.New("telegram verification code expired")
)
