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
	if err := ensureSysRoleDept(database); err != nil {
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
	if err := ensureSysDept(database); err != nil {
		return err
	}
	if err := ensureSysDict(database); err != nil {
		return err
	}
	if err := ensureSysDictItem(database); err != nil {
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

	// Seed 系统管理 / 用户 / 角色 / 菜单 / 部门 / 字典 / 字典项 菜单与按钮，权限码对齐前端 v-permission。
	// 所有 INSERT 都使用 WHERE NOT EXISTS 防重，因此可以在已有数据的情况下多次执行，
	// 方便后续新增菜单（例如这里补充的部门管理菜单）自动生效。
	const seedMenus = `
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1000, '系统管理', 0, 1, '/system', 'System', 'Layout', '/system/user', 'settings',
       FALSE, FALSE, FALSE, NULL, 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1000);

-- 用户管理
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1010, '用户管理', 1000, 2, '/system/user', 'SystemUser', 'system/user/index', NULL, 'user',
       FALSE, FALSE, FALSE, NULL, 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1010);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1011, '列表', 1010, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:user:list', 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1011);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1012, '详情', 1010, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:user:get', 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1012);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1013, '新增', 1010, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:user:create', 3, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1013);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1014, '修改', 1010, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:user:update', 4, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1014);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1015, '删除', 1010, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:user:delete', 5, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1015);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1016, '导出', 1010, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:user:export', 6, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1016);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1017, '导入', 1010, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:user:import', 7, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1017);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1018, '重置密码', 1010, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:user:resetPwd', 8, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1018);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1019, '分配角色', 1010, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:user:updateRole', 9, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1019);

-- 角色管理
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1030, '角色管理', 1000, 2, '/system/role', 'SystemRole', 'system/role/index', NULL, 'user-group',
       FALSE, FALSE, FALSE, NULL, 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1030);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1031, '列表', 1030, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:role:list', 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1031);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1032, '详情', 1030, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:role:get', 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1032);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1033, '新增', 1030, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:role:create', 3, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1033);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1034, '修改', 1030, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:role:update', 4, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1034);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1035, '删除', 1030, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:role:delete', 5, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1035);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1036, '修改权限', 1030, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:role:updatePermission', 6, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1036);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1037, '分配', 1030, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:role:assign', 7, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1037);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1038, '取消分配', 1030, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:role:unassign', 8, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1038);

-- 菜单管理
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1050, '菜单管理', 1000, 2, '/system/menu', 'SystemMenu', 'system/menu/index', NULL, 'menu',
       FALSE, FALSE, FALSE, NULL, 3, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1050);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1051, '列表', 1050, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:menu:list', 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1051);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1052, '详情', 1050, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:menu:get', 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1052);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1053, '新增', 1050, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:menu:create', 3, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1053);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1054, '修改', 1050, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:menu:update', 4, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1054);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1055, '删除', 1050, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:menu:delete', 5, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1055);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1056, '清除缓存', 1050, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:menu:clearCache', 6, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1056);

-- 部门管理（从 Java 版 main_data.sql 迁移过来）
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1070, '部门管理', 1000, 2, '/system/dept', 'SystemDept', 'system/dept/index', NULL, 'mind-mapping',
       FALSE, FALSE, FALSE, NULL, 4, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1070);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1071, '列表', 1070, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dept:list', 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1071);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1072, '详情', 1070, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dept:get', 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1072);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1073, '新增', 1070, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dept:create', 3, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1073);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1074, '修改', 1070, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dept:update', 4, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1074);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1075, '删除', 1070, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dept:delete', 5, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1075);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1076, '导出', 1070, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dept:export', 6, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1076);

-- 字典管理
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1130, '字典管理', 1000, 2, '/system/dict', 'SystemDict', 'system/dict/index', NULL, 'bookmark',
       FALSE, FALSE, FALSE, NULL, 7, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1130);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1131, '列表', 1130, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dict:list', 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1131);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1132, '详情', 1130, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dict:get', 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1132);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1133, '新增', 1130, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dict:create', 3, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1133);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1134, '修改', 1130, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dict:update', 4, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1134);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1135, '删除', 1130, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dict:delete', 5, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1135);

-- 前端使用 system:dict:item:clearCache 作为权限码，这里与之对齐。
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1136, '清除缓存', 1130, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dict:item:clearCache', 6, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1136);

-- 字典项管理
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1140, '字典项管理', 1000, 2, '/system/dict/item', 'SystemDictItem', 'system/dict/item/index', NULL, 'bookmark',
       FALSE, FALSE, TRUE, NULL, 8, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1140);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1141, '列表', 1140, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dict:item:list', 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1141);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1142, '详情', 1140, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dict:item:get', 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1142);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1143, '新增', 1140, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dict:item:create', 3, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1143);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1144, '修改', 1140, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dict:item:update', 4, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1144);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1145, '删除', 1140, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:dict:item:delete', 5, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1145);
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
	// 让默认管理员角色（ID=1）拥有当前所有菜单权限。
	const bindAllMenus = `
INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 1, m.id
FROM sys_menu AS m
WHERE NOT EXISTS (
    SELECT 1 FROM sys_role_menu rm WHERE rm.role_id = 1 AND rm.menu_id = m.id
);
`
	if _, err := db.Exec(bindAllMenus); err != nil {
		return err
	}
	return nil
}

func ensureSysRoleDept(db *sql.DB) error {
	const checkTable = `SELECT to_regclass('public.sys_role_dept');`
	var tableName sql.NullString
	if err := db.QueryRow(checkTable).Scan(&tableName); err != nil {
		return err
	}
	if tableName.Valid {
		return nil
	}

	const ddl = `
CREATE TABLE IF NOT EXISTS sys_role_dept (
    role_id BIGINT NOT NULL,
    dept_id BIGINT NOT NULL,
    PRIMARY KEY (role_id, dept_id)
);
CREATE INDEX IF NOT EXISTS idx_role_dept_role_id ON sys_role_dept (role_id);
CREATE INDEX IF NOT EXISTS idx_role_dept_dept_id ON sys_role_dept (dept_id);
`
	if _, err := db.Exec(ddl); err != nil {
		return err
	}
	return nil
}

// ensureSysDept creates a minimal sys_dept table if it does not exist,
// and seeds a default root department so that sys_user.dept_id has a target.
func ensureSysDept(db *sql.DB) error {
	const checkTable = `SELECT to_regclass('public.sys_dept');`
	var tableName sql.NullString
	if err := db.QueryRow(checkTable).Scan(&tableName); err != nil {
		return err
	}
	if !tableName.Valid {
		const ddl = `
CREATE TABLE IF NOT EXISTS sys_dept (
    id          BIGINT       NOT NULL,
    name        VARCHAR(30)  NOT NULL,
    parent_id   BIGINT       NOT NULL DEFAULT 0,
    sort        INTEGER      NOT NULL DEFAULT 999,
    status      SMALLINT     NOT NULL DEFAULT 1,
    is_system   BOOLEAN      NOT NULL DEFAULT FALSE,
    description VARCHAR(200) DEFAULT NULL,
    create_user BIGINT       NOT NULL,
    create_time TIMESTAMP    NOT NULL,
    update_user BIGINT       DEFAULT NULL,
    update_time TIMESTAMP    DEFAULT NULL,
    PRIMARY KEY (id)
);
CREATE INDEX IF NOT EXISTS idx_dept_parent_id   ON sys_dept (parent_id);
CREATE INDEX IF NOT EXISTS idx_dept_create_user ON sys_dept (create_user);
CREATE INDEX IF NOT EXISTS idx_dept_update_user ON sys_dept (update_user);
`
		if _, err := db.Exec(ddl); err != nil {
			return err
		}
	}

	// Seed a simple root department.
	const seed = `
INSERT INTO sys_dept (id, name, parent_id, sort, status, is_system, description, create_user, create_time)
SELECT 1, '默认部门', 0, 1, 1, TRUE, '系统初始部门', 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_dept WHERE id = 1);
`
	if _, err := db.Exec(seed); err != nil {
		return err
	}
	return nil
}

// ensureSysDict creates sys_dict table for dictionary definitions.
func ensureSysDict(db *sql.DB) error {
	const checkTable = `SELECT to_regclass('public.sys_dict');`
	var tableName sql.NullString
	if err := db.QueryRow(checkTable).Scan(&tableName); err != nil {
		return err
	}
	if tableName.Valid {
		return nil
	}

	const ddl = `
CREATE TABLE IF NOT EXISTS sys_dict (
    id          BIGINT       NOT NULL,
    name        VARCHAR(30)  NOT NULL,
    code        VARCHAR(30)  NOT NULL,
    description VARCHAR(200) DEFAULT NULL,
    is_system   BOOLEAN      NOT NULL DEFAULT FALSE,
    create_user BIGINT       NOT NULL,
    create_time TIMESTAMP    NOT NULL,
    update_user BIGINT       DEFAULT NULL,
    update_time TIMESTAMP    DEFAULT NULL,
    PRIMARY KEY (id)
);
CREATE UNIQUE INDEX IF NOT EXISTS uk_dict_code ON sys_dict (code);
CREATE INDEX IF NOT EXISTS idx_dict_create_user ON sys_dict (create_user);
CREATE INDEX IF NOT EXISTS idx_dict_update_user ON sys_dict (update_user);
`
	if _, err := db.Exec(ddl); err != nil {
		return err
	}
	return nil
}

// ensureSysDictItem creates sys_dict_item table for dictionary values.
func ensureSysDictItem(db *sql.DB) error {
	const checkTable = `SELECT to_regclass('public.sys_dict_item');`
	var tableName sql.NullString
	if err := db.QueryRow(checkTable).Scan(&tableName); err != nil {
		return err
	}
	if tableName.Valid {
		return nil
	}

	const ddl = `
CREATE TABLE IF NOT EXISTS sys_dict_item (
    id          BIGINT       NOT NULL,
    label       VARCHAR(30)  NOT NULL,
    value       VARCHAR(255) NOT NULL,
    color       VARCHAR(30)  DEFAULT NULL,
    sort        INTEGER      NOT NULL DEFAULT 999,
    description VARCHAR(200) DEFAULT NULL,
    status      SMALLINT     NOT NULL DEFAULT 1,
    dict_id     BIGINT       NOT NULL,
    create_user BIGINT       NOT NULL,
    create_time TIMESTAMP    NOT NULL,
    update_user BIGINT       DEFAULT NULL,
    update_time TIMESTAMP    DEFAULT NULL,
    PRIMARY KEY (id)
);
CREATE INDEX IF NOT EXISTS idx_dict_item_dict_id ON sys_dict_item (dict_id);
CREATE INDEX IF NOT EXISTS idx_dict_item_create_user ON sys_dict_item (create_user);
CREATE INDEX IF NOT EXISTS idx_dict_item_update_user ON sys_dict_item (update_user);
`
	if _, err := db.Exec(ddl); err != nil {
		return err
	}
	return nil
}
