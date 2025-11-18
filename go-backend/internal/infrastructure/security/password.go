package security

import (
	"errors"
	"strings"

	"golang.org/x/crypto/bcrypt"
)

// PasswordVerifier validates a raw password against a stored hash.
type PasswordVerifier interface {
	Verify(raw, encoded string) (bool, error)
}

// BcryptVerifier implements PasswordVerifier for {bcrypt} prefixed hashes
// as used by Spring Security's DelegatingPasswordEncoder.
type BcryptVerifier struct{}

func (BcryptVerifier) Verify(raw, encoded string) (bool, error) {
	if encoded == "" {
		return false, errors.New("empty password hash")
	}
	// Strip optional "{bcrypt}" prefix to support Spring-encoded values.
	const prefix = "{bcrypt}"
	if strings.HasPrefix(encoded, prefix) {
		encoded = encoded[len(prefix):]
	}
	if err := bcrypt.CompareHashAndPassword([]byte(encoded), []byte(raw)); err != nil {
		if errors.Is(err, bcrypt.ErrMismatchedHashAndPassword) {
			return false, nil
		}
		return false, err
	}
	return true, nil
}

