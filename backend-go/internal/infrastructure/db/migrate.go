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
	if err := ensureSysLog(database); err != nil {
		return err
	}
	if err := ensureSysFile(database); err != nil {
		return err
	}
	if err := ensureSysOption(database); err != nil {
		return err
	}
	if err := ensureSysStorage(database); err != nil {
		return err
	}
	if err := ensureSysClient(database); err != nil {
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

-- 系统配置
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1150, '系统配置', 1000, 2, '/system/config', 'SystemConfig', 'system/config/index', NULL, 'config',
       FALSE, FALSE, FALSE, NULL, 999, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1150);

-- 网站配置
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1160, '网站配置', 1150, 2, '/system/config?tab=site', 'SystemSiteConfig', 'system/config/site/index', NULL, 'apps',
       FALSE, FALSE, TRUE, NULL, 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1160);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1161, '查询', 1160, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:siteConfig:get', 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1161);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1162, '修改', 1160, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:siteConfig:update', 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1162);

-- 安全配置
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1170, '安全配置', 1150, 2, '/system/config?tab=security', 'SystemSecurityConfig', 'system/config/security/index', NULL, 'safe',
       FALSE, FALSE, TRUE, NULL, 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1170);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1171, '查询', 1170, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:securityConfig:get', 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1171);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1172, '修改', 1170, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:securityConfig:update', 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1172);

-- 登录配置
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1180, '登录配置', 1150, 2, '/system/config?tab=login', 'SystemLoginConfig', 'system/config/login/index', NULL, 'lock',
       FALSE, FALSE, TRUE, NULL, 3, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1180);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1181, '查询', 1180, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:loginConfig:get', 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1181);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1182, '修改', 1180, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:loginConfig:update', 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1182);

-- 存储配置（菜单和按钮先迁移，具体存储配置接口后续再迁）
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1230, '存储配置', 1150, 2, '/system/config?tab=storage', 'SystemStorage', 'system/config/storage/index', NULL, 'storage',
       FALSE, FALSE, TRUE, NULL, 6, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1230);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1231, '列表', 1230, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:storage:list', 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1231);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1232, '详情', 1230, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:storage:get', 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1232);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1233, '新增', 1230, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:storage:create', 3, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1233);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1234, '修改', 1230, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:storage:update', 4, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1234);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1235, '删除', 1230, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:storage:delete', 5, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1235);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1236, '修改状态', 1230, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:storage:updateStatus', 6, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1236);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1237, '设为默认存储', 1230, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:storage:setDefault', 7, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1237);

-- 客户端配置（同样先迁菜单）
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1250, '客户端配置', 1150, 2, '/system/config?tab=client', 'SystemClient', 'system/config/client/index', NULL, 'mobile',
       FALSE, FALSE, TRUE, NULL, 7, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1250);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1251, '列表', 1250, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:client:list', 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1251);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1252, '详情', 1250, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:client:get', 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1252);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1253, '新增', 1250, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:client:create', 3, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1253);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1254, '修改', 1250, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:client:update', 4, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1254);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1255, '删除', 1250, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:client:delete', 5, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1255);

-- 文件管理
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1110, '文件管理', 1000, 2, '/system/file', 'SystemFile', 'system/file/index', NULL, 'file',
       FALSE, FALSE, FALSE, NULL, 6, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1110);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1111, '列表', 1110, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:file:list', 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1111);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1112, '详情', 1110, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:file:get', 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1112);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1113, '上传', 1110, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:file:upload', 3, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1113);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1114, '修改', 1110, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:file:update', 4, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1114);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1115, '删除', 1110, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:file:delete', 5, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1115);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1116, '下载', 1110, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:file:download', 6, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1116);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1117, '创建文件夹', 1110, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:file:createDir', 7, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1117);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 1118, '计算文件夹大小', 1110, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'system:file:calcDirSize', 8, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 1118);

-- 系统监控（参考 Java main_data.sql）
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 2000, '系统监控', 0, 1, '/monitor', 'Monitor', 'Layout', '/monitor/online', 'computer',
       FALSE, FALSE, FALSE, NULL, 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 2000);

-- 在线用户
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 2010, '在线用户', 2000, 2, '/monitor/online', 'MonitorOnline', 'monitor/online/index', NULL, 'user',
       FALSE, FALSE, FALSE, NULL, 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 2010);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 2011, '列表', 2010, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'monitor:online:list', 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 2011);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 2012, '强退', 2010, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'monitor:online:kickout', 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 2012);

