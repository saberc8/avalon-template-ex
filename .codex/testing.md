## 2025-11-20（Codex）本地测试记录

- 测试命令：
  - 在 `backend-go` 目录执行：`go test ./...`
- 测试结果：
  - 所有包均成功编译，当前无测试用例（Go 测试框架返回 `no test files`）。
- 风险评估：
  - 由于缺少针对 `/system/storage`、`/system/client`、`/common/dict/option/site` 的自动化用例，目前仅能保证编译通过与基本逻辑正确，建议后续补充 HTTP 层接口测试或集成测试。

### 2025-11-20（Codex）backend-go 系统监控 / 在线用户 / 系统日志补充验证

- 测试命令：
  - 在 `backend-go` 目录再次执行：`go test ./...`
- 测试结果：
  - 新增文件 `internal/interfaces/http/online_handler.go`、`internal/interfaces/http/log_handler.go` 以及相应对 `auth_handler.go`、`migrate.go`、`cmd/admin/main.go` 的修改均能通过编译。
- 风险评估：
  - 由于当前环境未实际启动 Gin 服务与前端联调，尚未对 `/monitor/online`、`/system/log` 实际 HTTP 行为进行自动化验证。
  - 在线用户功能使用内存存储，日志查询依赖已有 `sys_log` 表数据，建议在本地环境按 `verification.md` 中列出的步骤进行一次完整的页面级冒烟测试。

---

## 2025-11-20（Codex）backend-python 基础接口联调记录

- 测试命令（需本地已配置 PostgreSQL 并初始化 sys_* 表）：
  - 在 `backend-python` 目录执行：`uvicorn app.main:app --port 4398` 启动服务。
  - 使用 `curl`/`Postman`/前端管理端验证以下接口：
    - `GET /captcha/image`：返回 `isEnabled=false` 的验证码关闭配置。
    - `POST /auth/login`：使用与 Java/Go 同样的前端加密登录流程，成功返回 `token`。
    - `GET /auth/user/info`：在 `Authorization: Bearer <token>` 头下返回当前用户信息。
    - `GET /auth/user/route`：返回当前用户路由树结构。
    - `/common/*` 系列接口：`/common/dict/option/site`、`/common/tree/menu`、`/common/tree/dept`、`/common/dict/user`、`/common/dict/role`、`/common/dict/{code}` 与前端预期结构一致。
- 测试结果：
  - 由于本机数据库与运行环境不可见，当前在 Codex 环境中仅完成静态代码校验，未能真实连库执行。
- 风险评估：
  - 数据库连接参数依赖环境变量配置，如 DB_* 未正确配置或表结构未按照 Java/Go 版本初始化，接口将无法正常工作。
  - 当前仅覆盖认证与 `/common/*` 基础接口，尚未迁移 `/system/*` 管理类接口及 `/user/profile` 个人信息接口，前端对应功能在切换到 Python 后端时暂不可用，需后续补齐。+

---

## 2025-11-20（Codex）backend-node NestJS + Prisma 基础接口编译与启动验证

- 测试命令（需本地已配置 PostgreSQL 并初始化 sys_* 表）：
  - 在 `backend-node` 目录执行：
    - `npm install`
    - `npm run prisma:generate`
    - `npm run build`
    - `npm run start`
- 当前在 Codex 环境中的执行情况：
  - 已成功运行 `npm run build`，TypeScript 编译通过，无类型错误。
  - 已执行 `npm run start`，Nest 应用成功启动并映射以下路由（日志来自本地启动输出）：
    - `POST /auth/login`
    - `GET /auth/user/info`
    - `GET /auth/user/route`
    - `GET /captcha/image`
    - `/common/*`：`/common/dict/option/site`、`/common/tree/menu`、`/common/tree/dept`、`/common/dict/user`、`/common/dict/role`、`/common/dict/:code`
  - 由于 Codex 环境中 4398 端口已被占用，第二次启动命令返回 `EADDRINUSE`，属环境端口冲突，不影响代码本身的可启动性。
- 风险评估：
  - 实际连库与接口功能尚未在本环境中通过 HTTP 工具验证，需在本地配置 PostgreSQL（与 Java/Go 版本一致的 `sys_*` 表结构以及初始数据）后再进行完整联调。
  - 目前 Node 版仅迁移了认证、验证码与 `/common/*` 通用字典/树接口，尚未覆盖 `/system/*` 管理接口及 `/user/profile` 个人信息接口，切换前端到 Node 后端时，对应管理功能暂不可用，需后续补齐。

## 2025-11-20（Codex）backend-php 初始迁移与测试记录

