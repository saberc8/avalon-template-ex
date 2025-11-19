# VOC Admin Monorepo

基于开源项目 **ContiNew Admin / ContiNew Admin UI** 改造的后台管理系统多端一体仓库：

- 后端提供两种实现：原版 Java（`backend`）与 Go 版本（`go-backend`）
- 前端提供 Vue3 管理端（`admin`）与 Next.js 管理端（`next-admin`），共用同一套权限与菜单模型

适合用来对比学习 Java/Go 两种后端实现，以及 Vue3 / React(Next.js) 两种管理端技术栈。

---

## 模块说明

- `admin`：
  - 基于 Vue3 + Vite + Arco Design 的管理后台前端
  - 直接复用/改造自开源项目 **ContiNew Admin UI**

- `backend`：
  - 原版 Java Spring Boot 后端，完整保留自 **ContiNew Admin**，主要作为对照与参考

- `go-backend`：
  - 基于 Go + Gin + PostgreSQL 的后端实现
  - 按照 Java 版的领域模型和数据结构迁移，实现兼容同一套前端（接口风格与权限模型对齐）

- `next-admin`：
  - 基于 Next.js 14 + React + Chakra UI 的管理后台前端
  - 以 `admin` 中 Vue3 管理端为蓝本迁移实现，复用同一套菜单、权限与接口规范

---

## 技术栈

- 前端（Vue3 管理端 `admin`）
  - Vue 3、TypeScript、Vite、Pinia、Vue Router
  - Arco Design Vue、ECharts 等

- 前端（Next 管理端 `next-admin`）
  - React 18、Next.js 14 App Router
  - Chakra UI、TypeScript

- 后端（Go 实现 `go-backend`）
  - Go 1.22+（go.mod 中为 1.24.0，建议使用最新版 Go）
  - Gin、JWT、BCrypt
  - PostgreSQL（自动建表 + 初始化管理员、角色、菜单等）

- 后端（Java 原版 `backend`）
  - Spring Boot 3、Sa-Token、MyBatis-Plus、Redis、Liquibase 等
  - 默认 MySQL（也提供 PostgreSQL 配置模板）

---

## 环境准备

请根据需要选择对应模块运行，不一定全部都要启动。

- 通用
  - Git
  - Node.js ≥ 18（建议 18/20）
  - 推荐包管理器：`pnpm`（也可使用 `npm`/`yarn`）

- Go 后端（`go-backend`）
  - Go ≥ 1.22
  - PostgreSQL ≥ 13
    - 默认连接配置：
      - `DB_HOST=127.0.0.1`
      - `DB_PORT=5432`
      - `DB_USER=postgres`
      - `DB_PWD=123456`
      - `DB_NAME=nv_admin`

- Java 后端（`backend`，可选）
  - JDK 17
  - Maven 3.8+

---

## 快速开始

### 1. 准备数据库（Go 后端）

1. 启动 PostgreSQL
2. 创建数据库（可用默认值）：

   ```sql
   CREATE DATABASE nv_admin;
   ```

3. 确认账号密码与默认配置一致（或通过环境变量覆盖）：
   - 用户名：`postgres`
   - 密码：`123456`
   - 数据库名：`nv_admin`

`go-backend` 在启动时会执行自动迁移脚本（见 `go-backend/internal/infrastructure/db/migrate.go`）：

- 自动创建 `sys_user、sys_role、sys_menu、sys_dept、sys_dict` 等核心表
- 自动插入默认管理员、角色及菜单数据
- 多次启动是幂等的，可安全重复执行

---

### 2. 启动 Go 后端（推荐）

Go 后端主入口：`go-backend/cmd/admin/main.go`，默认 HTTP 端口 `4398`。

```bash
# 进入 go 后端目录
cd go-backend

# 启动服务（开发环境）
go run ./cmd/admin
```

常用环境变量（可选）：

- 数据库连接：
  - `DB_HOST`（默认 `127.0.0.1`）
  - `DB_PORT`（默认 `5432`）
  - `DB_USER`（默认 `postgres`）
  - `DB_PWD`（默认 `123456`）
  - `DB_NAME`（默认 `nv_admin`）
- 服务端口：
  - `HTTP_PORT`（默认 `4398`）
- 安全相关：
  - `AUTH_RSA_PRIVATE_KEY`：登录时前端加密密码的私钥，对应 Java 版的 RSA 配置（已提供默认与 Java 保持一致）
  - `AUTH_JWT_SECRET`：JWT 签名密钥（默认与 Java 版保持一致）

启动成功后，接口默认地址为：

- `http://localhost:4398`

> 注意：Go 后端 CORS 默认为允许 `http://localhost:3000`（Next.js 开发端口），使用 Vue 前端开发时通过 Vite 代理转发，无需额外配置。

---

### 3. 启动 Vue3 管理端（`admin`）

Vue3 管理端在开发模式下默认运行在 `http://localhost:4399`，并通过 Vite 代理将接口转发到 `http://localhost:4398` 的后端。

1. 安装依赖

   ```bash
   cd admin

   # 推荐
   pnpm install
   # 或者
   # npm install
   # yarn
   ```

