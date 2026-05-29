-- Core system schema for the Rust admin backend.

CREATE TABLE IF NOT EXISTS sys_user (
    id             BIGINT       NOT NULL,
    username       VARCHAR(64)  NOT NULL,
    nickname       VARCHAR(30)  NOT NULL,
    password       VARCHAR(255) DEFAULT NULL,
    gender         SMALLINT     NOT NULL DEFAULT 0,
    email          VARCHAR(255) DEFAULT NULL,
    phone          VARCHAR(255) DEFAULT NULL,
    avatar         TEXT         DEFAULT NULL,
    description    VARCHAR(200) DEFAULT NULL,
    status         SMALLINT     NOT NULL DEFAULT 1,
    is_system      BOOLEAN      NOT NULL DEFAULT FALSE,
    pwd_reset_time TIMESTAMP    DEFAULT NULL,
    dept_id        BIGINT       NOT NULL,
    create_user    BIGINT       DEFAULT NULL,
    create_time    TIMESTAMP    NOT NULL,
    update_user    BIGINT       DEFAULT NULL,
    update_time    TIMESTAMP    DEFAULT NULL,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_user_username ON sys_user (username);
CREATE UNIQUE INDEX IF NOT EXISTS uk_user_email ON sys_user (email);
CREATE UNIQUE INDEX IF NOT EXISTS uk_user_phone ON sys_user (phone);
CREATE INDEX IF NOT EXISTS idx_user_dept_id ON sys_user (dept_id);
CREATE INDEX IF NOT EXISTS idx_user_create_user ON sys_user (create_user);
CREATE INDEX IF NOT EXISTS idx_user_update_user ON sys_user (update_user);

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

CREATE UNIQUE INDEX IF NOT EXISTS uk_role_name ON sys_role (name);
CREATE UNIQUE INDEX IF NOT EXISTS uk_role_code ON sys_role (code);
CREATE INDEX IF NOT EXISTS idx_role_create_user ON sys_role (create_user);
CREATE INDEX IF NOT EXISTS idx_role_update_user ON sys_role (update_user);

CREATE TABLE IF NOT EXISTS sys_user_role (
    id      BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    role_id BIGINT NOT NULL,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_user_id_role_id ON sys_user_role (user_id, role_id);

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

CREATE INDEX IF NOT EXISTS idx_menu_parent_id ON sys_menu (parent_id);
CREATE INDEX IF NOT EXISTS idx_menu_permission ON sys_menu (permission);
CREATE INDEX IF NOT EXISTS idx_menu_create_user ON sys_menu (create_user);
CREATE INDEX IF NOT EXISTS idx_menu_update_user ON sys_menu (update_user);
CREATE UNIQUE INDEX IF NOT EXISTS uk_menu_title_parent_id ON sys_menu (title, parent_id);

CREATE TABLE IF NOT EXISTS sys_role_menu (
    role_id BIGINT NOT NULL,
    menu_id BIGINT NOT NULL,
    PRIMARY KEY (role_id, menu_id)
);

CREATE TABLE IF NOT EXISTS sys_role_dept (
    role_id BIGINT NOT NULL,
    dept_id BIGINT NOT NULL,
    PRIMARY KEY (role_id, dept_id)
);

CREATE INDEX IF NOT EXISTS idx_role_dept_role_id ON sys_role_dept (role_id);
CREATE INDEX IF NOT EXISTS idx_role_dept_dept_id ON sys_role_dept (dept_id);

CREATE TABLE IF NOT EXISTS sys_dept (
    id          BIGINT       NOT NULL,
    name        VARCHAR(30)  NOT NULL,
    parent_id   BIGINT       NOT NULL DEFAULT 0,
    ancestors   VARCHAR(512) NOT NULL DEFAULT '',
    description VARCHAR(200) DEFAULT NULL,
    sort        INTEGER      NOT NULL DEFAULT 999,
    status      SMALLINT     NOT NULL DEFAULT 1,
    is_system   BOOLEAN      NOT NULL DEFAULT FALSE,
    create_user BIGINT       NOT NULL,
    create_time TIMESTAMP    NOT NULL,
    update_user BIGINT       DEFAULT NULL,
    update_time TIMESTAMP    DEFAULT NULL,
    PRIMARY KEY (id)
);

ALTER TABLE sys_dept ADD COLUMN IF NOT EXISTS ancestors VARCHAR(512) NOT NULL DEFAULT '';
CREATE INDEX IF NOT EXISTS idx_dept_parent_id ON sys_dept (parent_id);
CREATE INDEX IF NOT EXISTS idx_dept_create_user ON sys_dept (create_user);
CREATE INDEX IF NOT EXISTS idx_dept_update_user ON sys_dept (update_user);
CREATE UNIQUE INDEX IF NOT EXISTS uk_dept_name_parent_id ON sys_dept (name, parent_id);

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

CREATE UNIQUE INDEX IF NOT EXISTS uk_dict_name ON sys_dict (name);
CREATE UNIQUE INDEX IF NOT EXISTS uk_dict_code ON sys_dict (code);
CREATE INDEX IF NOT EXISTS idx_dict_create_user ON sys_dict (create_user);
CREATE INDEX IF NOT EXISTS idx_dict_update_user ON sys_dict (update_user);

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

CREATE UNIQUE INDEX IF NOT EXISTS uk_dict_item_value_dict_id ON sys_dict_item (value, dict_id);
CREATE INDEX IF NOT EXISTS idx_dict_item_dict_id ON sys_dict_item (dict_id);
CREATE INDEX IF NOT EXISTS idx_dict_item_create_user ON sys_dict_item (create_user);
CREATE INDEX IF NOT EXISTS idx_dict_item_update_user ON sys_dict_item (update_user);

CREATE TABLE IF NOT EXISTS sys_option (
    id            BIGINT       NOT NULL,
    category      VARCHAR(50)  NOT NULL,
    name          VARCHAR(50)  NOT NULL,
    code          VARCHAR(100) NOT NULL,
    value         TEXT         DEFAULT NULL,
    default_value TEXT         DEFAULT NULL,
    description   VARCHAR(200) DEFAULT NULL,
    update_user   BIGINT       DEFAULT NULL,
    update_time   TIMESTAMP    DEFAULT NULL,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_option_category_code ON sys_option (category, code);

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

ALTER TABLE sys_storage ADD COLUMN IF NOT EXISTS region VARCHAR(100) DEFAULT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uk_storage_code ON sys_storage (code);
CREATE INDEX IF NOT EXISTS idx_storage_create_user ON sys_storage (create_user);
CREATE INDEX IF NOT EXISTS idx_storage_update_user ON sys_storage (update_user);

CREATE TABLE IF NOT EXISTS sys_file (
    id                 BIGINT       NOT NULL,
    name               VARCHAR(255) NOT NULL,
    original_name      VARCHAR(255) NOT NULL,
    size               BIGINT       DEFAULT NULL,
    parent_path        VARCHAR(512) NOT NULL DEFAULT '/',
    path               VARCHAR(512) NOT NULL,
    extension          VARCHAR(100) DEFAULT NULL,
    content_type       VARCHAR(255) DEFAULT NULL,
    type               SMALLINT     NOT NULL DEFAULT 1,
    sha256             VARCHAR(256) NOT NULL,
    metadata           TEXT         DEFAULT NULL,
    thumbnail_name     VARCHAR(255) DEFAULT NULL,
    thumbnail_size     BIGINT       DEFAULT NULL,
    thumbnail_metadata TEXT         DEFAULT NULL,
    storage_id         BIGINT       NOT NULL,
    create_user        BIGINT       NOT NULL,
    create_time        TIMESTAMP    NOT NULL,
    update_user        BIGINT       DEFAULT NULL,
    update_time        TIMESTAMP    DEFAULT NULL,
    PRIMARY KEY (id)
);

CREATE INDEX IF NOT EXISTS idx_file_type ON sys_file (type);
CREATE INDEX IF NOT EXISTS idx_file_sha256 ON sys_file (sha256);
CREATE INDEX IF NOT EXISTS idx_file_parent_path ON sys_file (parent_path);
CREATE INDEX IF NOT EXISTS idx_file_storage_id ON sys_file (storage_id);
CREATE INDEX IF NOT EXISTS idx_file_create_user ON sys_file (create_user);

CREATE TABLE IF NOT EXISTS sys_client (
    id             BIGINT      NOT NULL,
    client_id      VARCHAR(50) NOT NULL,
    client_type    VARCHAR(50) NOT NULL,
    auth_type      JSON        NOT NULL,
    active_timeout BIGINT      NOT NULL DEFAULT -1,
    timeout        BIGINT      NOT NULL DEFAULT 2592000,
    status         SMALLINT    NOT NULL DEFAULT 1,
    create_user    BIGINT      NOT NULL,
    create_time    TIMESTAMP   NOT NULL,
    update_user    BIGINT      DEFAULT NULL,
    update_time    TIMESTAMP   DEFAULT NULL,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_client_client_id ON sys_client (client_id);
CREATE INDEX IF NOT EXISTS idx_client_create_user ON sys_client (create_user);
CREATE INDEX IF NOT EXISTS idx_client_update_user ON sys_client (update_user);

CREATE TABLE IF NOT EXISTS sys_log (
    id               BIGINT       NOT NULL,
    trace_id         VARCHAR(255) DEFAULT NULL,
    description      VARCHAR(255) NOT NULL,
    module           VARCHAR(100) NOT NULL,
    type             SMALLINT     NOT NULL DEFAULT 1,
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

ALTER TABLE sys_log ADD COLUMN IF NOT EXISTS type SMALLINT NOT NULL DEFAULT 1;
CREATE INDEX IF NOT EXISTS idx_log_create_user ON sys_log (create_user);
CREATE INDEX IF NOT EXISTS idx_log_create_time ON sys_log (create_time);
CREATE INDEX IF NOT EXISTS idx_log_module ON sys_log (module);
CREATE INDEX IF NOT EXISTS idx_log_type ON sys_log (type);
CREATE INDEX IF NOT EXISTS idx_log_ip ON sys_log (ip);

CREATE TABLE IF NOT EXISTS sys_online_user (
    id               BIGINT       NOT NULL,
    token            VARCHAR(512) NOT NULL,
    user_id          BIGINT       NOT NULL,
    username         VARCHAR(64)  NOT NULL,
    nickname         VARCHAR(30)  NOT NULL,
    client_type      VARCHAR(50)  NOT NULL DEFAULT 'PC',
    client_id        VARCHAR(50)  NOT NULL,
    ip               VARCHAR(100) DEFAULT NULL,
    address          VARCHAR(255) DEFAULT NULL,
    browser          VARCHAR(100) DEFAULT NULL,
    os               VARCHAR(100) DEFAULT NULL,
    login_time       TIMESTAMP    NOT NULL,
    last_active_time TIMESTAMP    NOT NULL,
    create_time      TIMESTAMP    NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_online_user_token ON sys_online_user (token);
CREATE INDEX IF NOT EXISTS idx_online_user_user_id ON sys_online_user (user_id);
CREATE INDEX IF NOT EXISTS idx_online_user_login_time ON sys_online_user (login_time);
CREATE INDEX IF NOT EXISTS idx_online_user_last_active_time ON sys_online_user (last_active_time);
