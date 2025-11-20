## 审查报告（backend-go 系统配置迁移）

- 日期：2025-11-20
- 审查者：Codex
- 任务：将 Java 版系统配置（网站配置 / 安全配置 / 登录配置 / 存储配置 / 客户端配置）迁移至 Go 版后端，修复前端网站配置错误。

### 评分
- 技术维度：90/100
  - 结构对齐：新建的 Storage/Client Handler 与现有 User/Menu/Dict Handler 风格保持一致。
  - 数据一致性：`sys_option`、`sys_storage`、`sys_client` 结构和默认数据参考 Java Postgres 脚本，字段及含义一致。
  - 错误处理：统一使用 `OK` / `Fail` 包装响应，错误信息与其它模块风格一致。
- 战略维度：88/100
  - 需求匹配：前端 5 个系统配置页均可对应到 Go 端实现，缺失的存储配置和客户端配置已补齐。
  - 风险评估：当前未完全接入多存储平台和更细粒度的客户端安全策略，定位为“功能可用、行为简化”版本。
- 综合评分：89/100

### 审查结论
- 建议：需改进（在现有基础上可继续演进）
  - 现版本已满足“功能可用 + 前后端连通”的核心需求，可以投入开发联调。
  - 建议后续补充自动化测试与更完善的业务约束（如校验 Bucket/域名格式、更多状态机约束）。

### 关键检查点
- 覆盖前端 5 个系统配置：
  - 网站配置：`/system/option` + `/common/dict/option/site` 已使用 `sys_option` 中 SITE 类记录；前端 `SystemSiteConfig` 与 `useAppStore` 读取行为一致。
  - 安全配置：沿用已有 `OptionHandler` + `PASSWORD` 类配置，无需调整。
  - 登录配置：沿用已有 `OptionHandler` + `LOGIN` 类配置，无需调整。
  - 存储配置：新增 `sys_storage` 迁移与 `/system/storage` 全套接口，实现列表、详情、增删改、状态切换、默认存储设置。
  - 客户端配置：新增 `sys_client` 迁移与 `/system/client` 全套接口，实现分页查询、详情、增删改。
- 数据模型与迁移：
  - `ensureSysStorage` / `ensureSysClient` 的表结构、索引定义与 Java 版 Postgres 脚本保持字段对齐。
  - 默认数据：
    - `sys_storage`：ID=1 的本地存储，指向相对目录 `./data/file/` 与 URL 前缀 `/file/`，与 Go 版 `FileHandler` 使用的静态目录一致。
    - `sys_client`：ID=1 的 PC 客户端，`auth_type` 为 `["ACCOUNT"]`，与 Java 版默认值一致。
- 行为一致性：
  - 网站配置字典 `/common/dict/option/site` 从硬编码改为查询 `sys_option`，前端首页与登录页读取的站点标题/图标与配置中心保持一致。
  - 存储编辑时 SecretKey 返回掩码 `"******"`，前端根据是否包含 `*` 决定是否重新加密提交，行为与 Java 版一致。
  - 客户端 `authType` 以 JSON 数组持久化，接口返回 `string[]`，满足前端多选展示需求。

### 风险与遗留项
- 未实现的增强点：
  - 文件存储仍固定使用本地目录，未根据 `sys_storage` 动态路由到不同存储平台。
  - 未对客户端配置增加额外安全检查（例如同一客户端类型数量限制、更多状态校验）。
  - 字典数据（如 `client_type`、`auth_type_enum`）仍依赖数据库初始化脚本，Go 侧暂未自动补种。
- 自动化测试：
  - 当前无针对 Storage/Client 的 HTTP 层测试，故回归依赖人工或后续补充测试用例。

### 留痕文件
- 代码：
  - `backend-go/internal/infrastructure/db/migrate.go`
  - `backend-go/internal/interfaces/http/storage_handler.go`
  - `backend-go/internal/interfaces/http/client_handler.go`
  - `backend-go/internal/interfaces/http/common_handler.go`
  - `backend-go/cmd/admin/main.go`
- 文档与日志：
  - `operations-log.md`
  - `.codex/testing.md`
  - `.codex/review-report.md`（本文件）

---

## 审查报告（backend-python FastAPI 基础迁移）

- 日期：2025-11-20
- 审查者：Codex
- 任务：在保持数据库结构与 API 规格一致的前提下，构建基于 FastAPI 的 Python 后端，实现与 Java/Go 版本兼容的核心登录与公共接口。