2. 确认开发环境配置：`admin/.env.development`

   默认关键配置（可根据需要调整）：

   ```env
   # 接口前缀（与 Vite 代理匹配）
   VITE_API_PREFIX = '/dev-api'

   # 后端 API 地址（Go 或 Java 后端）
   VITE_API_BASE_URL = 'http://localhost:4398'

   # WebSocket 地址（如后端支持）
   VITE_API_WS_URL = 'ws://localhost:4398'

   # 前端开发端口
   VITE_PORT = 4399
   ```

3. 启动开发服务器

   ```bash
   pnpm dev
   ```

   Vite 默认浏览器自动打开，访问：

   - `http://localhost:4399`

只要 `go-backend` 在 `4398` 端口运行，前端接口即可正常通过 `/dev-api` 代理访问。

---

### 4. 启动 Next.js 管理端（`next-admin`）

Next.js 管理端默认开发端口为 `3000`，通过直接请求后端 `http://localhost:4398`。

1. 安装依赖

   ```bash
   cd next-admin

   # 推荐
   pnpm install
   # 或者
   # npm install
   # yarn
   ```

2. 配置后端地址（可选）

   `next-admin/src/utils/api.ts` 中默认 API 地址：

   ```ts
   const DEFAULT_API_BASE_URL = "http://localhost:4398";
   export const API_BASE_URL =
     process.env.NEXT_PUBLIC_API_BASE_URL ?? DEFAULT_API_BASE_URL;
   ```

   如需修改后端地址，可通过环境变量覆盖：

   ```bash
   # 示例：使用自定义后端地址
   NEXT_PUBLIC_API_BASE_URL=http://127.0.0.1:4398 pnpm dev
   ```

3. 启动开发服务器

   ```bash
   pnpm dev
   ```

   访问：

   - `http://localhost:3000`

> 提示：Go 后端内置 CORS 规则，已允许 `http://localhost:3000` 发起跨域调用。

---

### 5. 管理后台登录

初始管理员账号来自原版 ContiNew Admin 数据脚本（见 `backend/continew-webapi/src/main/resources/db/changelog/postgresql/main_data.sql`）：

- 账号：`admin`
- 密码：`admin123`

Go 后端的自动迁移脚本复用了同一套加密密码，因此在全新 PostgreSQL 数据库环境下，默认也可以使用相同账号密码登录。

---

## Java 原版后端（可选）

`backend` 目录完整保留了原版 **ContiNew Admin** 的 Java 后端，可用于对照或需要 Java 技术栈时使用。

典型启动方式（需 MySQL 和 Redis 环境）：

```bash
cd backend

# 修改 config/application-dev.yml，配置数据库和 Redis

# 启动 API 模块
mvn clean spring-boot:run -pl continew-webapi
```

默认端口与前端一致：

- HTTP 端口：`4398`
- 默认前端地址：`http://localhost:4399`（见 `application-dev.yml` 中 `project.url`）

通常推荐在本仓库中使用 `go-backend` 作为后端主实现，`backend` 更多用于参考对照。

---

## 目录结构概览

```text
.
├── admin              # Vue3 管理端（基于 ContiNew Admin UI）
│   ├── src
│   ├── public
│   └── vite.config.ts
├── backend            # Java 原版后端（ContiNew Admin）
│   ├── continew-webapi
│   ├── continew-module-system
│   ├── continew-common
│   └── docker         # Java 版配套 Docker 示例
├── go-backend         # Go 后端实现（兼容前端接口）
│   ├── cmd
│   │   └── admin      # Go 服务入口 main.go
│   ├── internal
│   │   ├── application
│   │   ├── domain
│   │   ├── infrastructure
│   │   └── interfaces
│   └── go.mod
└── next-admin         # Next.js 管理端（迁移自 Vue3 admin）
    ├── app
    ├── src
    └── next.config.mjs
```

---

## 配置速查表

- Go 后端（`go-backend`）环境变量：
  - `DB_HOST` / `DB_PORT` / `DB_USER` / `DB_PWD` / `DB_NAME`
  - `DB_SSLMODE`（默认 `disable`）
  - `HTTP_PORT`（默认 `4398`）
  - `AUTH_RSA_PRIVATE_KEY`（默认与 Java 一致）
  - `AUTH_JWT_SECRET`（默认与 Java 一致）

- Vue3 前端（`admin`）：
  - `.env.development`：
    - `VITE_API_BASE_URL`：后端地址（默认 `http://localhost:4398`）
    - `VITE_PORT`：前端开发端口（默认 `4399`）

- Next.js 前端（`next-admin`）：
  - 环境变量：
    - `NEXT_PUBLIC_API_BASE_URL`：后端地址（默认 `http://localhost:4398`）

---

## 致谢与来源

本项目基于以下优秀开源项目二次开发与迁移：

- ContiNew Admin：
  - GitHub：https://github.com/continew-org/continew-admin
  - Gitee：https://gitee.com/continew/continew-admin
- ContiNew Admin UI：
  - GitHub：https://github.com/continew-org/continew-admin-ui
  - Gitee：https://gitee.com/continew/continew-admin-ui

在使用或发布本项目时，请同时遵守原项目的 Apache-2.0 许可证条款。

