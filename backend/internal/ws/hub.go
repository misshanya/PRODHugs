package ws

import (
	"encoding/json"
	"log/slog"
	"net/http"
	"sync"

	"go-service-template/internal/jwt"

	"github.com/google/uuid"
	"github.com/labstack/echo/v4"
	"golang.org/x/net/websocket"
)

// WSMessage is the typed envelope for all outgoing WebSocket messages.
type WSMessage struct {
	Type string `json:"type"`
	Data any    `json:"data"`
}

// Hub manages WebSocket clients and broadcasts messages
type Hub struct {
	mu         sync.RWMutex
	clients    map[*client]struct{}
	userIndex  map[uuid.UUID]map[*client]struct{} // user -> clients
	jwtManager *jwt.Manager
}

type client struct {
	userID uuid.UUID
	ws     *websocket.Conn
	send   chan []byte
}

func NewHub(jwtManager *jwt.Manager) *Hub {
	return &Hub{
		clients:    make(map[*client]struct{}),
		userIndex:  make(map[uuid.UUID]map[*client]struct{}),
		jwtManager: jwtManager,
	}
}

// HandleWS is the Echo handler for WebSocket connections.
// Authenticates via ?token= query parameter.
func (h *Hub) HandleWS(c echo.Context) error {
	tokenStr := c.QueryParam("token")
	if tokenStr == "" {
		return c.JSON(http.StatusUnauthorized, map[string]string{"error": "missing token"})
	}

	userID, _, tokenType, err := h.jwtManager.ParseToken(tokenStr)
	if err != nil || tokenType != "access" {
		return c.JSON(http.StatusUnauthorized, map[string]string{"error": "invalid token"})
	}

	websocket.Handler(func(ws *websocket.Conn) {
		defer ws.Close()

		cl := &client{
			userID: userID,
			ws:     ws,
			send:   make(chan []byte, 256),
		}

		h.register(cl)

		// Write pump
		done := make(chan struct{})
		go func() {
			defer close(done)
			for msg := range cl.send {
				if _, err := ws.Write(msg); err != nil {
					return
				}
			}
		}()

		// Read pump (keep connection alive, discard messages)
		buf := make([]byte, 512)
		for {
			if _, err := ws.Read(buf); err != nil {
				break
			}
		}

		// Unregister BEFORE closing the channel to prevent Broadcast/SendToUser
		// from sending on a closed channel (which would panic).
		h.unregister(cl)
		close(cl.send)
		<-done
	}).ServeHTTP(c.Response(), c.Request())

	return nil
}

func (h *Hub) register(cl *client) {
	h.mu.Lock()
	defer h.mu.Unlock()

	h.clients[cl] = struct{}{}
	if h.userIndex[cl.userID] == nil {
		h.userIndex[cl.userID] = make(map[*client]struct{})
	}
	h.userIndex[cl.userID][cl] = struct{}{}
}

func (h *Hub) unregister(cl *client) {
	h.mu.Lock()
	defer h.mu.Unlock()

	delete(h.clients, cl)
	if uClients, ok := h.userIndex[cl.userID]; ok {
		delete(uClients, cl)
		if len(uClients) == 0 {
			delete(h.userIndex, cl.userID)
		}
	}
}

// Broadcast sends a typed message to all connected clients.
func (h *Hub) Broadcast(msgType string, data any) {
	msg := WSMessage{Type: msgType, Data: data}
	payload, err := json.Marshal(msg)
	if err != nil {
		slog.Error("failed to marshal ws message", "error", err)
		return
	}

	h.mu.RLock()
	defer h.mu.RUnlock()

	for cl := range h.clients {
		select {
		case cl.send <- payload:
		default:
			// Client buffer full, skip
		}
	}
}

// SendToUser sends a typed message to all connections of a specific user.
func (h *Hub) SendToUser(userID uuid.UUID, msgType string, data any) {
	msg := WSMessage{Type: msgType, Data: data}
	payload, err := json.Marshal(msg)
	if err != nil {
		slog.Error("failed to marshal ws message", "error", err)
		return
	}

	h.mu.RLock()
	defer h.mu.RUnlock()

	uClients, ok := h.userIndex[userID]
	if !ok {
		return
	}

	for cl := range uClients {
		select {
		case cl.send <- payload:
		default:
		}
	}
}

// ClientCount returns the number of connected clients
func (h *Hub) ClientCount() int {
	h.mu.RLock()
	defer h.mu.RUnlock()
	return len(h.clients)
}