- 当前状态：
  - 已在 `backend-php` 目录下搭建基于 Slim 的 PHP 后端骨架，并通过 Composer 配置 `Voc\Admin\` 命名空间。
  - 已实现与 Go/Java 一致的认证与用户会话相关接口：
    - `POST /auth/login`
    - `GET /auth/user/info`
    - `GET /auth/user/route`
- 测试命令建议（需本地安装 PHP ≥8.1 且配置 PostgreSQL）：
  - 在 `backend-php` 目录执行：`composer install`
  - 启动服务：`php -S 0.0.0.0:4398 -t public public/index.php`
  - 使用前端或 `curl` 校验与 Go/Java 版同一组接口。
- Codex 本地环境限制：
  - 当前容器内未安装 `php` 运行时，无法在此处直接启动或执行接口联调，仅完成静态代码与结构校验。
- 风险评估：
  - 由于缺少真实运行验证，数据库连接参数（`DB_HOST`/`DB_PORT`/`DB_USER`/`DB_PWD`/`DB_NAME`）与 RSA/JWT 配置需在本地环境中自行确认。
  - 目前仅迁移了认证与用户信息/路由相关接口，`/system/*`、`/common/*`、`/user/profile` 等管理与配置接口将在后续补齐，对应前端功能在切换到 PHP 后端时暂时不可用。

---

## 2025-11-20（Codex）backend-rust 初始迁移与构建验证记录

- 当前状态：
  - 在 `backend-rust` 目录下手工初始化 Rust 工程结构（因当前环境未安装 `cargo`，无法使用 `cargo init` 与 `cargo build`）：
    - `Cargo.toml`：选型 `axum` + `tokio` + `sqlx` + `tower-http` + `jsonwebtoken` + `bcrypt` + `rsa`，作为主流 Rust Web 技术栈。
    - `src/config.rs`：复用 Java/Go 的环境变量约定（`DB_HOST`/`DB_PORT`/`DB_USER`/`DB_PWD`/`DB_NAME`、`AUTH_RSA_PRIVATE_KEY`、`AUTH_JWT_SECRET`、`FILE_STORAGE_DIR`、`HTTP_PORT`），并提供同样的默认值。
    - `src/db.rs`：基于 `sqlx::Pool<Postgres>` 创建连接池，连接字符串与 Go 版 `NewPostgres` 一致。
    - `src/security/`：实现 RSA 解密（Base64 + PKCS#8 + PKCS#1 v1.5）、BCrypt 密码校验（兼容 `{bcrypt}` 前缀）、JWT 生成与解析（HS256，载荷含 `userId/iat/exp`），对齐 backend-go 的安全行为。
    - `src/interfaces/http/response.rs`：实现与前端 `ApiRes<T>` 完全一致的响应包装结构（`code/data/msg/success/timestamp`），并提供 `api_ok/api_fail` 帮助函数。
    - `src/interfaces/http/auth_handler.rs`：实现 `POST /auth/login`，复用 Java/Go 的登录流程（RSA 解密 → BCrypt 校验 → 状态校验 → JWT 生成）。
    - `src/interfaces/http/user_handler.rs`：实现 `GET /auth/user/info` 与 `GET /auth/user/route`，SQL 查询与返回结构参考 backend-go 的 `UserHandler` 与 RBAC 仓储逻辑。
    - `src/interfaces/http/common_handler.rs`：实现 `/common/dict/option/site`、`/common/tree/menu`、`/common/tree/dept`、`/common/dict/user`、`/common/dict/role`、`/common/dict/:code` 等接口，SQL 与 Go 版保持一致。
    - `src/interfaces/http/captcha_handler.rs`：实现 `GET /captcha/image`，返回禁用验证码配置（`isEnabled=false`），与其他多语言后端保持一致。
    - `src/main.rs`：组装 `AppState`（DB 连接池 + 安全组件），构建 `axum::Router`，注册所有上述路由，并通过 `tower-http::ServeDir` 映射静态 `/file` 目录。
- 预期测试命令（需本地安装 Rust toolchain 与 PostgreSQL，并初始化 Java/Go 版相同的 `sys_*` 表）：
  - 在 `backend-rust` 目录执行：
    - `cargo build`
    - `RUST_LOG=info HTTP_PORT=4398 DB_HOST=... DB_PORT=... DB_USER=... DB_PWD=... DB_NAME=... AUTH_RSA_PRIVATE_KEY=... AUTH_JWT_SECRET=... FILE_STORAGE_DIR=./data/file cargo run`
  - 使用前端或 `curl` 验证以下接口：
    - `GET /captcha/image`
    - `POST /auth/login`
    - `GET /auth/user/info`
    - `GET /auth/user/route`
    - `/common/*`：`/common/dict/option/site`、`/common/tree/menu`、`/common/tree/dept`、`/common/dict/user`、`/common/dict/role`、`/common/dict/{code}`
- Codex 本地环境限制：
  - 当前容器内未成功安装 `cargo`（尝试通过 `rustup` 安装因超时失败），无法在本环境内执行 `cargo build` 与 `cargo run`，仅完成静态代码与结构层面的校验。
- 风险评估：
  - 由于缺少实际编译与运行验证，可能存在少量语法或依赖版本问题，需要在本地 Rust 环境中执行 `cargo build` 进行修正。
  - 数据库访问依赖与 Go/Java 相同的 `sys_*` 表结构，如 DB 初始化不一致会导致查询错误。
  - 当前 Rust 版与 Python/Node/PHP 一致，仅迁移认证、用户信息/路由以及 `/common/*` 基础接口，尚未覆盖 `/system/*` 管理接口及 `/user/profile` 个人中心接口，对应前端管理功能在切换到 Rust 后端时暂时不可用，后续可按 backend-go 的 handler 逐模块补齐。

## 2025-11-20（Codex）backend-php /system 核心模块迁移记录

- 本次新增 PHP 实现的接口：
  - 用户管理：
    - `GET /system/user`（分页查询）
    - `GET /system/user/list`
    - `GET /system/user/{id}`
    - `POST /system/user`
    - `PUT /system/user/{id}`
    - `DELETE /system/user`
    - `PATCH /system/user/{id}/password`
    - `PATCH /system/user/{id}/role`
    - `GET /system/user/export`
    - `GET /system/user/import/template`
    - `POST /system/user/import/parse`
    - `POST /system/user/import`
  - 角色管理：
    - `GET /system/role/list`
    - `GET /system/role/{id}`
    - `POST /system/role`
    - `PUT /system/role/{id}`
    - `DELETE /system/role`
    - `PUT /system/role/{id}/permission`
    - `GET /system/role/{id}/user`
    - `POST /system/role/{id}/user`
    - `DELETE /system/role/user`
    - `GET /system/role/{id}/user/id`
  - 菜单管理：
    - `GET /system/menu/tree`
    - `GET /system/menu/{id}`
    - `POST /system/menu`
    - `PUT /system/menu/{id}`
    - `DELETE /system/menu`
    - `DELETE /system/menu/cache`
  - 部门管理：
    - `GET /system/dept/tree`
    - `GET /system/dept/{id}`
    - `POST /system/dept`
    - `PUT /system/dept/{id}`
    - `DELETE /system/dept`
    - `GET /system/dept/export`
- 行为说明：
  - 入参与字段校验、密码规则（长度 8-32、需含字母和数字）、系统内置角色/用户/部门保护逻辑，与 backend-go 中对应 handler 尽量保持一致。
  - ID 生成策略使用微秒时间戳 + 随机数，避免与现有数据冲突；在已有库数据下建议优先用于新增，而不会影响现有 ID。
- 测试建议（本地 PHP 环境）：
  - 启动 `backend-php`：`composer install && php -S 0.0.0.0:4398 -t public public/index.php`。
  - 使用现有前端管理端分别切换到 PHP 后端，逐项验证 `/system/user`、`/system/role`、`/system/menu`、`/system/dept` 页面功能是否与 Java/Go 行为一致。
- 由于 Codex 环境缺少 PHP 运行时，本次仅完成静态代码与结构层迁移，未能实际连库与前端联调，需在本地完成最终验证。

---

### 2025-11-20（Codex）backend-go 系统操作日志中间件接入测试记录

- 测试命令：
  - 在 `backend-go` 目录执行：`go test ./...`
- 测试结果：
  - 新增 `internal/domain/syslog`、`internal/infrastructure/persistence/syslog/postgres_repository.go` 与 `internal/interfaces/http/log_middleware.go` 之后，`go test ./...` 仍然全部通过（当前各包无测试用例，仅做编译校验）。
- 风险评估：
  - 由于未在本环境中实际启动 Gin 服务并对 `/auth/*`、`/system/*`、`/monitor/*` 接口进行 HTTP 访问，尚未验证中间件在真实请求流水线中的表现。
  - 日志中间件当前同步写入数据库，在高并发场景下可能增加少量延迟，建议后续按需改造为异步落库并在本地压测环境中评估性能影响。
