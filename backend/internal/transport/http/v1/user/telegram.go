package user

import (
	"context"
	"errors"

	"go-service-template/internal/errorz"
	"go-service-template/internal/transport/http/middleware"
	v1 "go-service-template/internal/transport/http/v1"

	"github.com/google/uuid"
)

func (h *UserHandler) SendTelegramCode(ctx context.Context, req v1.SendTelegramCodeRequestObject) (v1.SendTelegramCodeResponseObject, error) {
	userID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)

	err := h.svc.SendTelegramCode(ctx, userID, req.Body.TelegramId)
	if err != nil {
		if errors.Is(err, errorz.ErrInvalidTelegramID) {
			return v1.SendTelegramCode400JSONResponse{
				BadRequestJSONResponse: v1.BadRequestJSONResponse{
					Code:    v1.INVALIDTELEGRAMID,
					Message: "Не удалось связаться с этим Telegram ID. Убедитесь, что вы начали диалог с ботом.",
				},
			}, nil
		}
		if errors.Is(err, errorz.ErrTelegramIDTaken) {
			return v1.SendTelegramCode400JSONResponse{
				BadRequestJSONResponse: v1.BadRequestJSONResponse{
					Code:    v1.TELEGRAMIDTAKEN,
					Message: "Этот Telegram ID уже привязан к другому аккаунту.",
				},
			}, nil
		}
		return nil, err
	}

	return v1.SendTelegramCode200JSONResponse{
		Message: "verification code sent",
	}, nil
}

func (h *UserHandler) VerifyTelegramCode(ctx context.Context, req v1.VerifyTelegramCodeRequestObject) (v1.VerifyTelegramCodeResponseObject, error) {
	userID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)

	u, err := h.svc.VerifyTelegramCode(ctx, userID, req.Body.TelegramId, req.Body.Code)
	if err != nil {
		if errors.Is(err, errorz.ErrTelegramCodeInvalid) {
			return v1.VerifyTelegramCode400JSONResponse{
				BadRequestJSONResponse: v1.BadRequestJSONResponse{
					Code:    v1.TELEGRAMCODEINVALID,
					Message: "Неверный код подтверждения.",
				},
			}, nil
		}
		if errors.Is(err, errorz.ErrTelegramCodeExpired) {
			return v1.VerifyTelegramCode400JSONResponse{
				BadRequestJSONResponse: v1.BadRequestJSONResponse{
					Code:    v1.TELEGRAMCODEEXPIRED,
					Message: "Код подтверждения истёк. Запросите новый.",
				},
			}, nil
		}
		if errors.Is(err, errorz.ErrTelegramIDTaken) {
			return v1.VerifyTelegramCode400JSONResponse{
				BadRequestJSONResponse: v1.BadRequestJSONResponse{
					Code:    v1.TELEGRAMIDTAKEN,
					Message: "Этот Telegram ID уже привязан к другому аккаунту.",
				},
			}, nil
		}
		return nil, err
	}

	return v1.VerifyTelegramCode200JSONResponse(toV1User(u)), nil
}

func (h *UserHandler) UnlinkTelegram(ctx context.Context, req v1.UnlinkTelegramRequestObject) (v1.UnlinkTelegramResponseObject, error) {
	userID := ctx.Value(middleware.UserIDContextKey).(uuid.UUID)

	u, err := h.svc.UnlinkTelegram(ctx, userID)
	if err != nil {
		return nil, err
	}

	return v1.UnlinkTelegram200JSONResponse(toV1User(u)), nil
}
