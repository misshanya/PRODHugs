package middleware

import (
	"log/slog"
	"net/http"
	"sync"
	"time"

	"github.com/labstack/echo/v4"
	"golang.org/x/time/rate"
)

// PathRateLimit configures rate limiting for a specific path.
type PathRateLimit struct {
	Rate  rate.Limit
	Burst int
	// TTL controls how long a visitor entry is kept after the last request.
	// Longer TTL is needed for slow limits (e.g. 2/hour for registration).
	TTL time.Duration
}

type authVisitor struct {
	limiter  *rate.Limiter
	lastSeen time.Time
}

// AuthRateLimitMiddleware applies per-IP, per-path token bucket rate limiting.
// pathLimits maps request paths to their rate limit configuration.
// defaultLimit/defaultBurst apply to paths not listed in pathLimits but
// matched by isAuthRateLimitedPath.
func AuthRateLimitMiddleware(defaultLimit rate.Limit, defaultBurst int, pathLimits map[string]PathRateLimit) echo.MiddlewareFunc {
	var (
		mu       sync.Mutex
		visitors = make(map[string]*authVisitor)
	)

	const defaultTTL = 10 * time.Minute

	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			path := c.Path()
			if !isAuthRateLimitedPath(path) {
				return next(c)
			}

			ip := c.RealIP()
			key := ip + ":" + path

			// Determine rate config for this path.
			r, b, ttl := defaultLimit, defaultBurst, defaultTTL
			if pl, ok := pathLimits[path]; ok {
				r, b, ttl = pl.Rate, pl.Burst, pl.TTL
			}

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

			// Evict stale entries — use per-path TTL for the entry being checked,
			// and the default TTL for others (conservative).
			for k, visitor := range visitors {
				evictTTL := defaultTTL
				if pl, ok := pathLimits[pathFromKey(k)]; ok {
					evictTTL = pl.TTL
				}
				if time.Since(visitor.lastSeen) > evictTTL {
					delete(visitors, k)
				}
			}
			mu.Unlock()

			if !v.limiter.Allow() {
				slog.Warn("rate limit exceeded",
					slog.String("ip", ip),
					slog.String("path", path),
					slog.String("key", key),
				)
				return c.JSON(http.StatusTooManyRequests, map[string]any{
					"code":    "RATE_LIMITED",
					"message": "too many requests, try again later",
				})
			}

			_ = ttl // used above in eviction via pathLimits lookup
			return next(c)
		}
	}
}

// pathFromKey extracts the path portion from a "ip:path" key.
func pathFromKey(key string) string {
	for i := len(key) - 1; i >= 0; i-- {
		if key[i] == ':' && i+1 < len(key) && key[i+1] == '/' {
			return key[i+1:]
		}
	}
	return ""
}

func isAuthRateLimitedPath(path string) bool {
	switch path {
	case "/api/v1/auth/login",
		"/api/v1/auth/register",
		"/api/v1/auth/refresh",
		"/api/v1/auth/check-username":
		return true
	default:
		return false
	}
}
