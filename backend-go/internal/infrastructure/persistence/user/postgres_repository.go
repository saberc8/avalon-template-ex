package user

import (
	"context"
	"database/sql"
	"errors"

	domain "voc-go-backend/internal/domain/user"
)

// PgRepository implements domain.Repository using PostgreSQL.
type PgRepository struct {
	db *sql.DB
}

// NewPgRepository creates a new PgRepository.
func NewPgRepository(db *sql.DB) *PgRepository {
	return &PgRepository{db: db}
}

var _ domain.Repository = (*PgRepository)(nil)

// GetByUsername loads a user by username from sys_user.
func (r *PgRepository) GetByUsername(ctx context.Context, username string) (*domain.User, error) {
	const query = `
SELECT
    id,
    username,
    nickname,
    password,
    gender,
    email,
    phone,
    avatar,
    description,
    status,
    is_system,
    pwd_reset_time,
    dept_id,
    create_user,
    create_time,
    update_user,
    update_time
FROM sys_user
WHERE username = $1
LIMIT 1;
`

	row := r.db.QueryRowContext(ctx, query, username)

	var (
		u                  domain.User
		email, phone       sql.NullString
		avatar, desc       sql.NullString
		pwdReset           sql.NullTime
		createUser, updUsr sql.NullInt64
		updateTime         sql.NullTime
	)

	err := row.Scan(
		&u.ID,
		&u.Username,
		&u.Nickname,
		&u.Password,
		&u.Gender,
		&email,
		&phone,
		&avatar,
		&desc,
		&u.Status,
		&u.IsSystem,
		&pwdReset,
		&u.DeptID,
		&createUser,
		&u.CreateTime,
		&updUsr,
		&updateTime,
	)
	if errors.Is(err, sql.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}

	if email.Valid {
		u.Email = &email.String
	}
	if phone.Valid {
		u.Phone = &phone.String
	}
	if avatar.Valid {
		u.Avatar = &avatar.String
	}
	if desc.Valid {
		u.Description = &desc.String
	}
	if pwdReset.Valid {
		t := pwdReset.Time
		u.PwdResetTime = &t
	}
	if createUser.Valid {
		id := createUser.Int64
		u.CreateUser = &id
	}
	if updUsr.Valid {
		id := updUsr.Int64
		u.UpdateUser = &id
	}
	if updateTime.Valid {
		t := updateTime.Time
		u.UpdateTime = &t
	}

	return &u, nil
}

// GetByID loads a user by id from sys_user.
func (r *PgRepository) GetByID(ctx context.Context, id int64) (*domain.User, error) {
	const query = `
SELECT
    id,
    username,
    nickname,
    password,
    gender,
    email,
    phone,
    avatar,
    description,
    status,
    is_system,
    pwd_reset_time,
    dept_id,
    create_user,
    create_time,
    update_user,
    update_time
FROM sys_user
WHERE id = $1
LIMIT 1;
`

	row := r.db.QueryRowContext(ctx, query, id)

	var (
		u                  domain.User
		email, phone       sql.NullString
		avatar, desc       sql.NullString
		pwdReset           sql.NullTime
		createUser, updUsr sql.NullInt64
		updateTime         sql.NullTime
	)

	err := row.Scan(
		&u.ID,
		&u.Username,
		&u.Nickname,
		&u.Password,
		&u.Gender,
		&email,
		&phone,
		&avatar,
		&desc,
		&u.Status,
		&u.IsSystem,
		&pwdReset,
		&u.DeptID,
		&createUser,
		&u.CreateTime,
		&updUsr,
		&updateTime,
	)
	if errors.Is(err, sql.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}

	if email.Valid {
		u.Email = &email.String
	}
	if phone.Valid {
		u.Phone = &phone.String
	}
	if avatar.Valid {
		u.Avatar = &avatar.String
	}
	if desc.Valid {
		u.Description = &desc.String
	}
	if pwdReset.Valid {
		t := pwdReset.Time
		u.PwdResetTime = &t
	}
	if createUser.Valid {
		id := createUser.Int64
		u.CreateUser = &id
	}
	if updUsr.Valid {
		id := updUsr.Int64
		u.UpdateUser = &id
	}
	if updateTime.Valid {
		t := updateTime.Time
		u.UpdateTime = &t
	}

	return &u, nil
}

