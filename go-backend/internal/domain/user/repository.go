package user

import "context"

// Repository defines read/write access to user aggregates.
type Repository interface {
	// GetByUsername returns the user with the given username, or (nil, nil) if not found.
	GetByUsername(ctx context.Context, username string) (*User, error)

	// GetByID returns the user with the given ID, or (nil, nil) if not found.
	GetByID(ctx context.Context, id int64) (*User, error)
}
