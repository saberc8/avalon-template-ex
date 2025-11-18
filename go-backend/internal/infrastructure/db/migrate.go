package db

import "database/sql"

// AutoMigrate performs minimal automatic migrations required for the Go service
// to work, focusing on the sys_user table and a default admin account.
//
// It is designed to be safe to call on every startup: it only creates the
// table and indexes if they do not exist, and only inserts the admin user
// when it is missing.
func AutoMigrate(database *sql.DB) error {
	if database == nil {
		return nil
	}

	if err := ensureSysUser(database); err != nil {
		return err
	}
	if err := ensureSysRole(database); err != nil {
		return err
	}
	if err := ensureSysUserRole(database); err != nil {
		return err
	}
	if err := ensureSysMenu(database); err != nil {
		return err
	}
	if err := ensureSysRoleMenu(database); err != nil {
		return err
	}

	return nil
}

func ensureSysUser(db *sql.DB) error {
	const checkTable = `SELECT to_regclass('public.sys_user');`
	var tableName sql.NullString
	if err := db.QueryRow(checkTable).Scan(&tableName); err != nil {
		return err
	}

	if !tableName.Valid {
		const ddl = `
CREATE TABLE IF NOT EXISTS sys_user (
    id              BIGINT       PRIMARY KEY,
    username        VARCHAR(64)  NOT NULL,
    nickname        VARCHAR(30)  NOT NULL,
    password        VARCHAR(255),
    gender          SMALLINT     NOT NULL DEFAULT 0,
    email           VARCHAR(255),
    phone           VARCHAR(255),
    avatar          TEXT,
    description     VARCHAR(200),
    status          SMALLINT     NOT NULL DEFAULT 1,
    is_system       BOOLEAN      NOT NULL DEFAULT FALSE,
    pwd_reset_time  TIMESTAMP,
    dept_id         BIGINT       NOT NULL,
    create_user     BIGINT,
    create_time     TIMESTAMP    NOT NULL,
    update_user     BIGINT,
    update_time     TIMESTAMP
);
CREATE UNIQUE INDEX IF NOT EXISTS uk_user_username ON sys_user (username);
CREATE UNIQUE INDEX IF NOT EXISTS uk_user_email    ON sys_user (email);
CREATE UNIQUE INDEX IF NOT EXISTS uk_user_phone    ON sys_user (phone);
CREATE INDEX IF NOT EXISTS idx_user_dept_id        ON sys_user (dept_id);
CREATE INDEX IF NOT EXISTS idx_user_create_user    ON sys_user (create_user);
CREATE INDEX IF NOT EXISTS idx_user_update_user    ON sys_user (update_user);
`
		if _, err := db.Exec(ddl); err != nil {
			return err
		}
	}

	const seedAdmin = `
INSERT INTO sys_user (
    id, username, nickname, password, gender, email, phone, avatar,
    description, status, is_system, pwd_reset_time, dept_id, create_user, create_time
)
SELECT
    1,
    'admin',
    '系统管理员',
    '{bcrypt}$2a$10$4jGwK2BMJ7FgVR.mgwGodey8.xR8FLoU1XSXpxJ9nZQt.pufhasSa',
    1,
    NULL,
    NULL,
    NULL,
    '系统初始用户',
    1,
    TRUE,
    NOW(),
    1,
    1,
    NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_user WHERE username = 'admin');
`
	if _, err := db.Exec(seedAdmin); err != nil {
		return err
	}
	return nil
}

func ensureSysRole(db *sql.DB) error {
	const checkTable = `SELECT to_regclass('public.sys_role');`
	var tableName sql.NullString
	if err := db.QueryRow(checkTable).Scan(&tableName); err != nil {
		return err
	}
	if tableName.Valid {
		return nil
	}

	const ddl = `
CREATE TABLE IF NOT EXISTS sys_role (
    id                  BIGINT       NOT NULL,
    name                VARCHAR(30)  NOT NULL,
    code                VARCHAR(30)  NOT NULL,
    data_scope          SMALLINT     NOT NULL DEFAULT 4,
    description         VARCHAR(200) DEFAULT NULL,
    sort                INTEGER      NOT NULL DEFAULT 999,
    is_system           BOOLEAN      NOT NULL DEFAULT FALSE,
    menu_check_strictly BOOLEAN      DEFAULT TRUE,
    dept_check_strictly BOOLEAN      DEFAULT TRUE,
    create_user         BIGINT       NOT NULL,
    create_time         TIMESTAMP    NOT NULL,
    update_user         BIGINT       DEFAULT NULL,
    update_time         TIMESTAMP    DEFAULT NULL,
    PRIMARY KEY (id)
);
CREATE UNIQUE INDEX IF NOT EXISTS uk_role_name  ON sys_role (name);
CREATE UNIQUE INDEX IF NOT EXISTS uk_role_code  ON sys_role (code);
CREATE INDEX IF NOT EXISTS idx_role_create_user ON sys_role (create_user);
CREATE INDEX IF NOT EXISTS idx_role_update_user ON sys_role (update_user);
`
	if _, err := db.Exec(ddl); err != nil {
		return err
	}

	// Seed admin / general roles (simplified from main_data.sql).
	const seedRoles = `
INSERT INTO sys_role (id, name, code, data_scope, description, sort, is_system, create_user, create_time)
SELECT 1, '系统管理员', 'admin', 1, '系统初始角色', 1, TRUE, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_role WHERE id = 1);

INSERT INTO sys_role (id, name, code, data_scope, description, sort, is_system, create_user, create_time)
SELECT 2, '普通用户', 'general', 4, '系统初始角色', 2, TRUE, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_role WHERE id = 2);
`
	if _, err := db.Exec(seedRoles); err != nil {
		return err
	}
	return nil
}