-- 系统日志
INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 2030, '系统日志', 2000, 2, '/monitor/log', 'MonitorLog', 'monitor/log/index', NULL, 'history',
       FALSE, FALSE, FALSE, NULL, 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 2030);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 2031, '列表', 2030, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'monitor:log:list', 1, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 2031);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 2032, '详情', 2030, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'monitor:log:get', 2, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 2032);

INSERT INTO sys_menu (id, title, parent_id, type, path, name, component, redirect, icon,
                      is_external, is_cache, is_hidden, permission, sort, status,
                      create_user, create_time)
SELECT 2033, '导出', 2030, 3, NULL, NULL, NULL, NULL, NULL,
       NULL, NULL, NULL, 'monitor:log:export', 3, 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_menu WHERE id = 2033);
`
	if _, err := db.Exec(seedMenus); err != nil {
		return err
	}
	return nil
}

func ensureSysFile(db *sql.DB) error {
	const checkTable = `SELECT to_regclass('public.sys_file');`
	var tableName sql.NullString
	if err := db.QueryRow(checkTable).Scan(&tableName); err != nil {
		return err
	}
	if tableName.Valid {
		return nil
	}

	const ddl = `
CREATE TABLE IF NOT EXISTS sys_file (
    id                 BIGINT       NOT NULL,
    name               VARCHAR(255) NOT NULL,
    original_name      VARCHAR(255) NOT NULL,
    size               BIGINT,
    parent_path        VARCHAR(512) NOT NULL DEFAULT '/',
    path               VARCHAR(512) NOT NULL,
    extension          VARCHAR(100),
    content_type       VARCHAR(255),
    type               SMALLINT     NOT NULL DEFAULT 1,
    sha256             VARCHAR(256) NOT NULL,
    metadata           TEXT,
    thumbnail_name     VARCHAR(255),
    thumbnail_size     BIGINT,
    thumbnail_metadata TEXT,
    storage_id         BIGINT       NOT NULL,
    create_user        BIGINT       NOT NULL,
    create_time        TIMESTAMP    NOT NULL,
    update_user        BIGINT,
    update_time        TIMESTAMP,
    PRIMARY KEY (id)
);
CREATE INDEX IF NOT EXISTS idx_file_type       ON sys_file (type);
CREATE INDEX IF NOT EXISTS idx_file_sha256     ON sys_file (sha256);
CREATE INDEX IF NOT EXISTS idx_file_storage_id ON sys_file (storage_id);
CREATE INDEX IF NOT EXISTS idx_file_create_user ON sys_file (create_user);
`
	if _, err := db.Exec(ddl); err != nil {
		return err
	}
	return nil
}

// ensureSysOption creates sys_option table and seeds default options.
func ensureSysOption(db *sql.DB) error {
	const checkTable = `SELECT to_regclass('public.sys_option');`
	var tableName sql.NullString
	if err := db.QueryRow(checkTable).Scan(&tableName); err != nil {
		return err
	}
	if !tableName.Valid {
		const ddl = `
CREATE TABLE IF NOT EXISTS sys_option (
    id            BIGINT       NOT NULL,
    category      VARCHAR(50)  NOT NULL,
    name          VARCHAR(50)  NOT NULL,
    code          VARCHAR(100) NOT NULL,
    value         TEXT,
    default_value TEXT,
    description   VARCHAR(200),
    update_user   BIGINT,
    update_time   TIMESTAMP,
    PRIMARY KEY (id)
);
CREATE UNIQUE INDEX IF NOT EXISTS uk_option_category_code ON sys_option (category, code);
`
		if _, err := db.Exec(ddl); err != nil {
			return err
		}
	}

	// Seed a subset of default options from Java main_data.sql.
	const seed = `
INSERT INTO sys_option (id, category, name, code, value, default_value, description)
SELECT 1, 'SITE', '系统名称', 'SITE_TITLE', NULL, 'ContiNew Admin', '显示在浏览器标题栏和登录界面的系统名称'
WHERE NOT EXISTS (SELECT 1 FROM sys_option WHERE id = 1);

