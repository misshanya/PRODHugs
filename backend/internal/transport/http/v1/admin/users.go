package admin

import (
	"context"
	"go-service-template/internal/models"
	v1 "go-service-template/internal/transport/http/v1"
)

func (h *AdminHandler) GetAdminUsers(ctx context.Context, req v1.GetAdminUsersRequestObject) (v1.GetAdminUsersResponseObject, error) {
	limit := int32(20)
	if req.Params.Limit != nil {
		limit = int32(*req.Params.Limit)
	}
	offset := int32(0)
	if req.Params.Offset != nil {
		offset = int32(*req.Params.Offset)
	}

	var users []*models.AdminUser
	var err error

	if req.Params.Q != nil && *req.Params.Q != "" {
		users, err = h.svc.SearchUsersAdmin(ctx, *req.Params.Q, limit, offset)
	} else {
		users, err = h.svc.ListUsersAdmin(ctx, limit, offset)
	}
	if err != nil {
		return nil, err
	}

	result := make(v1.GetAdminUsers200JSONResponse, 0, len(users))
	for _, u := range users {
		result = append(result, toV1AdminUserFromAdmin(u))
	}

	return result, nil
}
