package admin

import (
	"context"
	"errors"
	"go-service-template/internal/errorz"
	v1 "go-service-template/internal/transport/http/v1"
)

func (h *AdminHandler) AdminDeleteUser(ctx context.Context, req v1.AdminDeleteUserRequestObject) (v1.AdminDeleteUserResponseObject, error) {
	err := h.svc.AdminDeleteUser(ctx, req.UserId)
	if err != nil {
		if errors.Is(err, errorz.ErrCannotDeleteAdmin) {
			return v1.AdminDeleteUser400JSONResponse{
				BadRequestJSONResponse: v1.BadRequestJSONResponse{
					Message: "cannot delete admin user",
					Code:    v1.CANNOTDELETEADMIN,
				},
			}, nil
		}
		return nil, err
	}

	return v1.AdminDeleteUser204Response{}, nil
}