INSERT INTO sys_option (id, category, name, code, value, default_value, description)
SELECT 2, 'SITE', '系统描述', 'SITE_DESCRIPTION', NULL, '持续迭代优化的前后端分离中后台管理系统框架', '用于 SEO 的网站元描述'
WHERE NOT EXISTS (SELECT 1 FROM sys_option WHERE id = 2);

INSERT INTO sys_option (id, category, name, code, value, default_value, description)
SELECT 3, 'SITE', '版权声明', 'SITE_COPYRIGHT', NULL, 'Copyright © 2022 - present ContiNew Admin 版权所有', '显示在页面底部的版权声明文本'
WHERE NOT EXISTS (SELECT 1 FROM sys_option WHERE id = 3);

INSERT INTO sys_option (id, category, name, code, value, default_value, description)
SELECT 4, 'SITE', '备案号', 'SITE_BEIAN', NULL, NULL, '工信部 ICP 备案编号（如：京ICP备12345678号）'
WHERE NOT EXISTS (SELECT 1 FROM sys_option WHERE id = 4);

INSERT INTO sys_option (id, category, name, code, value, default_value, description)
SELECT 5, 'SITE', '系统图标', 'SITE_FAVICON', NULL, '/favicon.ico', '浏览器标签页显示的网站图标（建议 .ico 格式）'
WHERE NOT EXISTS (SELECT 1 FROM sys_option WHERE id = 5);

INSERT INTO sys_option (id, category, name, code, value, default_value, description)
SELECT 6, 'SITE', '系统LOGO', 'SITE_LOGO', NULL, '/logo.svg', '显示在登录页面和系统导航栏的网站图标（建议 .svg 格式）'
WHERE NOT EXISTS (SELECT 1 FROM sys_option WHERE id = 6);

INSERT INTO sys_option (id, category, name, code, value, default_value, description)
SELECT 10, 'PASSWORD', '密码错误锁定阈值', 'PASSWORD_ERROR_LOCK_COUNT', NULL, '5', '连续登录失败次数达到该值将锁定账号（0-10次，0表示禁用锁定）'
WHERE NOT EXISTS (SELECT 1 FROM sys_option WHERE id = 10);

INSERT INTO sys_option (id, category, name, code, value, default_value, description)
SELECT 11, 'PASSWORD', '账号锁定时长（分钟）', 'PASSWORD_ERROR_LOCK_MINUTES', NULL, '5', '账号锁定后自动解锁的时间（1-1440分钟，即24小时）'
WHERE NOT EXISTS (SELECT 1 FROM sys_option WHERE id = 11);

INSERT INTO sys_option (id, category, name, code, value, default_value, description)
SELECT 12, 'PASSWORD', '密码有效期（天）', 'PASSWORD_EXPIRATION_DAYS', NULL, '0', '密码强制修改周期（0-999天，0表示永不过期）'
WHERE NOT EXISTS (SELECT 1 FROM sys_option WHERE id = 12);

INSERT INTO sys_option (id, category, name, code, value, default_value, description)
SELECT 13, 'PASSWORD', '密码到期提醒（天）', 'PASSWORD_EXPIRATION_WARNING_DAYS', NULL, '0', '密码过期前的提前提醒天数（0表示不提醒）'
WHERE NOT EXISTS (SELECT 1 FROM sys_option WHERE id = 13);

INSERT INTO sys_option (id, category, name, code, value, default_value, description)
SELECT 14, 'PASSWORD', '历史密码重复校验次数', 'PASSWORD_REPETITION_TIMES', NULL, '3', '禁止使用最近 N 次的历史密码（3-32次）'
WHERE NOT EXISTS (SELECT 1 FROM sys_option WHERE id = 14);

INSERT INTO sys_option (id, category, name, code, value, default_value, description)
SELECT 15, 'PASSWORD', '密码最小长度', 'PASSWORD_MIN_LENGTH', NULL, '8', '密码最小字符长度要求（8-32个字符）'
WHERE NOT EXISTS (SELECT 1 FROM sys_option WHERE id = 15);

INSERT INTO sys_option (id, category, name, code, value, default_value, description)
SELECT 16, 'PASSWORD', '是否允许密码包含用户名', 'PASSWORD_ALLOW_CONTAIN_USERNAME', NULL, '1', '是否允许密码包含正序或倒序的用户名字符'
WHERE NOT EXISTS (SELECT 1 FROM sys_option WHERE id = 16);

