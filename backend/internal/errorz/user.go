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
)
