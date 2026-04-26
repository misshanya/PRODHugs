package user

import (
	"context"
	"errors"
	"go-service-template/internal/errorz"
	"go-service-template/internal/transport/http/middleware"
	v1 "go-service-template/internal/transport/http/v1"

	"github.com/google/uuid"
)

func (h *UserHandler) UpdateUserSettings(ctx context.Context, req v1.UpdateUserSettingsRequestObject) (v1.UpdateUserSettingsResponseObject, error) {
	userID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)

	var gender *string
	if req.Body.Gender != nil {
		g := string(*req.Body.Gender)
		gender = &g
	}

	u, err := h.svc.UpdateSettings(ctx, userID, gender)
	if err != nil {
		return nil, err
	}

	resp := v1.UpdateUserSettings200JSONResponse(toV1User(u))
	return resp, nil
}

func (h *UserHandler) ChangePassword(ctx context.Context, req v1.ChangePasswordRequestObject) (v1.ChangePasswordResponseObject, error) {
	userID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)

	// Validate new password strength
	if !hasLetter.MatchString(req.Body.NewPassword) {
		return v1.ChangePassword400JSONResponse{
			BadRequestJSONResponse: v1.BadRequestJSONResponse{
				Code:    v1.WEAKPASSWORD,
				Message: "password must contain at least one letter",
			},
		}, nil
	}
	if !hasDigit.MatchString(req.Body.NewPassword) {
		return v1.ChangePassword400JSONResponse{
			BadRequestJSONResponse: v1.BadRequestJSONResponse{
				Code:    v1.WEAKPASSWORD,
				Message: "password must contain at least one digit",
			},
		}, nil
	}
	if !hasSpecial.MatchString(req.Body.NewPassword) {
		return v1.ChangePassword400JSONResponse{
			BadRequestJSONResponse: v1.BadRequestJSONResponse{
				Code:    v1.WEAKPASSWORD,
				Message: "password must contain at least one special character",
			},
		}, nil
	}

	err := h.svc.ChangePassword(ctx, userID, req.Body.OldPassword, req.Body.NewPassword)
	if err != nil {
		if errors.Is(err, errorz.ErrWrongPassword) {
			return v1.ChangePassword400JSONResponse{
				BadRequestJSONResponse: v1.BadRequestJSONResponse{
					Code:    v1.WRONGPASSWORD,
					Message: "current password is incorrect",
				},
			}, nil
		}
		return nil, err
	}

	return v1.ChangePassword200JSONResponse{
		Message: "password changed successfully",
	}, nil
}
