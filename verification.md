## 2025-11-20（Codex）功能验证说明

### 已验证内容
- Go 项目编译与基础单元测试入口：
  - 在 `backend-go` 目录执行 `go test ./...`，所有包均成功编译，当前无实际测试用例。
- 静态代码级联通性检查：
  - `cmd/admin/main.go` 中已注册：
    - `/system/storage*` 路由（StorageHandler）
    - `/system/client*` 路由（ClientHandler）
    - `/common/dict/option/site` 仍由 CommonHandler 提供，但实现改为基于 `sys_option`。
  - 迁移函数 `AutoMigrate` 已按顺序调用：
    - `ensureSysOption` → `ensureSysStorage` → `ensureSysClient`，保证配置表在服务启动时自动创建与补种默认数据。

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

### 风险评估
- 现阶段改动主要集中在配置与 CRUD 层，未引入复杂业务逻辑或跨模块副作用，风险可控。
- 建议在接入正式环境前，与前端一起完成一次完整的系统配置回归（五个配置页逐一操作、刷新与重登录验证）。+
