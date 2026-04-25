package hug

import (
	"context"
	"errors"
	"go-service-template/internal/errorz"
	"go-service-template/internal/transport/http/middleware"
	v1 "go-service-template/internal/transport/http/v1"
	"time"

	"github.com/google/uuid"
)

func (h *HugHandler) SendHug(ctx context.Context, req v1.SendHugRequestObject) (v1.SendHugResponseObject, error) {
	giverID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)
	receiverID := req.UserId

	if giverID == receiverID {
		return v1.SendHug400JSONResponse{
			BadRequestJSONResponse: v1.BadRequestJSONResponse{
				Code:    v1.CANNOTHUGSELF,
				Message: "You cannot hug yourself",
			},
		}, nil
	}

	hug, err := h.svc.SendHug(ctx, giverID, receiverID)
	if err != nil {
		if errors.Is(err, errorz.ErrHugCooldownActive) {
			return v1.SendHug429JSONResponse{
				TooManyRequestsJSONResponse: v1.TooManyRequestsJSONResponse{
					Code:    v1.COOLDOWNACTIVE,
					Message: "You need to wait before hugging this user again",
				},
			}, nil
		}
		if errors.Is(err, errorz.ErrUserNotFound) {
			return v1.SendHug404JSONResponse{
				NotFoundJSONResponse: v1.NotFoundJSONResponse{
					Code:    v1.USERNOTFOUND,
					Message: "User not found",
				},
			}, nil
		}
		return nil, err
	}

	return v1.SendHug201JSONResponse{
		Id:        hug.ID,
		GiverId:   hug.GiverID,
		ReceiverId: hug.ReceiverID,
		CreatedAt: hug.CreatedAt,
	}, nil
}

func (h *HugHandler) GetCooldown(ctx context.Context, req v1.GetCooldownRequestObject) (v1.GetCooldownResponseObject, error) {
	giverID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)
	receiverID := req.UserId

	cd, remaining, canHug, err := h.svc.GetCooldownInfo(ctx, giverID, receiverID)
	if err != nil {
		return nil, err
	}

	return v1.GetCooldown200JSONResponse{
		GiverId:          cd.GiverID,
		ReceiverId:       cd.ReceiverID,
		CooldownSeconds:  int(cd.CooldownSeconds),
		RemainingSeconds: int(remaining),
		CanHug:           canHug,
	}, nil
}

func (h *HugHandler) UpgradeCooldown(ctx context.Context, req v1.UpgradeCooldownRequestObject) (v1.UpgradeCooldownResponseObject, error) {
	giverID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)
	receiverID := req.UserId

	cd, err := h.svc.UpgradeCooldown(ctx, giverID, receiverID)
	if err != nil {
		if errors.Is(err, errorz.ErrInsufficientBalance) {
			return v1.UpgradeCooldown400JSONResponse{
				BadRequestJSONResponse: v1.BadRequestJSONResponse{
					Code:    v1.INSUFFICIENTBALANCE,
					Message: "Not enough balance to upgrade cooldown",
				},
			}, nil
		}
		return nil, err
	}

	// Calculate remaining time
	elapsed := time.Since(cd.LastHugAt)
	remaining := time.Duration(cd.CooldownSeconds)*time.Second - elapsed
	if remaining < 0 {
		remaining = 0
	}

	return v1.UpgradeCooldown200JSONResponse{
		GiverId:          cd.GiverID,
		ReceiverId:       cd.ReceiverID,
		CooldownSeconds:  int(cd.CooldownSeconds),
		RemainingSeconds: int(remaining.Seconds()),
		CanHug:           remaining <= 0,
	}, nil
}