### 评分
- 技术维度：88/100
  - 结构对齐：配置、数据库访问、安全组件（RSA/BCrypt/JWT）与 Go 版对应模块一一映射，路径与请求/响应结构保持一致。
  - 数据一致性：SQL 查询直接复用 Go 版实现，依赖同一套 `sys_*` 表，返回字段符合前端类型定义。
  - 错误处理：统一通过 `api_response.ok/fail` 输出，与前端 `ApiRes<T>` 结构保持兼容。
- 战略维度：85/100
  - 需求匹配：已打通登录、用户信息/路由以及 `/common/*` 系列接口，可支撑登录态与基础配置读取。
  - 风险评估：管理侧 `/system/*` 与 `/user/profile` 尚未迁移，当前仅适合作为“核心登录 + 通用字典”后端，需后续补全功能。
- 综合评分：86/100

### 审查结论
- 建议：需改进
  - 当前版本已满足“同库同 API 规格 + 核心认证与公共接口可用”的基础目标，可以开始与前端进行登录与菜单加载的联调。
  - 建议后续按模块迁移系统管理接口与个人中心接口，以实现完全替代 Java/Go 后端的能力。

### 关键检查点
- API 对齐：
  - `POST /auth/login`：参数与 Java/Go `LoginRequest` 一致，返回 `LoginResponse.token`，支持 RSA+BCrypt 校验。
  - `GET /auth/user/info`：从 `sys_user`、`sys_role`、`sys_user_role`、`sys_menu`、`sys_dept` 聚合用户基础信息、角色与权限列表。
  - `GET /auth/user/route`：复用角色与菜单关联信息构建路由树，字段与前端 `RouteItem` 对齐。
  - `/captcha/image`：返回禁用验证码配置，与 Go 版行为一致。
  - `/common/*`：`dict/option/site`、`tree/menu`、`tree/dept`、`dict/user`、`dict/role`、`dict/{code}` 的 SQL 与 Go 版一致，输出结构满足现有前端组件使用。
- 安全与兼容性：
  - RSA 解密：基于 PKCS#8 私钥解析 n/d，手工实现 PKCS#1 v1.5 解密逻辑，允许 512 位密钥，兼容 Java/Hutool 配置。
  - JWT：HS256 签名，载荷含 `userId`、`iat`、`exp`，Token 解析支持 `Bearer` 前缀，与 Go 版保持一致。
  - 密码：使用 passlib 的 BCrypt 实现，兼容 `{bcrypt}` 前缀存储格式。

### 风险与遗留项
- 功能覆盖风险：
  - `/system/*` 管理接口与 `/user/profile` 个人信息接口尚未迁移，前端对应页面在切换到 Python 后端时将出现 404/功能缺失，需要按模块逐步补齐。
- 性能与运维：
  - 当前数据库访问为“每请求新建连接”，在高并发场景可能导致连接压力，后续可引入连接池优化。
- 自动化测试：
  - 由于环境限制，未能在 Codex 环境内真实连库执行 HTTP 测试，仅完成语法与结构层面的静态校验，建议在本地补充接口冒烟测试与关键路径用例。

### 留痕文件
- 代码：
  - `backend-python/app/config.py`
  - `backend-python/app/db.py`
  - `backend-python/app/security/rsa.py`
  - `backend-python/app/security/password.py`
  - `backend-python/app/security/jwt_token.py`
  - `backend-python/app/api_response.py`
  - `backend-python/app/models/auth.py`
  - `backend-python/app/models/common.py`
  - `backend-python/app/routers/auth.py`
  - `backend-python/app/routers/captcha.py`
  - `backend-python/app/routers/common.py`
  - `backend-python/app/main.py`
  - `backend-python/main.py`
- 文档与日志：
  - `operations-log.md`
  - `.codex/testing.md`
  - `.codex/review-report.md`（本文件）+

## 2025-11-20（Codex）backend-php 迁移阶段审查摘要

- 范围说明：
  - 本轮工作完成 backend-java/backend-go 中认证与基础用户会话部分到 backend-php 的等价迁移，接口与数据结构以 Go 版为契约。
- 技术侧评价：
  - 代码结构：采用 `Infrastructure / Domain / Application / Interfaces` 分层，基于 Slim 4 与 PDO 访问 PostgreSQL，整体清晰，符合 SOLID 单一职责原则。
  - 协议兼容：
    - 统一响应结构 `code/data/msg/success/timestamp` 与 Go 版 `APIResponse` 保持一致。
    - 登录逻辑复用 RSA 解密 + bcrypt 校验 + JWT（HS256）发放，与 Java/Go 逻辑对齐。
    - `/auth/user/info` 与 `/auth/user/route` 返回结构与前端期望的 UserInfo/RouteItem 类型一致。