INSERT INTO sys_option (id, category, name, code, value, default_value, description)
SELECT 17, 'PASSWORD', '密码是否必须包含特殊字符', 'PASSWORD_REQUIRE_SYMBOLS', NULL, '0', '是否要求密码必须包含特殊字符（如：!@#$%）'
WHERE NOT EXISTS (SELECT 1 FROM sys_option WHERE id = 17);

INSERT INTO sys_option (id, category, name, code, value, default_value, description)
SELECT 27, 'LOGIN', '是否启用验证码', 'LOGIN_CAPTCHA_ENABLED', NULL, '1', NULL
WHERE NOT EXISTS (SELECT 1 FROM sys_option WHERE id = 27);
`
	if _, err := db.Exec(seed); err != nil {
		return err
	}

	return nil
}

// ensureSysStorage 创建 sys_storage 表并写入与 Java 版一致的默认存储配置（简化版）。
func ensureSysStorage(db *sql.DB) error {
	const checkTable = `SELECT to_regclass('public.sys_storage');`
	var tableName sql.NullString
	if err := db.QueryRow(checkTable).Scan(&tableName); err != nil {
		return err
	}
	if !tableName.Valid {
		const ddl = `
CREATE TABLE IF NOT EXISTS sys_storage (
    id          BIGINT       NOT NULL,
    name        VARCHAR(100) NOT NULL,
    code        VARCHAR(30)  NOT NULL,
    type        SMALLINT     NOT NULL DEFAULT 1,
    access_key  VARCHAR(255) DEFAULT NULL,
    secret_key  VARCHAR(255) DEFAULT NULL,
    endpoint    VARCHAR(255) DEFAULT NULL,
    region      VARCHAR(100) DEFAULT NULL,
    bucket_name VARCHAR(255) NOT NULL,
    domain      VARCHAR(255) DEFAULT NULL,
    description VARCHAR(200) DEFAULT NULL,
    is_default  BOOLEAN      NOT NULL DEFAULT FALSE,
    sort        INTEGER      NOT NULL DEFAULT 999,
    status      SMALLINT     NOT NULL DEFAULT 1,
    create_user BIGINT       NOT NULL,
    create_time TIMESTAMP    NOT NULL,
    update_user BIGINT       DEFAULT NULL,
    update_time TIMESTAMP    DEFAULT NULL,
    PRIMARY KEY (id)
);
CREATE UNIQUE INDEX IF NOT EXISTS uk_storage_code  ON sys_storage (code);
CREATE INDEX IF NOT EXISTS idx_storage_create_user ON sys_storage (create_user);
CREATE INDEX IF NOT EXISTS idx_storage_update_user ON sys_storage (update_user);
`
		if _, err := db.Exec(ddl); err != nil {
			return err
		}
	} else {
		// 已存在表时，确保新增的 region 字段已创建，用于兼容七牛等需要 Region 的对象存储。
		const checkRegion = `
SELECT 1
FROM information_schema.columns
WHERE table_name = 'sys_storage' AND column_name = 'region'
LIMIT 1;
`
		var dummy int
		err := db.QueryRow(checkRegion).Scan(&dummy)
		if err != nil && err != sql.ErrNoRows {
			return err
		}
		if err == sql.ErrNoRows {
			if _, err := db.Exec(`ALTER TABLE sys_storage ADD COLUMN region VARCHAR(100) DEFAULT NULL;`); err != nil {
				return err
			}
		}
	}

	// 默认存储：本地存储 + 相对访问路径，便于开发环境直接使用。
	const seed = `
INSERT INTO sys_storage (
    id, name, code, type, access_key, secret_key, endpoint,
    bucket_name, domain, description, is_default, sort, status,
    create_user, create_time
)
SELECT 1,
       '开发环境',
       'local_dev',
       1,
       NULL,
       NULL,
       NULL,
       './data/file/',
       '/file/',
       '本地存储',
       TRUE,
       1,
       1,
       1,
       NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_storage WHERE id = 1);
`
	if _, err := db.Exec(seed); err != nil {
		return err
	}
	return nil
}

// ensureSysClient 创建 sys_client 表并写入一个默认客户端配置。
func ensureSysClient(db *sql.DB) error {
	const checkTable = `SELECT to_regclass('public.sys_client');`
	var tableName sql.NullString
	if err := db.QueryRow(checkTable).Scan(&tableName); err != nil {
		return err
	}
	if !tableName.Valid {
		const ddl = `
