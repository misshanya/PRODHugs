package user

import (
	"context"

	"go-service-template/internal/transport/http/middleware"
	v1 "go-service-template/internal/transport/http/v1"

	"github.com/google/uuid"
)

func (h *UserHandler) CreateTelegramLinkToken(ctx context.Context, req v1.CreateTelegramLinkTokenRequestObject) (v1.CreateTelegramLinkTokenResponseObject, error) {
	userID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)

	token, botURL, err := h.svc.GenerateLinkToken(ctx, userID)
	if err != nil {
		return nil, err
	}

	return v1.CreateTelegramLinkToken200JSONResponse{
		Token:  token,
		BotUrl: botURL,
	}, nil
}

func (h *UserHandler) UnlinkTelegram(ctx context.Context, req v1.UnlinkTelegramRequestObject) (v1.UnlinkTelegramResponseObject, error) {
	userID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)

	u, err := h.svc.UnlinkTelegram(ctx, userID)
	if err != nil {
		return nil, err
	}

	return v1.UnlinkTelegram200JSONResponse(toV1User(u)), nil
}
