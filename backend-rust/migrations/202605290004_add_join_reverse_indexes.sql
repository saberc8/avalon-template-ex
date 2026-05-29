-- Add reverse indexes needed by later RBAC/menu CRUD and clean misleading seed data.

CREATE INDEX IF NOT EXISTS idx_user_role_role_id ON sys_user_role (role_id);
CREATE INDEX IF NOT EXISTS idx_role_menu_menu_id ON sys_role_menu (menu_id);

DELETE FROM sys_role_dept
WHERE role_id IN (
    SELECT id
    FROM sys_role
    WHERE code = 'general' AND data_scope = 4
);