CREATE TABLE IF NOT EXISTS sys_client (
    id             BIGINT       NOT NULL,
    client_id      VARCHAR(50)  NOT NULL,
    client_type    VARCHAR(50)  NOT NULL,
    auth_type      JSON         NOT NULL,
    active_timeout BIGINT       NOT NULL DEFAULT -1,
    timeout        BIGINT       NOT NULL DEFAULT 2592000,
    status         SMALLINT     NOT NULL DEFAULT 1,
    create_user    BIGINT       NOT NULL,
    create_time    TIMESTAMP    NOT NULL,
    update_user    BIGINT       DEFAULT NULL,
    update_time    TIMESTAMP    DEFAULT NULL,
    PRIMARY KEY (id)
);
CREATE UNIQUE INDEX IF NOT EXISTS uk_client_client_id  ON sys_client (client_id);
CREATE INDEX IF NOT EXISTS idx_client_create_user ON sys_client (create_user);
CREATE INDEX IF NOT EXISTS idx_client_update_user ON sys_client (update_user);
`
		if _, err := db.Exec(ddl); err != nil {
			return err
		}
	}

	// 默认客户端，行为与 Java 版保持一致（PC + ACCOUNT）。
	const seed = `
INSERT INTO sys_client (
    id, client_id, client_type, auth_type,
    active_timeout, timeout, status,
    create_user, create_time
)
SELECT 1,
       'ef51c9a3e9046c4f2ea45142c8a8344a',
       'PC',
       '["ACCOUNT"]'::json,
       1800,
       86400,
       1,
       1,
       NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_client WHERE id = 1);
`
	if _, err := db.Exec(seed); err != nil {
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

	// 如果表不存在则先创建
	if !tableName.Valid {
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
	}

	// 同步 Java 版 main_data.sql 中的默认字典：
	// notice_type（公告分类）、client_type（客户端类型）、auth_type_enum（认证类型）、storage_type_enum（存储类型）。
	const seed = `
INSERT INTO sys_dict (id, name, code, description, is_system, create_user, create_time)
SELECT 1, '公告分类', 'notice_type', NULL, TRUE, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_dict WHERE id = 1 OR code = 'notice_type');

INSERT INTO sys_dict (id, name, code, description, is_system, create_user, create_time)
SELECT 2, '客户端类型', 'client_type', NULL, TRUE, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_dict WHERE id = 2 OR code = 'client_type');

INSERT INTO sys_dict (id, name, code, description, is_system, create_user, create_time)
SELECT 3, '认证类型', 'auth_type_enum', NULL, TRUE, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_dict WHERE id = 3 OR code = 'auth_type_enum');

INSERT INTO sys_dict (id, name, code, description, is_system, create_user, create_time)
SELECT 4, '存储类型', 'storage_type_enum', NULL, TRUE, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_dict WHERE id = 4 OR code = 'storage_type_enum');
`
	if _, err := db.Exec(seed); err != nil {
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

	// 如果表不存在则先创建
	if !tableName.Valid {
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
	}

	// 初始化默认字典项：
	// - 公告分类（notice_type，dict_id=1）
	// - 客户端类型（client_type，dict_id=2）
	// - 认证类型（auth_type_enum，dict_id=3）
	// - 存储类型（storage_type_enum，dict_id=4）
	const seedItems = `
-- 公告分类
INSERT INTO sys_dict_item (
    id, label, value, color, sort, description, status,
    dict_id, create_user, create_time
)
SELECT 1, '产品新闻', '1', 'primary', 1, NULL, 1,
       1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_dict_item WHERE id = 1);

INSERT INTO sys_dict_item (
    id, label, value, color, sort, description, status,
    dict_id, create_user, create_time
)
SELECT 2, '企业动态', '2', 'success', 2, NULL, 1,
       1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_dict_item WHERE id = 2);

-- 客户端类型
INSERT INTO sys_dict_item (
    id, label, value, color, sort, description, status,
    dict_id, create_user, create_time
)
SELECT 3, '桌面端', 'PC', 'primary', 1, NULL, 1,
       2, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_dict_item WHERE id = 3);

