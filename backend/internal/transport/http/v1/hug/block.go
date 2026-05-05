package hug

import (
	"context"
	"errors"

	"go-service-template/internal/errorz"
	"go-service-template/internal/transport/http/middleware"
	v1 "go-service-template/internal/transport/http/v1"

	"github.com/google/uuid"
)

func (h *HugHandler) BlockUser(ctx context.Context, req v1.BlockUserRequestObject) (v1.BlockUserResponseObject, error) {
	userID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)

	err := h.svc.BlockUser(ctx, userID, req.UserId)
	if err != nil {
		if errors.Is(err, errorz.ErrCannotBlockSelf) {
			return v1.BlockUser400JSONResponse{
				BadRequestJSONResponse: v1.BadRequestJSONResponse{Code: v1.CANNOTHUGSELF, Message: "Cannot block yourself"},
			}, nil
		}
		if errors.Is(err, errorz.ErrUserNotFound) {
			return v1.BlockUser404JSONResponse{
				NotFoundJSONResponse: v1.NotFoundJSONResponse{Code: v1.USERNOTFOUND, Message: "User not found"},
			}, nil
		}
		return nil, err
	}

	return v1.BlockUser200JSONResponse{Message: "User blocked"}, nil
}

func (h *HugHandler) UnblockUser(ctx context.Context, req v1.UnblockUserRequestObject) (v1.UnblockUserResponseObject, error) {
	userID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)

	err := h.svc.UnblockUser(ctx, userID, req.UserId)
	if err != nil {
		return nil, err
	}

	return v1.UnblockUser200JSONResponse{Message: "User unblocked"}, nil
}

func (h *HugHandler) GetBlockedUsers(ctx context.Context, _ v1.GetBlockedUsersRequestObject) (v1.GetBlockedUsersResponseObject, error) {
	userID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)

	users, err := h.svc.GetBlockedUsers(ctx, userID)
	if err != nil {
		return nil, err
	}

	result := make(v1.GetBlockedUsers200JSONResponse, len(users))
	for i, u := range users {
		item := v1.BlockedUser{
			Id:          u.ID,
			Username:    u.Username,
			DisplayName: u.DisplayName,
			Tag:         u.Tag,
			SpecialTag:  u.SpecialTag,
			CreatedAt:   u.CreatedAt,
		}
		if u.Gender != nil {
			g := v1.Gender(*u.Gender)
			item.Gender = &g
		}
		result[i] = item
	}

	return result, nil
}
