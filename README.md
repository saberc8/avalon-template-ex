# Avalon Admin Monorepo

> AI代码占比 100%，全程使用codex以及copilot完成

> 更新日期：2025-11-26（执行者：Codex）

多端一体的后台管理系统示例仓库，包含多种后端实现（Java / Go / Node / Python）与多端前端（PC 管理端、H5、App 等），便于对比学习和本地联调。

如果只想快速跑通一个组合，推荐：`backend-go` + `pc-admin-vue3`。更详细的启动说明可以参考根目录的 `README-startup.md`。

---

## 1. 目录概览

- `backend-python`：FastAPI 实现的后端服务（推荐起步）
- `backend-go`：Go + Gin 实现的后端服务
- `backend-node`：NestJS + Prisma 实现的后端服务
- `backend-java`：原版 ContiNew Admin Java 后端
- `pc-admin-vue3`：Vue3 管理后台前端（Arco Design）
- `pc-admin-nextjs`：Next.js 管理后台前端
- `h5-nextjs`：H5 端 Next.js 应用（开发中，当前暂无业务代码）
- `app-uniapp`：UniApp 应用（开发中，当前暂无业务代码）
- `app-flutter`：Flutter 应用（开发中，当前暂无业务代码）

---

## 2. 通用准备

- Node.js ≥ 18（建议 18/20），推荐包管理器：`pnpm`
- Python ≥ 3.10
- Go ≥ 1.22
- JDK 17 + Maven 3.8+
- 数据库：PostgreSQL（Python / Go / Node 主要使用），Java 原版以 MySQL 为主

统一建议的数据库配置（以 PostgreSQL 为例）：

- `DB_HOST=127.0.0.1`
- `DB_PORT=5432`
- `DB_USER=postgres`
- `DB_PWD=123456`
- `DB_NAME=nv_admin`

认证相关环境变量（多个后端可复用）：

- `AUTH_RSA_PRIVATE_KEY`：登录密码 RSA 私钥（Base64 PKCS#8，用于解密前端加密密码）
- `AUTH_JWT_SECRET`：JWT 签名密钥

---

## 3. 各后端运行方式

### 3.1 Python 后端：`backend-python`

默认端口：`4398`。

```bash
cd backend-python

# （可选）创建虚拟环境
python3 -m venv .venv
source .venv/bin/activate    # Windows: .venv\Scripts\activate

# 安装依赖
pip install -r requirements.txt

# 启动开发服务
uvicorn app.main:app --reload --port 4398
```

确保 PostgreSQL 与环境变量（`DB_*`、`AUTH_*`）配置正确后，可通过 `http://localhost:4398` 访问接口。

---

### 3.2 Go 后端：`backend-go`

默认端口：`HTTP_PORT=4398`（未设置时）。

```bash
cd backend-go

# 直接运行开发环境
go run ./cmd/admin
```

首次启动会自动迁移数据库并初始化管理员账号等数据，数据库使用上文统一的 `DB_*` 环境变量。

---

### 3.3 Node 后端：`backend-node`

基于 NestJS + Prisma，默认端口与其他后端保持一致（通常为 `4398`，可在代码中修改）。

```bash
cd backend-node

# 安装依赖（推荐使用 pnpm）
pnpm install           # 或 npm install / yarn

# 生成 Prisma Client
pnpm prisma:generate

# 开发模式启动
pnpm start:dev

# 或构建后以生产模式启动
pnpm build
pnpm start
```

请通过 `DATABASE_URL` 或 `DB_*` 环境变量配置数据库连接，保持与其他后端共用同一库，方便切换。

---

### 3.4 Java 后端：`backend-java`

基于原版 ContiNew Admin，多模块 Maven 项目，推荐使用 MySQL + Redis。

```bash
cd backend-java

# 修改 continew-webapi 的 application-dev.yml，配置数据库和 Redis
# 然后启动 API 模块
mvn clean spring-boot:run -pl continew-webapi
```

开发环境默认 HTTP 端口为 `4398`，前端默认地址为 `http://localhost:4399`（见 `continew-webapi/src/main/resources/config/application-dev.yml` 中的 `project.url`）。

---

## 4. 前端运行方式

### 4.1 Vue3 管理端：`pc-admin-vue3`

默认开发端口：`4399`，通过 Vite 代理访问后端。

```bash
cd pc-admin-vue3

pnpm install
pnpm dev
```

开发环境下可在 `pc-admin-vue3/.env.development` 中配置后端地址，例如：

```env
VITE_API_BASE_URL=http://localhost:4398
```

启动后访问：`http://localhost:4399`。

---

### 4.2 Next.js 管理端：`pc-admin-nextjs`

默认开发端口：`3000`。

```bash
cd pc-admin-nextjs

pnpm install
pnpm dev
```

如需指定后端地址，可通过环境变量覆盖，例如：

```bash
NEXT_PUBLIC_API_BASE_URL=http://localhost:4398 pnpm dev
```

启动后访问：`http://localhost:3000`。

---

### 4.3 H5 端：`h5-nextjs`

`h5-nextjs` 目录目前仅创建了基础结构，业务代码尚未补充，处于**开发中**状态。

暂时可以将其视为预留的 H5 Next.js 实现占位，不需要运行。

---

### 4.4 其他客户端：`app-uniapp` / `app-flutter`

这两个目录当前仅为占位，尚未接入具体代码与脚本，同样处于**开发中**状态。

后续接入时可参照各自技术栈的标准项目结构（如 UniApp CLI、Flutter 工程）进行初始化，并复用相同的后端接口规范。

---

## 5. 常用组合示例

**组合 A：Python 后端 + Vue3 管理端**

```bash
# 终端 1：后端
cd backend-python
uvicorn app.main:app --reload --port 4398

# 终端 2：前端
cd pc-admin-vue3
pnpm dev
```

浏览器访问：`http://localhost:4399`。

**组合 B：Go 后端 / Node 后端 + Vue3 或 Next 管理端**

只需将前端的后端地址改为对应端口（如 `http://localhost:4398`），再启动对应后端和前端，即可完成切换。

---

## 6. 登录账号

默认管理员账号沿用 ContiNew Admin 的初始化数据（在全新数据库环境下）：

- 账号：`admin`
- 密码：`admin123`

如使用不同后端实现（Python / Go / Node / Java），只要数据库初始化脚本保持一致，即可通用该账号密码。
