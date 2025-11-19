package id

import (
	"sync"
	"time"
)

// Simple in-process ID generator.
// It returns a monotonically increasing int64 value based on milliseconds.
// This is sufficient for this single-process admin backend.
var (
	mu   sync.Mutex
	last int64
)

// Next returns the next unique ID.
func Next() int64 {
	mu.Lock()
	defer mu.Unlock()

	now := time.Now().UnixMilli()
	if now <= last {
		last++
	} else {
		last = now
	}
	return last
}

