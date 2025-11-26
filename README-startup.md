# 启动指南（Avalon Admin Monorepo）

> 更新日期：2025-11-26（执行者：Codex）

本仓库包含多种后端实现（Java / Go / Node / Python / Rust / PHP）和多端前端（PC 管理端、H5、小程序、RN、Flutter 等）。下面给出推荐的启动组合与最常用的启动命令，方便本地开发联调。

## 1. 推荐组合

- 后端：`backend-python`（FastAPI，实现与 Java/Go 兼容的 API）
- 管理端前端（Vue3）：`pc-admin-vue3`
- 可选管理端前端（Next.js）：`pc-admin-nextjs`

其他后端实现（`backend-go`、`backend-java`、`backend-node`、`backend-rust`、`backend-php`）可按需替换，只要保持端口和接口规范一致，前端即可复用。

---

## 2. 数据库与环境变量

所有后端统一约定使用 PostgreSQL（除 Java 原版仍以 MySQL 为主），典型配置如下：

- 主机：`DB_HOST=127.0.0.1`
- 端口：`DB_PORT=5432`
- 用户：`DB_USER=postgres`
- 密码：`DB_PWD=123456`
- 库名：`DB_NAME=nv_admin`

认证相关环境变量（多个后端共用）：

- `AUTH_RSA_PRIVATE_KEY`：登录密码 RSA 私钥（Base64 编码 PKCS#8），用于解密前端加密密码。
- `AUTH_JWT_SECRET`：JWT 签名密钥。
- `AUTH_JWT_TTL_HOURS`：Token 有效期（小时），默认 `24`。

> 建议：在仓库根目录创建统一的 `.env` 或各后端子目录下创建本地环境文件，保持这些变量一致，方便前端在不同后端之间切换。

---

## 3. 启动 Python 后端（backend-python）

Python 版本后端基于 FastAPI，实现了认证、通用接口以及大部分 `/system/*` 管理接口，默认端口 `4398`。

```bash
cd backend-python

# 推荐：创建虚拟环境并安装依赖
python3 -m venv .venv
source .venv/bin/activate    # Windows: .venv\Scripts\activate
pip install -r requirements.txt

# 启动开发服务
uvicorn app.main:app --reload --port 4398
```

如果 PostgreSQL 与环境变量配置正确，启动后即可通过：

- `http://localhost:4398/auth/login`
- `http://localhost:4398/common/*`
- `/system/user`、`/system/role`、`/system/menu`、`/system/dept`、`/system/dict`、`/system/option`

等接口进行访问。

---

## 4. 启动 Node 后端（backend-node，可选）

Node 版本基于 NestJS + Prisma，兼容 Java/Go 的 API 协议。

```bash
cd backend-node

# 安装依赖（建议使用 pnpm）
pnpm install        # 或 npm install / yarn

# 生成 Prisma Client
pnpm prisma:generate

# 编译并启动
pnpm build
pnpm start          # 默认端口通常也为 4398（可在 main.ts 中调整）
```

> 注意：确保 `DATABASE_URL` 或 `DB_*` 环境变量指向与 Java/Go 同源的 PostgreSQL 数据库。

---

## 5. 启动 Go 后端（backend-go，可选）

Go 版后端入口位于 `backend-go/cmd/admin/main.go`：

```bash
cd backend-go
go run ./cmd/admin
```

环境变量与端口说明参考该项目内的 README 或 `internal/infrastructure/db/migrate.go` 中的注释。

---

## 6. 启动 Vue3 管理端（pc-admin-vue3）

Vue3 管理端是主管理后台前端，默认通过 Vite 运行在 `http://localhost:4399`。

```bash
cd pc-admin-vue3

# 安装依赖
pnpm install          # 或 npm install / yarn

# 启动开发服务器
pnpm dev
```

开发环境下 API 代理配置通常在：

- `pc-admin-vue3/.env.development`

请将其中的后端地址指向你希望联调的后端，例如：

```env
VITE_API_BASE_URL=http://localhost:4398
```

这样即可通过 Vue3 管理端直接联调 Python/Go/Node 等后端。

---

## 7. 启动 Next.js 管理端（pc-admin-nextjs）

Next.js 管理端基于 App Router，默认端口 `3000`：

```bash
cd pc-admin-nextjs

pnpm install
pnpm dev
```

通常会在环境变量中配置后端地址，例如：

```env
NEXT_PUBLIC_API_BASE_URL=http://localhost:4398
```

具体变量名请参考 `pc-admin-nextjs` 项目内的 README 或 `.env.example` 文件。

---

## 8. 其他前端与客户端

本仓库还包含：

- `h5-nextjs`：H5 端 Next.js 应用
- `app-uniapp`：UniApp 小程序 / H5 应用
- `app-rn`：React Native 应用
- `app-flutter`：Flutter 应用

它们的启动方式大致为：

1. 进入子目录。
2. 按照项目 README 或 `package.json` / `pubspec.yaml` 中的脚本说明安装依赖并运行。
3. 确保各自的 API 基地址与所选择的后端端口一致（如 `http://localhost:4398`）。

---

## 9. 常见联调组合示例

**组合 A：Python 后端 + Vue 管理端**

```bash
# 终端 1：Python 后端
cd backend-python
uvicorn app.main:app --reload --port 4398

# 终端 2：Vue 管理端
cd pc-admin-vue3
pnpm dev
```

浏览器访问：`http://localhost:4399`。

**组合 B：Node 后端 + Vue 管理端**

```bash
# 终端 1：Node 后端
cd backend-node
pnpm start

# 终端 2：Vue 管理端（同上）
cd pc-admin-vue3
pnpm dev
```

如需切换为 Go 或 Java 后端，只需在前端环境变量中调整 API 地址并启动对应后端即可，无需修改前端代码。 