INSERT INTO sys_dict_item (
    id, label, value, color, sort, description, status,
    dict_id, create_user, create_time
)
SELECT 4, '安卓', 'ANDROID', 'success', 2, NULL, 1,
       2, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_dict_item WHERE id = 4);

INSERT INTO sys_dict_item (
    id, label, value, color, sort, description, status,
    dict_id, create_user, create_time
)
SELECT 5, '小程序', 'XCX', 'warning', 3, NULL, 1,
       2, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_dict_item WHERE id = 5);

-- 认证类型（来自 AuthTypeEnum）
INSERT INTO sys_dict_item (
    id, label, value, color, sort, description, status,
    dict_id, create_user, create_time
)
SELECT 6, '账号', 'ACCOUNT', 'success', 1, NULL, 1,
       3, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_dict_item WHERE id = 6);

INSERT INTO sys_dict_item (
    id, label, value, color, sort, description, status,
    dict_id, create_user, create_time
)
SELECT 7, '邮箱', 'EMAIL', 'primary', 2, NULL, 1,
       3, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_dict_item WHERE id = 7);

INSERT INTO sys_dict_item (
    id, label, value, color, sort, description, status,
    dict_id, create_user, create_time
)
SELECT 8, '手机号', 'PHONE', 'primary', 3, NULL, 1,
       3, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_dict_item WHERE id = 8);

INSERT INTO sys_dict_item (
    id, label, value, color, sort, description, status,
    dict_id, create_user, create_time
)
SELECT 9, '第三方账号', 'SOCIAL', 'error', 4, NULL, 1,
       3, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_dict_item WHERE id = 9);

-- 存储类型
INSERT INTO sys_dict_item (
    id, label, value, color, sort, description, status,
    dict_id, create_user, create_time
)
SELECT 10, '本地存储', '1', 'primary', 1, NULL, 1,
       4, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_dict_item WHERE id = 10);

INSERT INTO sys_dict_item (
    id, label, value, color, sort, description, status,
    dict_id, create_user, create_time
)
SELECT 11, '对象存储', '2', 'primary', 2, NULL, 1,
       4, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM sys_dict_item WHERE id = 11);
`
	if _, err := db.Exec(seedItems); err != nil {
		return err
	}

	return nil
}

// ensureSysLog 创建 sys_log 表（结构参考 Java Postgres main_table.sql）。
func ensureSysLog(db *sql.DB) error {
	const checkTable = `SELECT to_regclass('public.sys_log');`
	var tableName sql.NullString
	if err := db.QueryRow(checkTable).Scan(&tableName); err != nil {
		return err
	}
	if tableName.Valid {
		return nil
	}

	const ddl = `
CREATE TABLE IF NOT EXISTS sys_log (
    id               BIGINT       NOT NULL,
    trace_id         VARCHAR(255) DEFAULT NULL,
    description      VARCHAR(255) NOT NULL,
    module           VARCHAR(100) NOT NULL,
    request_url      VARCHAR(512) NOT NULL,
    request_method   VARCHAR(10)  NOT NULL,
    request_headers  TEXT         DEFAULT NULL,
    request_body     TEXT         DEFAULT NULL,
    status_code      INTEGER      NOT NULL,
    response_headers TEXT         DEFAULT NULL,
    response_body    TEXT         DEFAULT NULL,
    time_taken       BIGINT       NOT NULL,
    ip               VARCHAR(100) DEFAULT NULL,
    address          VARCHAR(255) DEFAULT NULL,
    browser          VARCHAR(100) DEFAULT NULL,
    os               VARCHAR(100) DEFAULT NULL,
    status           SMALLINT     NOT NULL DEFAULT 1,
    error_msg        TEXT         DEFAULT NULL,
    create_user      BIGINT       DEFAULT NULL,
    create_time      TIMESTAMP    NOT NULL,
    PRIMARY KEY (id)
);
CREATE INDEX IF NOT EXISTS idx_log_module      ON sys_log (module);
CREATE INDEX IF NOT EXISTS idx_log_ip          ON sys_log (ip);
CREATE INDEX IF NOT EXISTS idx_log_address     ON sys_log (address);
CREATE INDEX IF NOT EXISTS idx_log_create_time ON sys_log (create_time);
`
	if _, err := db.Exec(ddl); err != nil {
		return err
	}
	return nil
}