func ensureSysUserRole(db *sql.DB) error {
	const checkTable = `SELECT to_regclass('public.sys_user_role');`
	var tableName sql.NullString
	if err := db.QueryRow(checkTable).Scan(&tableName); err != nil {
		return err
	}
	if !tableName.Valid {
		const ddl = `
CREATE TABLE IF NOT EXISTS sys_user_role (
    id      BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    role_id BIGINT NOT NULL,
    PRIMARY KEY (id)
);
CREATE UNIQUE INDEX IF NOT EXISTS uk_user_id_role_id ON sys_user_role (user_id, role_id);
`
		if _, err := db.Exec(ddl); err != nil {
			return err
		}
	}
	// Ensure admin -> admin role association exists.
	const seed = `
INSERT INTO sys_user_role (id, user_id, role_id)
SELECT 1, 1, 1
WHERE NOT EXISTS (SELECT 1 FROM sys_user_role WHERE user_id = 1 AND role_id = 1);
`
	if _, err := db.Exec(seed); err != nil {
		return err
	}
	return nil
}

func ensureSysMenu(db *sql.DB) error {
	const checkTable = `SELECT to_regclass('public.sys_menu');`
	var tableName sql.NullString
	if err := db.QueryRow(checkTable).Scan(&tableName); err != nil {
		return err
	}
	if !tableName.Valid {
		const ddl = `
CREATE TABLE IF NOT EXISTS sys_menu (
    id          BIGINT       NOT NULL,
    title       VARCHAR(30)  NOT NULL,
    parent_id   BIGINT       NOT NULL DEFAULT 0,
    type        SMALLINT     NOT NULL DEFAULT 1,
    path        VARCHAR(255) DEFAULT NULL,
    name        VARCHAR(50)  DEFAULT NULL,
    component   VARCHAR(255) DEFAULT NULL,
    redirect    VARCHAR(255) DEFAULT NULL,
    icon        VARCHAR(50)  DEFAULT NULL,
    is_external BOOLEAN      DEFAULT FALSE,
    is_cache    BOOLEAN      DEFAULT FALSE,
    is_hidden   BOOLEAN      DEFAULT FALSE,
    permission  VARCHAR(100) DEFAULT NULL,
    sort        INTEGER      NOT NULL DEFAULT 999,
    status      SMALLINT     NOT NULL DEFAULT 1,
    create_user BIGINT       NOT NULL,
    create_time TIMESTAMP    NOT NULL,
    update_user BIGINT       DEFAULT NULL,
    update_time TIMESTAMP    DEFAULT NULL,
    PRIMARY KEY (id)
);
CREATE INDEX IF NOT EXISTS idx_menu_parent_id   ON sys_menu (parent_id);
CREATE INDEX IF NOT EXISTS idx_menu_create_user ON sys_menu (create_user);
CREATE INDEX IF NOT EXISTS idx_menu_update_user ON sys_menu (update_user);
CREATE UNIQUE INDEX IF NOT EXISTS uk_menu_title_parent_id ON sys_menu (title, parent_id);
`
		if _, err := db.Exec(ddl); err != nil {
			return err
		}
	}

	// Seed a minimal 系统管理 根目录和下属核心菜单（对齐 Java 默认菜单的大类结构）
	const seedMenus = `
INSERT INTO sys_menu (
  id, title, parent_id, type, path, name, component, redirect, icon,
  is_external, is_cache, is_hidden, permission, sort, status,
  create_user, create_time
)
SELECT 1000, '系统管理', 0, 1, '/system', 'System', 'Layout', NULL, 'settings',
       FALSE, FALSE, FALSE, NULL, 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1000);

INSERT INTO sys_menu (
  id, title, parent_id, type, path, name, component, redirect, icon,
  is_external, is_cache, is_hidden, permission, sort, status,
  create_user, create_time
)
SELECT 1010, '用户管理', 1000, 2, '/system/user', 'SystemUser', '/system/user/index', NULL, 'user',
       FALSE, FALSE, FALSE, 'system:user:list', 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1010);

INSERT INTO sys_menu (
  id, title, parent_id, type, path, name, component, redirect, icon,
  is_external, is_cache, is_hidden, permission, sort, status,
  create_user, create_time
)
SELECT 1030, '角色管理', 1000, 2, '/system/role', 'SystemRole', 'system/role/index', NULL, 'user-group',
       FALSE, FALSE, FALSE, NULL, 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1030);

INSERT INTO sys_menu (
  id, title, parent_id, type, path, name, component, redirect, icon,
  is_external, is_cache, is_hidden, permission, sort, status,
  create_user, create_time
)
SELECT 1050, '菜单管理', 1000, 2, '/system/menu', 'SystemMenu', 'system/menu/index', NULL, 'menu',
       FALSE, FALSE, FALSE, NULL, 3, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1050);

INSERT INTO sys_menu (
  id, title, parent_id, type, path, name, component, redirect, icon,
  is_external, is_cache, is_hidden, permission, sort, status,
  create_user, create_time
)
SELECT 1070, '部门管理', 1000, 2, '/system/dept', 'SystemDept', 'system/dept/index', NULL, 'mind-mapping',
       FALSE, FALSE, FALSE, NULL, 4, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1070);

INSERT INTO sys_menu (
  id, title, parent_id, type, path, name, component, redirect, icon,
  is_external, is_cache, is_hidden, permission, sort, status,
  create_user, create_time
)
SELECT 1090, '通知公告', 1000, 2, '/system/notice', 'SystemNotice', 'system/notice/index', NULL, 'notification',
       FALSE, FALSE, FALSE, NULL, 5, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1090);

INSERT INTO sys_menu (
  id, title, parent_id, type, path, name, component, redirect, icon,
  is_external, is_cache, is_hidden, permission, sort, status,
  create_user, create_time
)
SELECT 1110, '文件管理', 1000, 2, '/system/file', 'SystemFile', 'system/file/index', NULL, 'file',
       FALSE, FALSE, FALSE, NULL, 6, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1110);

INSERT INTO sys_menu (
  id, title, parent_id, type, path, name, component, redirect, icon,
  is_external, is_cache, is_hidden, permission, sort, status,
  create_user, create_time
)
SELECT 1130, '字典管理', 1000, 2, '/system/dict', 'SystemDict', 'system/dict/index', NULL, 'bookmark',
       FALSE, FALSE, FALSE, NULL, 7, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1130);

INSERT INTO sys_menu (
  id, title, parent_id, type, path, name, component, redirect, icon,
  is_external, is_cache, is_hidden, permission, sort, status,
  create_user, create_time
)
SELECT 1140, '字典项管理', 1000, 2, '/system/dict/item', 'SystemDictItem', 'system/dict/item/index', NULL, 'bookmark',
       FALSE, FALSE, TRUE, NULL, 8, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1140);

INSERT INTO sys_menu (
  id, title, parent_id, type, path, name, component, redirect, icon,
  is_external, is_cache, is_hidden, permission, sort, status,
  create_user, create_time
)
SELECT 1150, '系统配置', 1000, 2, '/system/config', 'SystemConfig', 'system/config/index', NULL, 'config',
       FALSE, FALSE, FALSE, NULL, 999, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1150);
`
	if _, err := db.Exec(seedMenus); err != nil {
		return err
	}
	return nil
}

