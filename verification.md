## 2025-11-20（Codex）功能验证说明

### 已验证内容
- Go 项目编译与基础单元测试入口：
  - 在 `backend-go` 目录执行 `go test ./...`，所有包均成功编译，当前无实际测试用例。
- 静态代码级联通性检查（系统配置）：
  - `cmd/admin/main.go` 中已注册：
    - `/system/storage*` 路由（StorageHandler）
    - `/system/client*` 路由（ClientHandler）
    - `/common/dict/option/site` 仍由 CommonHandler 提供，但实现改为基于 `sys_option`。
  - 迁移函数 `AutoMigrate` 已按顺序调用：
    - `ensureSysOption` → `ensureSysStorage` → `ensureSysClient`，保证配置表在服务启动时自动创建与补种默认数据。
- 新增静态检查（系统监控 / 在线用户 / 系统日志）：
  - `cmd/admin/main.go` 中已注册：
    - `/monitor/online`、`/monitor/online/{token}` 路由（OnlineUserHandler）。
    - `/system/log`、`/system/log/{id}`、`/system/log/export/login`、`/system/log/export/operation` 路由（LogHandler）。
  - `AutoMigrate` 中 `ensureSysMenu` 已补充系统监控相关菜单种子数据（ID：2000、2010、2011、2012、2030、2031、2032、2033），确保前端菜单加载与权限控制可用。
  - 前端 `pc-admin-vue3` 中监控相关类型定义与后端响应结构已对齐（特别是 `OnlineUserResp` 字段）。

### 未能在本地直接验证的项
- 实际 HTTP 调用与前端联调：
  - 本次在 Codex 容器内仅完成代码实现与编译校验，未启动 Gin 服务与前端进行真实联调。
  - 风险点：
    - 某些查询条件（如客户端 `authType` 过滤）仅做了基础实现，可能与前端期望存在细节差异，但不会影响基础 CRUD。
    - 数据库中若已有历史数据，默认种子插入受 `WHERE NOT EXISTS` 限制，一般不会产生冲突，但建议在非空数据库上多做一次人工确认。

### 建议的人工验证步骤
1. 启动 Go 后端（确保连接到目标 Postgres）：
   - 在 `backend-go` 目录执行：`go run ./cmd/admin`
2. 使用 Postman 或 cURL 依次验证接口：
   - 网站配置：
     - `GET /common/dict/option/site`：确认返回包含 `SITE_TITLE` / `SITE_FAVICON` 等键，且与 `sys_option` 表内容一致。
   - 存储配置：
     - `GET /system/storage/list`：应至少返回一条 ID=1 的“开发环境”记录。
     - 通过前端存储配置页新增/修改本地存储，确认列表展示与状态切换、设为默认逻辑正常。
   - 客户端配置：
     - `GET /system/client?page=1&size=10`：确认返回默认 PC 客户端。
     - 在前端新增客户端，检查多选认证类型展示与详情抽屉内容是否正确。
   - 在线用户：
     - 登录后台后访问“系统监控 → 在线用户”页面，确认列表中出现当前登录用户（昵称 + 用户名 + 登录时间）。
     - 使用昵称和时间范围筛选，观察 `/monitor/online` 请求参数与返回数据是否符合预期。
     - 尝试对非当前会话执行“强退”，确认请求 `DELETE /monitor/online/{token}` 返回成功并刷新列表；对当前 token 强退应提示“不能强退自己”且不会下线当前用户。
  - 系统日志：
    - 访问“系统监控 → 系统日志”页面，切换“登录日志”和“操作日志” Tab，验证 `GET /system/log` 返回的数据字段与表格展示一致。
    - 点击操作日志记录的时间链接，触发 `GET /system/log/{id}`，检查详情抽屉中的 TraceID、请求/响应头体等是否正确展示。
    - 点击“导出”按钮，验证 `GET /system/log/export/login` 与 `/system/log/export/operation` 返回的 CSV 文件可正常下载并打开，列头与 Java 版一致。

### 新增：系统操作日志中间件验证建议
- 功能验证建议：
  - 启动 Go 后端后，使用前端或 Postman 执行以下操作，并在数据库中检查 `sys_log` 表：
    - 执行一次正常登录（`POST /auth/login`），确认插入一条模块为“登录”、描述为“用户登录”的日志，包含请求 URL、IP、浏览器等基础信息。
    - 访问若干管理接口（如 `/system/user`、`/system/role`、`/system/menu` 等），确认 `sys_log` 中对应产生多条“用户管理”“角色管理”“菜单管理”等模块的操作日志。
    - 执行一次失败的业务操作（例如携带无效 token 访问受保护接口），确认日志中 HTTP 状态码为 401/403，状态标记为失败。
  - 结合系统日志页面：
    - 确认新产生的日志记录能通过 `/system/log` 查询到，并在“登录日志”“操作日志” Tab 中正确展示模块、描述、状态与 IP 等字段。
- 风险评估补充：
  - 当前日志中间件不会记录 `OPTIONS` 预检请求，对业务行为无影响，但在调试某些跨域问题时无法通过 `sys_log` 直接看到预检流量，需要结合 Gin 自带日志或 `zap` 等技术日志。

### 风险评估
- 现阶段改动主要集中在配置、监控与 CRUD 层，未引入复杂跨模块副作用，风险可控。
- 在线用户功能目前基于内存存储，服务重启后在线列表会清空，且“强退”不会立即使 JWT 无效，建议在生产环境前视需求补充持久化与黑名单校验。
- 建议在接入正式环境前，与前端一起完成一次完整的系统配置与系统监控回归（配置页与监控页逐一操作、刷新与重登录验证）。
[2025-11-21 16:27:12] 本次任务新增 PHP 路由：DictRoutes、OptionRoutes、CommonRoutes；因环境未安装 php 命令，未能执行语法检查或自动化测试，需在本地 PHP 环境中手动验证。
[2025-11-21 16:53:54] 本轮在 PHP 侧补齐 monitor/log、system/storage、system/client、captcha/image、monitor/online 等接口，实现与 Java/Go 版本一致的路径及返回结构（在线用户列表暂返回空集合）。

## 2025-11-24（Codex）backend-go Swagger 接入验证

- 验证命令：
  - `cd backend-go && go build ./...`
- 验证结果：
  - 编译通过，无错误。
- 变更范围说明：
  - 在 `backend-go/go.mod` 中引入 Swagger 相关依赖（`github.com/swaggo/gin-swagger`、`github.com/swaggo/files`、`github.com/swaggo/swag`）。
  - 在 `backend-go/cmd/admin/main.go` 中配置 Swagger 全局注解（标题、版本、BasePath、Bearer 认证），并注册 `/swagger/*any` 路由。
  - 使用 `swag init -g cmd/admin/main.go -o docs` 生成 `backend-go/docs` 包，作为 Swagger 文档源。
  - 在 `backend-go/internal/interfaces/http/auth_handler.go` 中为登录与登出接口添加 Swagger 注解，作为文档示例入口。
- 待人工/联调验证点：
  - 启动服务后访问 `http://localhost:4398/swagger/index.html`，确认页面能正常打开且接口列表中包含认证模块。
  - 使用 Swagger UI 对 `POST /auth/login`、`POST /auth/logout` 进行调试，确认请求、响应结构与实际业务逻辑一致。
