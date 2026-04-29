package telegram

import (
	"context"
	"fmt"
	"log/slog"

	"go-service-template/internal/models"

	"github.com/google/uuid"
)

// userRepo is the interface for looking up user data needed by the notifier.
type userRepo interface {
	GetTelegramID(ctx context.Context, userID uuid.UUID) (*int64, error)
	GetByID(ctx context.Context, id uuid.UUID) (*models.User, error)
}

// Notifier sends Telegram notifications for hug events.
// All methods are fire-and-forget: errors are logged but never propagated.
type Notifier struct {
	client *Client
	bot    *Bot
	repo   userRepo
	logger *slog.Logger
}

// NewNotifier creates a new Notifier. If client is nil or disabled, notifications are skipped.
// The bot parameter is used for sending hug suggestions with inline buttons;
// pass nil to send plain-text suggestions instead.
func NewNotifier(client *Client, bot *Bot, repo userRepo, logger *slog.Logger) *Notifier {
	return &Notifier{
		client: client,
		bot:    bot,
		repo:   repo,
		logger: logger,
	}
}

// Enabled returns true if Telegram notifications are configured.
func (n *Notifier) Enabled() bool {
	return n.client != nil && n.client.Enabled()
}

// NotifyHugSuggestion notifies the receiver with Accept/Decline buttons.
func (n *Notifier) NotifyHugSuggestion(ctx context.Context, receiverID uuid.UUID, hugID uuid.UUID, giverName string) {
	if n.bot != nil {
		n.bot.SendHugSuggestion(ctx, receiverID, hugID, giverName)
		return
	}
	// Fallback: plain text without buttons
	n.sendToUser(ctx, receiverID, fmt.Sprintf("🤗 <b>%s</b> хочет тебя обнять!", giverName))
}

// NotifyHugCompleted notifies both participants that the hug was accepted.
func (n *Notifier) NotifyHugCompleted(ctx context.Context, giverID, receiverID uuid.UUID, giverName, receiverName string) {
	n.sendToUser(ctx, giverID, fmt.Sprintf("🎉 <b>%s</b> принял(а) объятие! +1 монета", receiverName))
	n.sendToUser(ctx, receiverID, fmt.Sprintf("🎉 Вы обнялись с <b>%s</b>! +1 монета", giverName))
}

// NotifyHugDeclined notifies the giver that their hug was declined.
func (n *Notifier) NotifyHugDeclined(ctx context.Context, giverID, receiverID uuid.UUID) {
	receiver, err := n.repo.GetByID(ctx, receiverID)
	if err != nil {
		n.logger.Error("telegram: failed to look up receiver", "receiver_id", receiverID, "error", err)
		return
	}
	name := receiver.Username
	if receiver.DisplayName != nil {
		name = *receiver.DisplayName
	}
	n.sendToUser(ctx, giverID, fmt.Sprintf("😔 <b>%s</b> отклонил(а) объятие", name))
}

// NotifyHugCancelled notifies the receiver that the hug request was cancelled.
func (n *Notifier) NotifyHugCancelled(ctx context.Context, receiverID, giverID uuid.UUID) {
	giver, err := n.repo.GetByID(ctx, giverID)
	if err != nil {
		n.logger.Error("telegram: failed to look up giver", "giver_id", giverID, "error", err)
		return
	}
	name := giver.Username
	if giver.DisplayName != nil {
		name = *giver.DisplayName
	}
	n.sendToUser(ctx, receiverID, fmt.Sprintf("❌ <b>%s</b> отменил(а) запрос на объятие", name))
}

func (n *Notifier) sendToUser(ctx context.Context, userID uuid.UUID, text string) {
	if !n.Enabled() {
		return
	}

	telegramID, err := n.repo.GetTelegramID(ctx, userID)
	if err != nil {
		n.logger.Error("telegram: failed to get user telegram_id", "user_id", userID, "error", err)
		return
	}
	if telegramID == nil {
		return // user hasn't configured Telegram notifications
	}

	if err := n.client.SendMessage(*telegramID, text); err != nil {
		n.logger.Error("telegram: failed to send message", "user_id", userID, "telegram_id", *telegramID, "error", err)
	}
}
