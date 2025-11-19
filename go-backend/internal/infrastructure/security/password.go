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

// PasswordHasher hashes a raw password into a stored representation.
type PasswordHasher interface {
	Hash(raw string) (string, error)
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

// BcryptHasher generates {bcrypt} prefixed hashes compatible with Spring Security.
type BcryptHasher struct{}

func (BcryptHasher) Hash(raw string) (string, error) {
	if strings.TrimSpace(raw) == "" {
		return "", errors.New("empty password")
	}
	bytes, err := bcrypt.GenerateFromPassword([]byte(raw), bcrypt.DefaultCost)
	if err != nil {
		return "", err
	}
	return "{bcrypt}" + string(bytes), nil
}

