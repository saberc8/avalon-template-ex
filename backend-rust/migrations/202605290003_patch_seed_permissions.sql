-- Patch seed data gaps found after the initial core seed migration.

INSERT INTO sys_menu
    (id, title, parent_id, type, path, name, component, redirect, icon, is_external, is_cache, is_hidden, permission, sort, status, create_user, create_time)
SELECT
    1119, '秒传检测', 1110, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:file:check', 9, 1, 1, NOW()
WHERE NOT EXISTS (
    SELECT 1
    FROM sys_menu
    WHERE id = 1119 OR permission = 'system:file:check'
);

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 1, id
FROM sys_menu
WHERE permission = 'system:file:check'
ON CONFLICT DO NOTHING;

INSERT INTO sys_role
    (id, name, code, data_scope, description, sort, is_system, menu_check_strictly, dept_check_strictly, create_user, create_time)
SELECT
    3, '自定义部门角色', 'custom_dept', 5, '用于数据权限自定义部门范围校验', 3, FALSE, TRUE, TRUE, 1, NOW()
WHERE NOT EXISTS (
    SELECT 1
    FROM sys_role
    WHERE id = 3 OR code = 'custom_dept'
);

INSERT INTO sys_role_dept (role_id, dept_id)
SELECT id, 1
FROM sys_role
WHERE code = 'custom_dept'
ON CONFLICT DO NOTHING;
