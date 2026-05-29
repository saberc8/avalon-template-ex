-- Seed platform data for the Rust admin backend.

INSERT INTO sys_dept
    (id, name, parent_id, ancestors, description, sort, status, is_system, create_user, create_time)
VALUES
    (1, 'Xxx科技有限公司', 0, '0', '系统初始部门', 1, 1, TRUE, 1, NOW())
ON CONFLICT DO NOTHING;

INSERT INTO sys_role
    (id, name, code, data_scope, description, sort, is_system, menu_check_strictly, dept_check_strictly, create_user, create_time)
VALUES
    (1, '系统管理员', 'admin', 1, '系统初始角色', 1, TRUE, TRUE, TRUE, 1, NOW()),
    (2, '普通用户', 'general', 4, '系统初始角色', 2, TRUE, TRUE, TRUE, 1, NOW())
ON CONFLICT DO NOTHING;

INSERT INTO sys_user
    (id, username, nickname, password, gender, email, phone, avatar, description, status, is_system, pwd_reset_time, dept_id, create_user, create_time)
VALUES
    (1, 'admin', '系统管理员', '{bcrypt}$2a$10$4jGwK2BMJ7FgVR.mgwGodey8.xR8FLoU1XSXpxJ9nZQt.pufhasSa', 1, NULL, NULL, NULL, '系统初始用户', 1, TRUE, NOW(), 1, 1, NOW())
ON CONFLICT DO NOTHING;

INSERT INTO sys_user_role (id, user_id, role_id)
VALUES (1, 1, 1)
ON CONFLICT DO NOTHING;

INSERT INTO sys_menu
    (id, title, parent_id, type, path, name, component, redirect, icon, is_external, is_cache, is_hidden, permission, sort, status, create_user, create_time)