func ensureSysRoleMenu(db *sql.DB) error {
	const checkTable = `SELECT to_regclass('public.sys_role_menu');`
	var tableName sql.NullString
	if err := db.QueryRow(checkTable).Scan(&tableName); err != nil {
		return err
	}
	if !tableName.Valid {
		const ddl = `
CREATE TABLE IF NOT EXISTS sys_role_menu (
    role_id BIGINT NOT NULL,
    menu_id BIGINT NOT NULL,
    PRIMARY KEY (role_id, menu_id)
);
`
		if _, err := db.Exec(ddl); err != nil {
			return err
		}
	}
	// Bind admin role to 系统管理及其主要子菜单。
	const seed = `
INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 1, 1000
WHERE NOT EXISTS (SELECT 1 FROM sys_role_menu WHERE role_id = 1 AND menu_id = 1000);

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 1, 1010
WHERE NOT EXISTS (SELECT 1 FROM sys_role_menu WHERE role_id = 1 AND menu_id = 1010);

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 1, 1030
WHERE NOT EXISTS (SELECT 1 FROM sys_role_menu WHERE role_id = 1 AND menu_id = 1030);

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 1, 1050
WHERE NOT EXISTS (SELECT 1 FROM sys_role_menu WHERE role_id = 1 AND menu_id = 1050);

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 1, 1070
WHERE NOT EXISTS (SELECT 1 FROM sys_role_menu WHERE role_id = 1 AND menu_id = 1070);

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 1, 1090
WHERE NOT EXISTS (SELECT 1 FROM sys_role_menu WHERE role_id = 1 AND menu_id = 1090);

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 1, 1110
WHERE NOT EXISTS (SELECT 1 FROM sys_role_menu WHERE role_id = 1 AND menu_id = 1110);

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 1, 1130
WHERE NOT EXISTS (SELECT 1 FROM sys_role_menu WHERE role_id = 1 AND menu_id = 1130);

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 1, 1140
WHERE NOT EXISTS (SELECT 1 FROM sys_role_menu WHERE role_id = 1 AND menu_id = 1140);

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 1, 1150
WHERE NOT EXISTS (SELECT 1 FROM sys_role_menu WHERE role_id = 1 AND menu_id = 1150);
`
	if _, err := db.Exec(seed); err != nil {
		return err
	}
	return nil
}
