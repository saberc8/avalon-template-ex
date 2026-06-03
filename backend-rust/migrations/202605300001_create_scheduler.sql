-- Scheduled jobs, durable triggers, execution logs, and RBAC seed.

CREATE TABLE IF NOT EXISTS sys_job (
    id                BIGINT       NOT NULL,
    name              VARCHAR(100) NOT NULL,
    group_name        VARCHAR(50)  NOT NULL DEFAULT 'default',
    task_type         SMALLINT     NOT NULL DEFAULT 1,
    cron_expression   VARCHAR(120) NOT NULL,
    status            SMALLINT     NOT NULL DEFAULT 2,
    concurrent        BOOLEAN      NOT NULL DEFAULT FALSE,
    misfire_policy    SMALLINT     NOT NULL DEFAULT 1,
    max_retry         INTEGER      NOT NULL DEFAULT 0,
    timeout_seconds   INTEGER      NOT NULL DEFAULT 30,
    http_method       VARCHAR(10)  DEFAULT NULL,
    http_url          VARCHAR(1000) DEFAULT NULL,
    http_headers      JSONB        NOT NULL DEFAULT '{}'::jsonb,
    http_body         TEXT         DEFAULT NULL,
    builtin_key       VARCHAR(120) DEFAULT NULL,
    description       VARCHAR(255) DEFAULT NULL,
    last_trigger_time TIMESTAMP    DEFAULT NULL,
    next_trigger_time TIMESTAMP    DEFAULT NULL,
    create_user       BIGINT       NOT NULL,
    create_time       TIMESTAMP    NOT NULL,
    update_user       BIGINT       DEFAULT NULL,
    update_time       TIMESTAMP    DEFAULT NULL,
    PRIMARY KEY (id)
);

CREATE INDEX IF NOT EXISTS idx_job_status_next_trigger ON sys_job (status, next_trigger_time);
CREATE INDEX IF NOT EXISTS idx_job_group_name ON sys_job (group_name);
CREATE INDEX IF NOT EXISTS idx_job_task_type ON sys_job (task_type);

CREATE TABLE IF NOT EXISTS sys_job_trigger (
    id           BIGINT    NOT NULL,
    job_id       BIGINT    NOT NULL,
    source       SMALLINT  NOT NULL DEFAULT 1,
    fire_time    TIMESTAMP NOT NULL,
    status       SMALLINT  NOT NULL DEFAULT 1,
    attempt      INTEGER   NOT NULL DEFAULT 0,
    max_attempts INTEGER   NOT NULL DEFAULT 1,
    payload      JSONB     NOT NULL DEFAULT '{}'::jsonb,
    trace_id     VARCHAR(64) DEFAULT NULL,
    error_msg    TEXT      DEFAULT NULL,
    queued_time  TIMESTAMP DEFAULT NULL,
    start_time   TIMESTAMP DEFAULT NULL,
    finish_time  TIMESTAMP DEFAULT NULL,
    create_time  TIMESTAMP NOT NULL,
    PRIMARY KEY (id)
);

CREATE INDEX IF NOT EXISTS idx_job_trigger_job_id ON sys_job_trigger (job_id);
CREATE INDEX IF NOT EXISTS idx_job_trigger_status ON sys_job_trigger (status);
CREATE INDEX IF NOT EXISTS idx_job_trigger_fire_time ON sys_job_trigger (fire_time);

CREATE TABLE IF NOT EXISTS sys_job_log (
    id               BIGINT    NOT NULL,
    trigger_id       BIGINT    NOT NULL,
    job_id           BIGINT    NOT NULL,
    attempt          INTEGER   NOT NULL,
    status           SMALLINT  NOT NULL,
    executor         VARCHAR(100) DEFAULT NULL,
    request_snapshot JSONB     NOT NULL DEFAULT '{}'::jsonb,
    response_status  INTEGER   DEFAULT NULL,
    response_body    TEXT      DEFAULT NULL,
    error_msg        TEXT      DEFAULT NULL,
    start_time       TIMESTAMP NOT NULL,
    finish_time      TIMESTAMP DEFAULT NULL,
    time_taken       BIGINT    NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE INDEX IF NOT EXISTS idx_job_log_job_id ON sys_job_log (job_id);
CREATE INDEX IF NOT EXISTS idx_job_log_trigger_id ON sys_job_log (trigger_id);
CREATE INDEX IF NOT EXISTS idx_job_log_status ON sys_job_log (status);
CREATE INDEX IF NOT EXISTS idx_job_log_start_time ON sys_job_log (start_time);

INSERT INTO sys_menu
    (id, title, parent_id, type, path, name, component, redirect, icon, is_external, is_cache, is_hidden, permission, sort, status, create_user, create_time)
VALUES
    (2050, '定时任务', 2000, 2, '/schedule/job', 'ScheduleJob', 'schedule/job/index', NULL, 'clock', FALSE, FALSE, FALSE, NULL, 3, 1, 1, NOW()),
    (2051, '列表', 2050, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'schedule:job:list', 1, 1, 1, NOW()),
    (2052, '详情', 2050, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'schedule:job:get', 2, 1, 1, NOW()),
    (2053, '新增', 2050, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'schedule:job:create', 3, 1, 1, NOW()),
    (2054, '修改', 2050, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'schedule:job:update', 4, 1, 1, NOW()),
    (2055, '删除', 2050, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'schedule:job:delete', 5, 1, 1, NOW()),
    (2056, '修改状态', 2050, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'schedule:job:updateStatus', 6, 1, 1, NOW()),
    (2057, '执行', 2050, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'schedule:job:run', 7, 1, 1, NOW()),
    (2058, '日志', 2050, 3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'schedule:job:log:list', 8, 1, 1, NOW())
ON CONFLICT DO NOTHING;

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 1, id
FROM sys_menu
WHERE id BETWEEN 2050 AND 2058
ON CONFLICT DO NOTHING;
