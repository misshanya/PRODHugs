package telegram

import (
	"context"
	"fmt"
	"go-service-template/internal/models"
	"log/slog"
	"strings"

	tgbot "github.com/go-telegram/bot"
	tgmodels "github.com/go-telegram/bot/models"
	"github.com/google/uuid"
)

// botUserRepo is the minimal interface the bot needs to link Telegram accounts.
type botUserRepo interface {
	SetTelegramID(ctx context.Context, userID uuid.UUID, telegramID int64) (*models.User, error)
	IsTelegramIDTaken(ctx context.Context, telegramID int64, excludeUserID uuid.UUID) (bool, error)
}

// Bot is a long-polling Telegram bot that handles /start deep-link commands
// to link user accounts.
type Bot struct {
	client    *Client
	linkStore *LinkStore
	userRepo  botUserRepo
	logger    *slog.Logger
}

// NewBot creates a new Telegram bot. If the client is disabled (no token),
// Run() will be a no-op.
func NewBot(client *Client, linkStore *LinkStore, userRepo botUserRepo, logger *slog.Logger) *Bot {
	return &Bot{
		client:    client,
		linkStore: linkStore,
		userRepo:  userRepo,
		logger:    logger,
	}
}

// Run starts the long-polling bot. Blocks until ctx is cancelled.
// If the bot token is empty, returns immediately.
func (b *Bot) Run(ctx context.Context) {
	if !b.client.Enabled() {
		b.logger.Info("telegram bot disabled (no token)")
		return
	}

	opts := []tgbot.Option{
		tgbot.WithDefaultHandler(b.handleUpdate),
	}

	tg, err := tgbot.New(b.client.token, opts...)
	if err != nil {
		b.logger.Error("telegram bot: failed to create", "error", err)
		return
	}

	b.logger.Info("telegram bot started (long-polling)")
	tg.Start(ctx) // blocks until ctx is cancelled
	b.logger.Info("telegram bot stopped")
}

func (b *Bot) handleUpdate(ctx context.Context, bot *tgbot.Bot, update *tgmodels.Update) {
	if update.Message == nil || update.Message.Text == "" {
		return
	}

	text := update.Message.Text
	chatID := update.Message.Chat.ID

	// Only handle /start commands
	if !strings.HasPrefix(text, "/start") {
		return
	}

	// Parse payload: "/start TOKEN" or just "/start"
	parts := strings.SplitN(text, " ", 2)
	if len(parts) < 2 || strings.TrimSpace(parts[1]) == "" {
		b.reply(ctx, bot, chatID, "Чтобы привязать аккаунт, используй настройки на сайте")
		return
	}

	token := strings.TrimSpace(parts[1])
	userID, ok := b.linkStore.ConsumeToken(token)
	if !ok {
		b.reply(ctx, bot, chatID, "Ссылка недействительна или истекла. Попробуй снова через настройки приложения")
		return
	}

	// Check if this Telegram account is already linked to a different user
	taken, err := b.userRepo.IsTelegramIDTaken(ctx, chatID, userID)
	if err != nil {
		b.logger.Error("telegram bot: failed to check telegram_id", "error", err)
		b.reply(ctx, bot, chatID, "Произошла ошибка. Попробуй позже :(")
		return
	}
	if taken {
		b.reply(ctx, bot, chatID, "Этот Telegram аккаунт уже привязан к другому пользователю :(")
		return
	}

	// Link the account
	_, err = b.userRepo.SetTelegramID(ctx, userID, chatID)
	if err != nil {
		b.logger.Error("telegram bot: failed to set telegram_id", "user_id", userID, "chat_id", chatID, "error", err)
		b.reply(ctx, bot, chatID, "Произошла ошибка при привязке. Попробуй позже :(")
		return
	}

	b.logger.Info("telegram bot: account linked", "user_id", userID, "chat_id", chatID)
	b.reply(ctx, bot, chatID, "✅ Аккаунт привязан! Теперь ты не пропустишь обнимашки от любимых продовцев")
}

func (b *Bot) reply(ctx context.Context, bot *tgbot.Bot, chatID int64, text string) {
	_, err := bot.SendMessage(ctx, &tgbot.SendMessageParams{
		ChatID: chatID,
		Text:   text,
	})
	if err != nil {
		b.logger.Error("telegram bot: failed to send message", "chat_id", chatID, "error", err)
	}
}

// DeepLinkURL returns the t.me deep-link URL for a given token and bot username.
func DeepLinkURL(botUsername, token string) string {
	return fmt.Sprintf("https://t.me/%s?start=%s", botUsername, token)
}
