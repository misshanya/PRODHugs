package ws

import (
	"encoding/json"
	"log/slog"
	"net/http"
	"sync"

	"github.com/labstack/echo/v4"
	"golang.org/x/net/websocket"
)

// Hub manages WebSocket clients and broadcasts messages
type Hub struct {
	mu      sync.RWMutex
	clients map[*client]struct{}
}

type client struct {
	ws   *websocket.Conn
	send chan []byte
}

func NewHub() *Hub {
	return &Hub{
		clients: make(map[*client]struct{}),
	}
}

// HandleWS is the Echo handler for WebSocket connections
func (h *Hub) HandleWS(c echo.Context) error {
	websocket.Handler(func(ws *websocket.Conn) {
		defer ws.Close()

		cl := &client{
			ws:   ws,
			send: make(chan []byte, 256),
		}

		h.mu.Lock()
		h.clients[cl] = struct{}{}
		h.mu.Unlock()

		defer func() {
			h.mu.Lock()
			delete(h.clients, cl)
			h.mu.Unlock()
		}()

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

		close(cl.send)
		<-done
	}).ServeHTTP(c.Response(), c.Request())

	return nil
}

// Broadcast sends a message to all connected clients
func (h *Hub) Broadcast(msg interface{}) {
	data, err := json.Marshal(msg)
	if err != nil {
		slog.Error("failed to marshal ws message", "error", err)
		return
	}

	h.mu.RLock()
	defer h.mu.RUnlock()

	for cl := range h.clients {
		select {
		case cl.send <- data:
		default:
			// Client buffer full, skip
		}
	}
}

// ClientCount returns the number of connected clients
func (h *Hub) ClientCount() int {
	h.mu.RLock()
	defer h.mu.RUnlock()
	return len(h.clients)
}

// Ensure we don't get unused import error
var _ = http.StatusOK