VALUES
    (1000, '系统管理', 0, 1, '/system', 'System', 'Layout', '/system/user', 'settings', FALSE, FALSE, FALSE, NULL, 1, 1, 1, NOW()),
    (1010, '用户管理', 1000, 2, '/system/user', 'SystemUser', 'system/user/index', NULL, 'user', FALSE, FALSE, FALSE, NULL, 1, 1, 1, NOW()),
    (1011, '列表', 1010, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:user:list', 1, 1, 1, NOW()),
    (1012, '详情', 1010, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:user:get', 2, 1, 1, NOW()),
    (1013, '新增', 1010, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:user:create', 3, 1, 1, NOW()),
    (1014, '修改', 1010, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:user:update', 4, 1, 1, NOW()),
    (1015, '删除', 1010, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:user:delete', 5, 1, 1, NOW()),
    (1016, '导出', 1010, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:user:export', 6, 1, 1, NOW()),
    (1017, '导入', 1010, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:user:import', 7, 1, 1, NOW()),
    (1018, '重置密码', 1010, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:user:resetPwd', 8, 1, 1, NOW()),
    (1019, '分配角色', 1010, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:user:updateRole', 9, 1, 1, NOW()),

    (1030, '角色管理', 1000, 2, '/system/role', 'SystemRole', 'system/role/index', NULL, 'user-group', FALSE, FALSE, FALSE, NULL, 2, 1, 1, NOW()),
    (1031, '列表', 1030, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:role:list', 1, 1, 1, NOW()),
    (1032, '详情', 1030, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:role:get', 2, 1, 1, NOW()),
    (1033, '新增', 1030, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:role:create', 3, 1, 1, NOW()),
    (1034, '修改', 1030, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:role:update', 4, 1, 1, NOW()),
    (1035, '删除', 1030, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:role:delete', 5, 1, 1, NOW()),
    (1036, '修改权限', 1030, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:role:updatePermission', 6, 1, 1, NOW()),
    (1037, '分配', 1030, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:role:assign', 7, 1, 1, NOW()),
    (1038, '取消分配', 1030, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:role:unassign', 8, 1, 1, NOW()),

    (1050, '菜单管理', 1000, 2, '/system/menu', 'SystemMenu', 'system/menu/index', NULL, 'menu', FALSE, FALSE, FALSE, NULL, 3, 1, 1, NOW()),
    (1051, '列表', 1050, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:menu:list', 1, 1, 1, NOW()),
    (1052, '详情', 1050, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:menu:get', 2, 1, 1, NOW()),
    (1053, '新增', 1050, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:menu:create', 3, 1, 1, NOW()),
    (1054, '修改', 1050, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:menu:update', 4, 1, 1, NOW()),
    (1055, '删除', 1050, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:menu:delete', 5, 1, 1, NOW()),
    (1056, '清除缓存', 1050, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:menu:clearCache', 6, 1, 1, NOW()),

    (1070, '部门管理', 1000, 2, '/system/dept', 'SystemDept', 'system/dept/index', NULL, 'mind-mapping', FALSE, FALSE, FALSE, NULL, 4, 1, 1, NOW()),
    (1071, '列表', 1070, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dept:list', 1, 1, 1, NOW()),
    (1072, '详情', 1070, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dept:get', 2, 1, 1, NOW()),
    (1073, '新增', 1070, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dept:create', 3, 1, 1, NOW()),
    (1074, '修改', 1070, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dept:update', 4, 1, 1, NOW()),
    (1075, '删除', 1070, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dept:delete', 5, 1, 1, NOW()),
    (1076, '导出', 1070, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dept:export', 6, 1, 1, NOW()),

    (1110, '文件管理', 1000, 2, '/system/file', 'SystemFile', 'system/file/index', NULL, 'file', FALSE, FALSE, FALSE, NULL, 6, 1, 1, NOW()),
    (1111, '列表', 1110, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:file:list', 1, 1, 1, NOW()),
    (1112, '详情', 1110, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:file:get', 2, 1, 1, NOW()),
    (1113, '上传', 1110, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:file:upload', 3, 1, 1, NOW()),
    (1114, '修改', 1110, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:file:update', 4, 1, 1, NOW()),
    (1115, '删除', 1110, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:file:delete', 5, 1, 1, NOW()),
    (1116, '下载', 1110, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:file:download', 6, 1, 1, NOW()),
    (1117, '创建文件夹', 1110, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:file:createDir', 7, 1, 1, NOW()),
    (1118, '计算文件夹大小', 1110, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:file:calcDirSize', 8, 1, 1, NOW()),

    (1130, '字典管理', 1000, 2, '/system/dict', 'SystemDict', 'system/dict/index', NULL, 'bookmark', FALSE, FALSE, FALSE, NULL, 7, 1, 1, NOW()),
    (1131, '列表', 1130, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dict:list', 1, 1, 1, NOW()),
    (1132, '详情', 1130, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dict:get', 2, 1, 1, NOW()),
    (1133, '新增', 1130, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dict:create', 3, 1, 1, NOW()),
    (1134, '修改', 1130, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dict:update', 4, 1, 1, NOW()),
    (1135, '删除', 1130, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dict:delete', 5, 1, 1, NOW()),
    (1136, '清除缓存', 1130, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dict:item:clearCache', 6, 1, 1, NOW()),

    (1140, '字典项管理', 1000, 2, '/system/dict/item', 'SystemDictItem', 'system/dict/item/index', NULL, 'bookmark', FALSE, FALSE, TRUE, NULL, 8, 1, 1, NOW()),
    (1141, '列表', 1140, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dict:item:list', 1, 1, 1, NOW()),
    (1142, '详情', 1140, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dict:item:get', 2, 1, 1, NOW()),
    (1143, '新增', 1140, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dict:item:create', 3, 1, 1, NOW()),
    (1144, '修改', 1140, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dict:item:update', 4, 1, 1, NOW()),
    (1145, '删除', 1140, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:dict:item:delete', 5, 1, 1, NOW()),

    (1150, '系统配置', 1000, 2, '/system/config', 'SystemConfig', 'system/config/index', NULL, 'config', FALSE, FALSE, FALSE, NULL, 999, 1, 1, NOW()),
    (1160, '网站配置', 1150, 2, '/system/config?tab=site', 'SystemSiteConfig', 'system/config/site/index', NULL, 'apps', FALSE, FALSE, TRUE, NULL, 1, 1, 1, NOW()),
    (1161, '查询', 1160, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:siteConfig:get', 1, 1, 1, NOW()),
    (1162, '修改', 1160, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:siteConfig:update', 2, 1, 1, NOW()),
    (1170, '安全配置', 1150, 2, '/system/config?tab=security', 'SystemSecurityConfig', 'system/config/security/index', NULL, 'safe', FALSE, FALSE, TRUE, NULL, 2, 1, 1, NOW()),
    (1171, '查询', 1170, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:securityConfig:get', 1, 1, 1, NOW()),
    (1172, '修改', 1170, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:securityConfig:update', 2, 1, 1, NOW()),
    (1180, '登录配置', 1150, 2, '/system/config?tab=login', 'SystemLoginConfig', 'system/config/login/index', NULL, 'lock', FALSE, FALSE, TRUE, NULL, 3, 1, 1, NOW()),
    (1181, '查询', 1180, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:loginConfig:get', 1, 1, 1, NOW()),
    (1182, '修改', 1180, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:loginConfig:update', 2, 1, 1, NOW()),
    (1230, '存储配置', 1150, 2, '/system/config?tab=storage', 'SystemStorage', 'system/config/storage/index', NULL, 'storage', FALSE, FALSE, TRUE, NULL, 6, 1, 1, NOW()),
    (1231, '列表', 1230, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:storage:list', 1, 1, 1, NOW()),
    (1232, '详情', 1230, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:storage:get', 2, 1, 1, NOW()),
    (1233, '新增', 1230, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:storage:create', 3, 1, 1, NOW()),
    (1234, '修改', 1230, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:storage:update', 4, 1, 1, NOW()),
    (1235, '删除', 1230, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:storage:delete', 5, 1, 1, NOW()),
    (1236, '修改状态', 1230, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:storage:updateStatus', 6, 1, 1, NOW()),
    (1237, '设为默认存储', 1230, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:storage:setDefault', 7, 1, 1, NOW()),
    (1250, '客户端配置', 1150, 2, '/system/config?tab=client', 'SystemClient', 'system/config/client/index', NULL, 'mobile', FALSE, FALSE, TRUE, NULL, 7, 1, 1, NOW()),
    (1251, '列表', 1250, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:client:list', 1, 1, 1, NOW()),
    (1252, '详情', 1250, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:client:get', 2, 1, 1, NOW()),
    (1253, '新增', 1250, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:client:create', 3, 1, 1, NOW()),
    (1254, '修改', 1250, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:client:update', 4, 1, 1, NOW()),
    (1255, '删除', 1250, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system:client:delete', 5, 1, 1, NOW()),

    (2000, '系统监控', 0, 1, '/monitor', 'Monitor', 'Layout', '/monitor/online', 'computer', FALSE, FALSE, FALSE, NULL, 2, 1, 1, NOW()),
    (2010, '在线用户', 2000, 2, '/monitor/online', 'MonitorOnline', 'monitor/online/index', NULL, 'user', FALSE, FALSE, FALSE, NULL, 1, 1, 1, NOW()),
    (2011, '列表', 2010, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'monitor:online:list', 1, 1, 1, NOW()),
    (2012, '强退', 2010, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'monitor:online:kickout', 2, 1, 1, NOW()),
    (2030, '系统日志', 2000, 2, '/monitor/log', 'MonitorLog', 'monitor/log/index', NULL, 'history', FALSE, FALSE, FALSE, NULL, 2, 1, 1, NOW()),
    (2031, '列表', 2030, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'monitor:log:list', 1, 1, 1, NOW()),
    (2032, '详情', 2030, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'monitor:log:get', 2, 1, 1, NOW()),
    (2033, '导出', 2030, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'monitor:log:export', 3, 1, 1, NOW())
ON CONFLICT DO NOTHING;

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 1, id
FROM sys_menu
ON CONFLICT DO NOTHING;

INSERT INTO sys_role_dept (role_id, dept_id)
VALUES (2, 1)
ON CONFLICT DO NOTHING;

INSERT INTO sys_dict
    (id, name, code, description, is_system, create_user, create_time)
VALUES
    (100, '用户性别', 'user_gender', NULL, TRUE, 1, NOW()),
    (101, '用户状态', 'user_status', NULL, TRUE, 1, NOW()),
    (102, '客户端类型', 'client_type', NULL, TRUE, 1, NOW()),
    (103, '认证类型', 'auth_type_enum', NULL, TRUE, 1, NOW()),
    (104, '存储类型', 'storage_type_enum', NULL, TRUE, 1, NOW()),
    (105, '数据权限', 'data_scope_enum', NULL, TRUE, 1, NOW())
ON CONFLICT DO NOTHING;

INSERT INTO sys_dict_item
    (id, label, value, color, sort, description, status, dict_id, create_user, create_time)
VALUES
    (1000, '男', '1', 'primary', 1, NULL, 1, 100, 1, NOW()),
    (1001, '女', '2', 'error', 2, NULL, 1, 100, 1, NOW()),
    (1002, '未知', '0', 'default', 3, NULL, 1, 100, 1, NOW()),

    (1010, '启用', '1', 'success', 1, NULL, 1, 101, 1, NOW()),
    (1011, '禁用', '2', 'error', 2, NULL, 1, 101, 1, NOW()),

    (1020, '桌面端', 'PC', 'primary', 1, NULL, 1, 102, 1, NOW()),
    (1021, '安卓', 'ANDROID', 'success', 2, NULL, 1, 102, 1, NOW()),
    (1022, '小程序', 'XCX', 'warning', 3, NULL, 1, 102, 1, NOW()),

    (1030, '账号', 'ACCOUNT', 'success', 1, NULL, 1, 103, 1, NOW()),
    (1031, '邮箱', 'EMAIL', 'primary', 2, NULL, 1, 103, 1, NOW()),
    (1032, '手机号', 'PHONE', 'primary', 3, NULL, 1, 103, 1, NOW()),
    (1033, '第三方账号', 'SOCIAL', 'error', 4, NULL, 1, 103, 1, NOW()),

    (1040, '本地存储', '1', 'primary', 1, NULL, 1, 104, 1, NOW()),
    (1041, '对象存储', '2', 'primary', 2, NULL, 1, 104, 1, NOW()),

    (1050, '全部数据权限', '1', 'success', 1, NULL, 1, 105, 1, NOW()),
    (1051, '本部门及以下数据权限', '2', 'primary', 2, NULL, 1, 105, 1, NOW()),
    (1052, '本部门数据权限', '3', 'warning', 3, NULL, 1, 105, 1, NOW()),
    (1053, '仅本人数据权限', '4', 'default', 4, NULL, 1, 105, 1, NOW()),
    (1054, '自定义数据权限', '5', 'error', 5, NULL, 1, 105, 1, NOW())
ON CONFLICT DO NOTHING;

INSERT INTO sys_option
    (id, category, name, code, value, default_value, description)
VALUES
    (1, 'SITE', '系统名称', 'SITE_TITLE', NULL, 'ContiNew Admin', '显示在浏览器标题栏和登录界面的系统名称'),
    (2, 'SITE', '系统描述', 'SITE_DESCRIPTION', NULL, '持续迭代优化的前后端分离中后台管理系统框架', '用于 SEO 的网站元描述'),
    (3, 'SITE', '版权声明', 'SITE_COPYRIGHT', NULL, 'Copyright (c) 2022 - present ContiNew Admin 版权所有', '显示在页面底部的版权声明文本'),
    (4, 'SITE', '备案号', 'SITE_BEIAN', NULL, NULL, '工信部 ICP 备案编号'),
    (5, 'SITE', '系统图标', 'SITE_FAVICON', NULL, '/favicon.ico', '浏览器标签页显示的网站图标'),
    (6, 'SITE', '系统LOGO', 'SITE_LOGO', NULL, '/logo.svg', '显示在登录页面和系统导航栏的网站图标'),
    (10, 'PASSWORD', '密码错误锁定阈值', 'PASSWORD_ERROR_LOCK_COUNT', NULL, '5', '连续登录失败次数达到该值将锁定账号'),
    (11, 'PASSWORD', '账号锁定时长（分钟）', 'PASSWORD_ERROR_LOCK_MINUTES', NULL, '5', '账号锁定后自动解锁的时间'),
    (12, 'PASSWORD', '密码有效期（天）', 'PASSWORD_EXPIRATION_DAYS', NULL, '0', '密码强制修改周期'),
    (13, 'PASSWORD', '密码到期提醒（天）', 'PASSWORD_EXPIRATION_WARNING_DAYS', NULL, '0', '密码过期前的提前提醒天数'),
    (14, 'PASSWORD', '历史密码重复校验次数', 'PASSWORD_REPETITION_TIMES', NULL, '3', '禁止使用最近 N 次的历史密码'),
    (15, 'PASSWORD', '密码最小长度', 'PASSWORD_MIN_LENGTH', NULL, '8', '密码最小字符长度要求'),
    (16, 'PASSWORD', '是否允许密码包含用户名', 'PASSWORD_ALLOW_CONTAIN_USERNAME', NULL, '1', '是否允许密码包含正序或倒序的用户名字符'),
    (17, 'PASSWORD', '密码是否必须包含特殊字符', 'PASSWORD_REQUIRE_SYMBOLS', NULL, '0', '是否要求密码必须包含特殊字符'),
    (27, 'LOGIN', '是否启用验证码', 'LOGIN_CAPTCHA_ENABLED', NULL, '1', NULL)
ON CONFLICT DO NOTHING;

INSERT INTO sys_storage
    (id, name, code, type, access_key, secret_key, endpoint, region, bucket_name, domain, description, is_default, sort, status, create_user, create_time)
VALUES
    (1, '开发环境', 'local_dev', 1, NULL, NULL, NULL, NULL, './data/file/', '/file/', '本地存储', TRUE, 1, 1, 1, NOW())
ON CONFLICT DO NOTHING;

INSERT INTO sys_client
    (id, client_id, client_type, auth_type, active_timeout, timeout, status, create_user, create_time)
VALUES
    (1, 'ef51c9a3e9046c4f2ea45142c8a8344a', 'PC', '["ACCOUNT"]'::json, 1800, 86400, 1, 1, NOW())
ON CONFLICT DO NOTHING;