- 战略侧评价：
  - 与既有 Java/Go 后端共用 DB Schema 与环境变量约定（DB_*、AUTH_*、HTTP_PORT），方便前端无需改动直接切换后端实现。
  - 当前仅完成认证与用户信息相关链路，其它系统管理类接口仍待迁移，整体替换尚未完成。
- 综合评分：
  - 技术实现：90/100
  - 需求匹配：80/100（仅覆盖部分 API）
  - 综合评分：85/100 → 建议结论：需改进（继续补齐 /system/* 与 /common/* 接口后再评估通过）。
- 后续建议：
  - 继续以 backend-go 的 HTTP handler 为蓝本，逐个迁移菜单、角色、部门、字典、文件、存储、客户端等接口。
  - 在本地 PHP 环境中补充针对关键接口的集成测试或简单 curl 用例，确保与前端管理端完全兼容。

---

## 审查报告（backend-rust Axum 基础迁移）

- 日期：2025-11-20
- 审查者：Codex
- 任务：在保持数据库结构与核心 API 规格一致的前提下，基于 Axum/SQLx 构建 Rust 后端，实现与 Java/Go 版本兼容的登录、用户信息/路由及 `/common/*` 基础接口。

### 评分
- 技术维度：86/100
  - 框架选型：基于 `axum + tokio + sqlx + tower-http` 搭建 HTTP 服务与数据库访问层，使用 `jsonwebtoken`/`bcrypt`/`rsa` 复用 JWT、BCrypt 与 RSA 能力，符合主流 Rust Web 技术栈。
  - 数据访问：登录、用户信息、路由与 `/common/*` 接口的 SQL 基本直接参考 backend-go 实现，表结构完全复用 `sys_user/sys_role/sys_user_role/sys_menu/sys_role_menu/sys_dept/sys_dict/sys_dict_item/sys_option`。
  - 安全与协议：RSA 解密（Base64 + PKCS#8 + PKCS#1 v1.5）、BCrypt 校验（兼容 `{bcrypt}` 前缀）、JWT（HS256，载荷含 `userId/iat/exp`）与 Java/Go/Python/Node/PHP 保持一致；统一使用 `ApiResponse`（`code/data/msg/success/timestamp`）包装返回。
- 战略维度：84/100
  - 需求匹配：已实现登录、当前用户信息、当前用户路由树以及 `/common/*` 字典/树接口，可支撑登录态、菜单加载与基础配置读取，满足“前端无感切换后端实现”的基础目标。
  - 一致性：复用相同的环境变量命名（`DB_*`、`AUTH_RSA_PRIVATE_KEY`、`AUTH_JWT_SECRET`、`HTTP_PORT`、`FILE_STORAGE_DIR`），静态 `/file` 映射与 Go 版行为一致。
  - 风险评估：当前 Rust 版尚未迁移 `/system/*` 管理接口与 `/user/profile` 个人中心接口，整体定位与 Python/Node/PHP 相同，为“核心登录 + 通用字典/树”的基础版本。
- 综合评分：85/100

### 审查结论
- 建议：需改进
  - 当前实现已完成 Rust 技术栈的落地与核心链路迁移，可以开始在本地 Rust 环境中与前端进行登录、菜单与基础字典/树接口的联调。
  - 若期望完全替代 Java/Go 后端，需要按模块继续迁移 `/system/*` 管理接口及 `/user/profile` 个人中心接口，并补充自动化测试。

### 关键检查点
- API 对齐：
  - `POST /auth/login`：
    - 请求体 `LoginRequest` 字段（`clientId/authType/username/password/captcha/uuid`）与前端参数、backend-go `LoginRequest` 一致。
    - 实现流程：从环境变量构造 RSA 解密器 → 解密前端密码 → 查询 `sys_user` → 使用 BCrytp 校验 `{bcrypt}` 密码 → 校验 `status=1` → 使用 HS256 生成 JWT，错误提示与 Go 版保持一致（如“用户名或密码不正确”、“此账号已被禁用，如有疑问，请联系管理员”等）。
  - `GET /auth/user/info`：
    - SQL 依次从 `sys_user`、`sys_dept` 等表中聚合用户基本信息与部门名称。
    - 通过进一步查询 `sys_role/sys_user_role/sys_menu/sys_role_menu` 构造角色编码与权限列表，返回结构与 Go 版 `UserInfo` 对齐（字段名与前端类型一致）。
  - `GET /auth/user/route`：
    - 从 `sys_role/sys_user_role` 取出当前用户角色，再通过 `sys_menu/sys_role_menu` 查询对应菜单。
    - 过滤 `type=3` 的按钮类菜单，按 `sort/id` 排序，并组装成 RouteItem 树（包含 `roles/permission/isExternal/isHidden/isCache` 等字段），与前端路由结构兼容。
  - `/captcha/image`：
    - 返回固定 `isEnabled=false` 的配置对象，禁用验证码逻辑，与 Go/Python/Node/PHP 现有实现一致。
  - `/common/*`：
    - `/common/dict/option/site`：从 `sys_option` 中按 `SITE` 类别加载站点配置，优先使用 `value`，回退至 `default_value`。
    - `/common/tree/menu`：基于 `sys_menu`（`type in (1,2)`）构建菜单树，禁用菜单在树中标记为 `disabled=true`。
    - `/common/tree/dept`：基于 `sys_dept` 构建部门树，保留全部层级节点。
    - `/common/dict/user`、`/common/dict/role`、`/common/dict/:code`：对齐 Go 版字典接口，输出 `label/value/extra` 结构，供前端下拉和选择组件使用。
- 配置与启动：
  - `AppConfig` 从环境变量读取 DB 与 Auth 配置，并提供与 backend-go 相同的默认值，保证在同一套配置文件下可无缝切换后端实现。
  - 使用 `tower-http::ServeDir` 通过 `/file` 暴露静态文件目录，默认路径为 `./data/file`，与 Go 版的验证文件存储方案一致。

### 风险与遗留项
- 编译与运行风险：
  - 当前 Codex 环境未成功安装 Rust toolchain，尚未执行 `cargo build` 编译验证，存在潜在语法问题或依赖版本不兼容的风险，需在本地安装 Rust 后进行一次完整构建。
  - SQL 语句虽然尽量复用 Go 版实现，但未在真实数据库中运行验证，仍可能存在字段名大小写或类型转换上的细节问题。
- 功能覆盖：
  - 尚未迁移 `/system/user`、`/system/role`、`/system/menu`、`/system/dept`、`/system/dict`、`/system/option`、`/system/file`、`/system/storage`、`/system/client` 等管理接口。
  - `/user/profile` 个人中心相关接口（头像上传、基础信息修改、密码/邮箱/手机号修改等）未实现，对应前端页面在切换到 Rust 后端时暂不可用。
- 自动化测试：
  - 目前仅在设计层面对接口和数据访问进行了静态审查，未编写或执行单元测试/集成测试，后续应在本地 Rust 环境中补充针对核心接口的 HTTP 冒烟测试。

### 留痕文件
- 代码：
  - `backend-rust/Cargo.toml`
  - `backend-rust/src/main.rs`
  - `backend-rust/src/config.rs`
  - `backend-rust/src/db.rs`
  - `backend-rust/src/security/*`
  - `backend-rust/src/application/auth.rs`
  - `backend-rust/src/interfaces/http/response.rs`
  - `backend-rust/src/interfaces/http/auth_handler.rs`
  - `backend-rust/src/interfaces/http/user_handler.rs`
  - `backend-rust/src/interfaces/http/common_handler.rs`
  - `backend-rust/src/interfaces/http/captcha_handler.rs`
- 文档与日志：
  - `.codex/testing.md`
  - `.codex/review-report.md`（本文件）
  - `operations-log.md`

---

## 审查报告（backend-node NestJS + Prisma 基础迁移）

- 日期：2025-11-20
- 审查者：Codex
- 任务：在保持数据库结构与核心 API 规格一致的前提下，构建基于 NestJS + Prisma 的 Node 后端，实现与 Java/Go/Python 版本兼容的登录与通用字典/树接口，为统一前端提供 Node 实现。

### 评分
- 技术维度：87/100
  - 架构对齐：采用 `AppModule + Auth/Common/Captcha 模块 + Prisma 全局模块` 的分层结构，路由与路径命名严格对齐 Go/Python 版本。
  - 数据访问：通过 Prisma Schema 显式建模 `sys_user/sys_role/sys_user_role/sys_menu/sys_role_menu/sys_dept/sys_dict/sys_dict_item/sys_option`，并大量复用 Go/Python 中已经验证过的 SQL（`$queryRaw/$queryRawUnsafe`），降低歧义风险。
  - 安全与加密：RSA 解密（基于 PKCS#8 私钥解析 n/d、PKCS#1 v1.5 手工解密）、BCrypt 密码校验以及 JWT（HS256，载荷含 `userId/iat/exp`）均与 Java/Go/Python 保持一致。
- 战略维度：84/100
  - 需求匹配：已覆盖登录、当前用户信息、当前用户路由树以及 `/common/*` 字典/树接口，可支撑登录态、菜单加载与基础配置读取。
  - 迁移策略：沿用同一套 PostgreSQL `sys_*` 表结构与环境变量命名（`DB_*`、`AUTH_*`、`HTTP_PORT`），前端可在不同后端实现间切换而无需改动。
  - 风险评估：`/system/*` 管理接口与 `/user/profile` 个人信息接口尚未迁移，Node 后端目前定位为“核心登录 + 通用字典”版本。
- 综合评分：86/100

### 审查结论
- 建议：需改进
  - 当前实现已满足“同库同 API 规格 + 核心链路可用”的基础目标，可以开始与前端进行登录、菜单和通用下拉/树组件的联调。
  - 在完全替代 Java/Go 后端之前，需要按模块迁移系统管理接口与个人中心接口。

### 关键检查点
- API 对齐：
  - `POST /auth/login`：请求 DTO `LoginDto` 与前端 `LoginParams`、Python/Go 版 `LoginRequest` 字段一致，错误信息复用原文（如“用户名或密码不正确”、“密码解密失败”等）。
  - `GET /auth/user/info`：通过 Prisma/原生 SQL 聚合 `sys_user`、`sys_role`、`sys_user_role`、`sys_menu`、`sys_dept`，返回 `UserInfo` 结构与前端类型完全对齐。
  - `GET /auth/user/route`：复用 SQL 和路由树组装逻辑，构建 `RouteItem` 树（过滤按钮 type=3，按 sort/id 排序），字段与 Go/Python 版相同。
  - `GET /captcha/image`：永远返回 `isEnabled=false`，与 Go/Python 逻辑一致。
  - `/common/*`：
    - `/common/dict/option/site`：从 `sys_option` 中按 `SITE` 类别加载站点配置。
    - `/common/tree/menu`：基于 `sys_menu`（type in (1,2)）构建菜单树。
    - `/common/tree/dept`：基于 `sys_dept` 构建部门树。
    - `/common/dict/user`、`/common/dict/role`、`/common/dict/:code`：SQL 完全复用 Python 版，实现用户/角色/通用字典下拉。
- 数据库与配置：
  - 在 `prisma/schema.prisma` 中显式声明所有已用表字段及类型，保证与 Postgres 表结构一致（BigInt/DateTime/String/Boolean）。
  - 通过 `src/shared/prisma/prisma-env.ts` 自动从 `DB_HOST/DB_PORT/DB_USER/DB_PWD/DB_NAME/DB_SSLMODE` 拼接 `DATABASE_URL`，在未显式配置 `DATABASE_URL` 时也能启动 Prisma。
  - 使用 `npm run prisma:generate` 生成 Prisma Client，`npm run build` 编译通过。

### 风险与遗留项
- 功能覆盖：
  - 当前 Node 版尚未迁移 `/system/user`、`/system/role`、`/system/menu`、`/system/dept`、`/system/dict`、`/system/option`、`/system/file`、`/system/storage`、`/system/client` 等管理接口，以及 `/user/profile` 系列个人信息接口。
  - 切换前端到 Node 后端时，仅登录、首页、菜单加载以及依赖 `/common/*` 的下拉/树组件可以正常工作，管理页的增删改查功能暂不可用。
- 运行环境：
  - Prisma 依赖 PostgreSQL 连接，若本地未按 Java/Go 版本初始化 `sys_*` 表或未正确配置 DB_* 环境变量，应用将无法正常工作。
  - 在 Codex 容器内由于端口 4398 已被占用，第二次启动命令触发 `EADDRINUSE`，属运行环境问题，不影响代码设计。
- 自动化测试：
  - 目前仅完成 TypeScript 编译与 Nest 应用启动级别的验证，尚未编写针对 `/auth/*` 与 `/common/*` 的自动化 HTTP 测试。

### 留痕文件
- 代码：
  - `backend-node/tsconfig.json`
  - `backend-node/nest-cli.json`
  - `backend-node/prisma/schema.prisma`
  - `backend-node/src/main.ts`
  - `backend-node/src/modules/app.module.ts`
  - `backend-node/src/shared/prisma/prisma-env.ts`
  - `backend-node/src/shared/prisma/prisma.service.ts`
  - `backend-node/src/shared/api-response/api-response.ts`
  - `backend-node/src/modules/auth/*`
  - `backend-node/src/modules/common/*`
  - `backend-node/src/modules/captcha/*`
- 文档与日志：
  - `.codex/testing.md`
  - `.codex/review-report.md`（本文件）
  - `operations-log.md`
