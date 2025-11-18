package security

import (
	"errors"
	"strings"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

// TokenService issues JWT tokens compatible with the front-end expectation
// (plain string token returned by /auth/login).
type TokenService struct {
	secret []byte
	ttl    time.Duration
}

// NewTokenService creates a new TokenService with the given secret and TTL.
func NewTokenService(secret string, ttl time.Duration) *TokenService {
	if ttl <= 0 {
		ttl = 24 * time.Hour
	}
	return &TokenService{
		secret: []byte(secret),
		ttl:    ttl,
	}
}

// Claims defines minimal JWT claims we care about.
type Claims struct {
	UserID int64 `json:"userId"`
	jwt.RegisteredClaims
}

// Generate issues a token for the given user.
func (s *TokenService) Generate(userID int64) (string, error) {
	now := time.Now()
	claims := Claims{
		UserID: userID,
		RegisteredClaims: jwt.RegisteredClaims{
			IssuedAt:  jwt.NewNumericDate(now),
			ExpiresAt: jwt.NewNumericDate(now.Add(s.ttl)),
		},
	}
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	return token.SignedString(s.secret)
}

// Parse extracts Claims from a token string and validates signature and expiry.
func (s *TokenService) Parse(tokenStr string) (*Claims, error) {
	if tokenStr == "" {
		return nil, errors.New("empty token")
	}
	// Allow passing full "Bearer xxx" header value.
	if strings.HasPrefix(strings.ToLower(tokenStr), "bearer ") {
		tokenStr = strings.TrimSpace(tokenStr[7:])
	}
	token, err := jwt.ParseWithClaims(tokenStr, &Claims{}, func(token *jwt.Token) (interface{}, error) {
		return s.secret, nil
	})
	if err != nil {
		return nil, err
	}
	claims, ok := token.Claims.(*Claims)
	if !ok || !token.Valid {
		return nil, errors.New("invalid token")
	}
	return claims, nil
}

