package middleware

import (
	"net/http"
	"sync"
	"time"

	"github.com/labstack/echo/v4"
	"golang.org/x/time/rate"
)

type authVisitor struct {
	limiter  *rate.Limiter
	lastSeen time.Time
}

func AuthRateLimitMiddleware(r rate.Limit, b int) echo.MiddlewareFunc {
	var (
		mu       sync.Mutex
		visitors = make(map[string]*authVisitor)
	)

	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			path := c.Path()
			if !isAuthRateLimitedPath(path) {
				return next(c)
			}

			ip := c.RealIP()
			key := ip + ":" + path

			mu.Lock()
			v, ok := visitors[key]
			if !ok {
				v = &authVisitor{
					limiter:  rate.NewLimiter(r, b),
					lastSeen: time.Now(),
				}
				visitors[key] = v
			}
			v.lastSeen = time.Now()

			for k, visitor := range visitors {
				if time.Since(visitor.lastSeen) > 10*time.Minute {
					delete(visitors, k)
				}
			}
			mu.Unlock()

			if !v.limiter.Allow() {
				return c.JSON(http.StatusTooManyRequests, map[string]any{
					"code":    "RATE_LIMITED",
					"message": "too many authentication attempts",
				})
			}

			return next(c)
		}
	}
}

func isAuthRateLimitedPath(path string) bool {
	switch path {
	case "/api/v1/auth/login", "/api/v1/auth/register", "/api/v1/auth/refresh":
		return true
	default:
		return false
	}
}
