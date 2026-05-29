# Rust Next.js Admin Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust + Next.js/shadcn admin system that aligns with the existing Java + Vue3 admin system, including API-compatible RBAC and SQL-level data permissions.

**Architecture:** Implement the backend as a lightweight-DDD Rust service with `interfaces/http`, `application`, `domain`, `infrastructure`, and `shared` layers. Keep API paths and response envelopes compatible with the current Vue3 frontend while building the new Next.js frontend on shadcn/ui. Push data permission filters into PostgreSQL queries through repository helpers.

**Tech Stack:** Rust, axum, tokio, sqlx, PostgreSQL, jsonwebtoken, bcrypt/argon2, tower-http, tracing, Next.js App Router, TypeScript, shadcn/ui, Tailwind CSS, TanStack Table, react-hook-form, Zod, Playwright.

---

## Ground Rules

- Work on the current branch as requested by the user.
- Preserve unrelated staged deletions already present in the worktree. When committing, commit only paths touched by the task.
- Use TDD for policy-heavy code: response envelope, auth guard, permission checks, data-scope SQL generation.
- Keep all frontend pages operational; do not create a marketing landing page.
- Use existing Java/Vue3 files as reference:
  - `backend-java/continew-webapi/src/main/java/top/continew/admin/controller`
  - `backend-java/continew-common/src/main/java/top/continew/admin/common/enums/DataScopeEnum.java`
  - `backend-go/internal/infrastructure/db/migrate.go`
  - `pc-admin-vue3/src/apis`
  - `pc-admin-vue3/src/views`

---

### Task 1: Scaffold Rust Backend Workspace

**Files:**
- Create: `backend-rust/Cargo.toml`
- Create: `backend-rust/.env.example`
- Create: `backend-rust/src/main.rs`
- Create: `backend-rust/src/lib.rs`
- Create: `backend-rust/src/shared/mod.rs`
- Create: `backend-rust/src/shared/config.rs`
- Create: `backend-rust/src/shared/error.rs`
- Create: `backend-rust/src/shared/response.rs`
- Create: `backend-rust/src/shared/pagination.rs`
- Create: `backend-rust/src/shared/time.rs`
- Create: `backend-rust/src/interfaces/mod.rs`
- Create: `backend-rust/src/interfaces/http/mod.rs`
- Create: `backend-rust/src/application/mod.rs`
- Create: `backend-rust/src/domain/mod.rs`
- Create: `backend-rust/src/infrastructure/mod.rs`

**Step 1: Create Cargo manifest**

Use this dependency baseline:

```toml
[package]
name = "backend-rust"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
async-trait = "0.1"
axum = { version = "0.7", features = ["macros", "multipart"] }
bcrypt = "0.15"
chrono = { version = "0.4", features = ["serde"] }
dotenvy = "0.15"
jsonwebtoken = "9"
mime_guess = "2"
rand = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "chrono", "json", "uuid", "migrate"] }
thiserror = "1"
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["cors", "trace", "fs"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
uuid = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
http-body-util = "0.1"
tower = "0.5"
```

**Step 2: Write response tests**

Create unit tests in `src/shared/response.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ok_response_uses_existing_envelope() {
        let res = ApiResponse::ok(json!({"id": "1"}));
        assert_eq!(res.code, "200");
        assert_eq!(res.msg, "成功");
    }

    #[test]
    fn fail_response_uses_existing_envelope() {
        let res: ApiResponse<()> = ApiResponse::fail("403", "没有访问权限，请联系管理员授权");
        assert_eq!(res.code, "403");
        assert_eq!(res.msg, "没有访问权限，请联系管理员授权");
    }
}
```

**Step 3: Run tests to verify baseline**

Run:

```bash
cd backend-rust
cargo test shared::response
```

Expected: tests compile and pass.

**Step 4: Implement app entry**

`main.rs` should load config, initialize tracing, connect PostgreSQL, run migrations later, build the router, and serve on `HTTP_PORT` defaulting to `4398`.

**Step 5: Add health route**

Create `GET /health` returning `{ code: "200", msg: "成功", data: "ok" }`.

**Step 6: Verify**

Run:

```bash
cd backend-rust
cargo fmt --check
cargo test
cargo run
```

Expected: server starts and `/health` returns success.

**Step 7: Commit**

```bash
git add backend-rust
git commit --only backend-rust -m "feat: scaffold rust admin backend"
```

---

### Task 2: Add PostgreSQL Migrations and Seed Data

**Files:**
- Create: `backend-rust/migrations/202605290001_create_sys_core.sql`
- Create: `backend-rust/migrations/202605290002_seed_sys_core.sql`
- Modify: `backend-rust/src/main.rs`
- Create: `backend-rust/src/infrastructure/db.rs`

**Step 1: Write migration test plan**

Create a manual smoke checklist in `backend-rust/README.md` for migration commands:

```bash
DATABASE_URL=postgres://postgres:123456@127.0.0.1:5432/nv_admin sqlx migrate run
```

Expected tables:

```sql
select to_regclass('public.sys_user');
select to_regclass('public.sys_role');
select to_regclass('public.sys_menu');
```

**Step 2: Implement core schema**

Create tables aligned with the existing PostgreSQL versions:

- `sys_user`
- `sys_role`
- `sys_user_role`
- `sys_menu`
- `sys_role_menu`
- `sys_role_dept`
- `sys_dept`
- `sys_dict`
- `sys_dict_item`
- `sys_option`
- `sys_storage`
- `sys_file`
- `sys_client`
- `sys_log`
- `sys_online_user`

Include indexes for:

- `sys_user.username`, `email`, `phone`, `dept_id`
- `sys_role.code`, `name`
- `sys_menu.parent_id`, `permission`
- `sys_dept.parent_id`
- `sys_log.create_user`, `create_time`, `module`, `type`
- `sys_file.sha256`, `parent_path`, `storage_id`, `create_user`

**Step 3: Implement seed data**

Seed:

- admin user `admin` with password `admin123`
- root department
- roles `admin` and `general`
- all menu/button records used by `pc-admin-vue3/src/apis`
- role-menu bindings giving admin all menus
- role-dept binding for default custom checks
- default dictionaries: user gender/status, client type, auth type, storage type
- site/security/login options
- local storage config
- PC client config

Use `ON CONFLICT DO NOTHING` or deterministic `WHERE NOT EXISTS`.

**Step 4: Wire migrations**

In startup, run:

```rust
sqlx::migrate!("./migrations").run(&pool).await?;
```

Gate this behind env var `DB_AUTO_MIGRATE=true` so production can disable auto-DDL.

**Step 5: Verify**

Run:

```bash
cd backend-rust
cargo fmt --check
cargo test
DATABASE_URL=postgres://postgres:123456@127.0.0.1:5432/nv_admin sqlx migrate run
```

Expected: migrations complete without error.

**Step 6: Commit**

```bash
git add backend-rust
git commit --only backend-rust -m "feat: add rust admin database migrations"
```

---

### Task 3: Implement Auth, JWT, Password, and Current User Context

**Files:**
- Create: `backend-rust/src/domain/auth/mod.rs`
- Create: `backend-rust/src/domain/auth/model.rs`
- Create: `backend-rust/src/application/auth/mod.rs`
- Create: `backend-rust/src/application/auth/service.rs`
- Create: `backend-rust/src/infrastructure/security/mod.rs`
- Create: `backend-rust/src/infrastructure/security/jwt.rs`
- Create: `backend-rust/src/infrastructure/security/password.rs`
- Create: `backend-rust/src/infrastructure/persistence/mod.rs`
- Create: `backend-rust/src/infrastructure/persistence/user_repository.rs`
- Create: `backend-rust/src/interfaces/http/auth.rs`
- Create: `backend-rust/src/interfaces/http/extractor.rs`
- Modify: `backend-rust/src/interfaces/http/mod.rs`

**Step 1: Write JWT tests**

Create tests in `jwt.rs`:

```rust
#[tokio::test]
async fn token_round_trips_user_id() {
    let service = JwtService::new("secret".to_string(), 24);
    let token = service.issue(1, "admin").unwrap();
    let claims = service.parse(&format!("Bearer {token}")).unwrap();
    assert_eq!(claims.user_id, 1);
    assert_eq!(claims.username, "admin");
}
```

**Step 2: Write password tests**

Support existing `{bcrypt}` prefix:

```rust
#[test]
fn verifies_prefixed_bcrypt_password() {
    let hash = "{bcrypt}$2a$10$4jGwK2BMJ7FgVR.mgwGodey8.xR8FLoU1XSXpxJ9nZQt.pufhasSa";
    assert!(verify_password("admin123", hash).unwrap());
}
```

**Step 3: Implement login DTOs**

`POST /auth/login` accepts Vue3-compatible payload fields. At minimum support account login:

```json
{
  "username": "admin",
  "password": "admin123",
  "captchaKey": "",
  "captcha": ""
}
```

Return:

```json
{
  "token": "...",
  "expire": "2026-05-30T..."
}
```

wrapped by the common envelope.

**Step 4: Implement current user extractor**

`CurrentUser` should contain:

```rust
pub struct CurrentUser {
    pub id: i64,
    pub username: String,
    pub dept_id: i64,
    pub roles: Vec<RoleContext>,
    pub permissions: Vec<String>,
}
```

**Step 5: Implement routes**

- `POST /auth/login`
- `POST /auth/logout`
- `GET /auth/user/info`

`/auth/logout` can invalidate `sys_online_user` records if online sessions are implemented now; otherwise return success and add TODO only in code if a concrete follow-up issue exists.

**Step 6: Verify**

Run:

```bash
cd backend-rust
cargo test auth security
cargo run
curl -s -X POST http://localhost:4398/auth/login -H 'content-type: application/json' -d '{"username":"admin","password":"admin123"}'
```

Expected: token is returned.

**Step 7: Commit**

```bash
git add backend-rust
git commit --only backend-rust -m "feat: implement rust auth login"
```

---

### Task 4: Implement RBAC Permissions and Route Tree

**Files:**
- Create: `backend-rust/src/domain/rbac/mod.rs`
- Create: `backend-rust/src/domain/rbac/model.rs`
- Create: `backend-rust/src/application/rbac/mod.rs`
- Create: `backend-rust/src/application/rbac/service.rs`
- Create: `backend-rust/src/infrastructure/persistence/rbac_repository.rs`
- Create: `backend-rust/src/interfaces/http/middleware/permission.rs`
- Modify: `backend-rust/src/interfaces/http/auth.rs`
- Modify: `backend-rust/src/interfaces/http/mod.rs`

**Step 1: Write permission aggregation tests**

Test admin wildcard:

```rust
#[test]
fn admin_role_has_all_permissions() {
    let ctx = PermissionContext {
        role_codes: vec!["admin".to_string()],
        permissions: vec![],
    };
    assert!(ctx.has("system:user:delete"));
}
```

Test normal role:

```rust
#[test]
fn normal_role_requires_explicit_permission() {
    let ctx = PermissionContext {
        role_codes: vec!["general".to_string()],
        permissions: vec!["system:user:list".to_string()],
    };
    assert!(ctx.has("system:user:list"));
    assert!(!ctx.has("system:user:delete"));
}
```

**Step 2: Implement `GET /auth/user/route`**

Build a menu tree from `sys_menu`, matching Vue3 fields:

```json
{
  "path": "/system/user",
  "name": "SystemUser",
  "component": "system/user/index",
  "redirect": null,
  "meta": {
    "title": "用户管理",
    "icon": "user",
    "hidden": false,
    "cache": false
  },
  "children": []
}
```

**Step 3: Implement permission middleware**

Expose a helper for handlers:

```rust
pub fn require_permission(user: &CurrentUser, permission: &'static str) -> Result<(), AppError>;
```

Use it in system handlers instead of relying on frontend visibility.

**Step 4: Verify**

Run:

```bash
cd backend-rust
cargo test rbac
```

Then login and call:

```bash
curl -s http://localhost:4398/auth/user/info -H "Authorization: Bearer $TOKEN"
curl -s http://localhost:4398/auth/user/route -H "Authorization: Bearer $TOKEN"
```

Expected: roles and permissions include admin access; route tree includes system and monitor menus.

**Step 5: Commit**

```bash
git add backend-rust
git commit --only backend-rust -m "feat: add rbac permissions and route tree"
```

---

### Task 5: Implement Data Scope Resolver

**Files:**
- Create: `backend-rust/src/domain/data_scope/mod.rs`
- Create: `backend-rust/src/domain/data_scope/model.rs`
- Create: `backend-rust/src/application/data_scope/mod.rs`
- Create: `backend-rust/src/application/data_scope/resolver.rs`
- Create: `backend-rust/src/infrastructure/persistence/dept_repository.rs`
- Create: `backend-rust/tests/data_scope_tests.rs`

**Step 1: Write resolver tests**

Test all-data short-circuit:

```rust
#[test]
fn all_data_role_returns_unrestricted_filter() {
    let user = sample_user_with_scope(1);
    let filter = resolve_data_scope(&user, &sample_target(), &sample_dept_tree()).unwrap();
    assert!(filter.is_unrestricted());
}
```

Test self-only:

```rust
#[test]
fn self_scope_uses_create_user_column() {
    let user = sample_user_with_scope(4);
    let filter = resolve_data_scope(&user, &sample_target(), &sample_dept_tree()).unwrap();
    assert_eq!(filter.to_debug_sql(), "(create_user = $user_id)");
}
```

Test department and children:

```rust
#[test]
fn dept_and_child_scope_collects_descendants() {
    let user = sample_user_with_dept_and_scope(10, 2);
    let filter = resolve_data_scope(&user, &sample_target(), &dept_tree([(10, 11), (11, 12)])).unwrap();
    assert_eq!(filter.dept_ids(), vec![10, 11, 12]);
}
```

**Step 2: Implement model**

Implement:

```rust
pub enum DataScope {
    All,
    DeptAndChild,
    Dept,
    SelfOnly,
    Custom,
}

pub struct DataPermissionTarget<'a> {
    pub dept_column: Option<&'a str>,
    pub user_column: Option<&'a str>,
}

pub struct DataScopeFilter {
    pub unrestricted: bool,
    pub dept_ids: Vec<i64>,
    pub self_user_id: Option<i64>,
}
```

**Step 3: Implement SQL binding helper**

Use a helper that appends conditions to `sqlx::QueryBuilder<Postgres>` with bound values. Do not concatenate untrusted values. Column names must come from hardcoded repository targets only.

**Step 4: Verify**

Run:

```bash
cd backend-rust
cargo test data_scope
```

Expected: all resolver tests pass.

**Step 5: Commit**

```bash
git add backend-rust
git commit --only backend-rust -m "feat: add sql data scope resolver"
```

---

### Task 6: Implement Common APIs, Dept, Menu, Role

**Files:**
- Create: `backend-rust/src/interfaces/http/common.rs`
- Create: `backend-rust/src/interfaces/http/system/dept.rs`
- Create: `backend-rust/src/interfaces/http/system/menu.rs`
- Create: `backend-rust/src/interfaces/http/system/role.rs`
- Create: `backend-rust/src/interfaces/http/system/mod.rs`
- Create: `backend-rust/src/application/system/dept_service.rs`
- Create: `backend-rust/src/application/system/menu_service.rs`
- Create: `backend-rust/src/application/system/role_service.rs`
- Create: `backend-rust/src/infrastructure/persistence/system_dept_repository.rs`
- Create: `backend-rust/src/infrastructure/persistence/system_menu_repository.rs`
- Create: `backend-rust/src/infrastructure/persistence/system_role_repository.rs`

**Step 1: Write API compatibility tests**

Use `axum` route tests for:

- `GET /common/tree/dept`
- `GET /common/tree/menu`
- `GET /system/dept/tree`
- `GET /system/menu/tree`
- `GET /system/role/list`

Assert response envelope and key field names: `id`, `name`, `children`, `dataScope`, `menuIds`, `deptIds`.

**Step 2: Implement department APIs**

Implement:

- `GET /system/dept/tree`
- `GET /system/dept/:id`
- `POST /system/dept`
- `PUT /system/dept/:id`
- `DELETE /system/dept`
- `GET /system/dept/export`

Apply data scope to list/tree queries using `dept_id` or department tree restrictions.

**Step 3: Implement menu APIs**

Implement:

- `GET /system/menu/tree`
- `GET /system/menu/:id`
- `POST /system/menu`
- `PUT /system/menu/:id`
- `DELETE /system/menu`
- `DELETE /system/menu/cache`

`DELETE /cache` can return success because Rust has no menu cache yet.

**Step 4: Implement role APIs**

Implement:

- `GET /system/role/list`
- `GET /system/role/:id`
- `POST /system/role`
- `PUT /system/role/:id`
- `DELETE /system/role`
- `PUT /system/role/:id/permission`
- `GET /system/role/:id/user/id`

Preserve system-role protections from Java: system role code cannot be changed, built-in role data scope cannot be changed unexpectedly, and built-in roles cannot be deleted.

**Step 5: Verify**

Run:

```bash
cd backend-rust
cargo test system::dept system::menu system::role
cargo clippy --all-targets -- -D warnings
```

Expected: tests and clippy pass.

**Step 6: Commit**

```bash
git add backend-rust
git commit --only backend-rust -m "feat: implement dept menu role APIs"
```

---

### Task 7: Implement User Management and Profile APIs

**Files:**
- Create: `backend-rust/src/interfaces/http/system/user.rs`
- Create: `backend-rust/src/interfaces/http/user_profile.rs`
- Create: `backend-rust/src/application/system/user_service.rs`
- Create: `backend-rust/src/application/user_profile_service.rs`
- Create: `backend-rust/src/infrastructure/persistence/system_user_repository.rs`

**Step 1: Write user list data-scope tests**

Set up test rows:

- admin in dept 1
- manager in dept 10
- user in dept 11
- records created by different users

Assert:

- admin sees all
- dept-and-child sees dept 10 and 11
- dept-only sees dept 10
- self-only sees `create_user = current_user`
- custom sees `sys_role_dept` departments only

**Step 2: Implement user list/page APIs**

Implement:

- `GET /system/user`
- `GET /system/user/list`
- `GET /system/user/:id`

Support Vue3 query fields:

- `description`
- `status`
- `createTime`
- `deptId`
- `sort`
- `roleId`
- pagination fields

Apply data scope.

**Step 3: Implement user mutation APIs**

Implement:

- `POST /system/user`
- `PUT /system/user/:id`
- `DELETE /system/user`
- `PATCH /system/user/:id/password`
- `PATCH /system/user/:id/role`

Validate unique username, email, and phone. Prevent deleting built-in system users.

**Step 4: Implement import/export compatibility**

Implement endpoints:

- `GET /system/user/export`
- `GET /system/user/import/template`
- `POST /system/user/import/parse`
- `POST /system/user/import`

Use CSV output first if XLSX support is not introduced. Keep response envelope compatible for parse/import metadata.

**Step 5: Implement profile APIs**

Implement:

- `PATCH /user/profile/avatar`
- `PATCH /user/profile/basic/info`
- `PATCH /user/profile/password`
- `PATCH /user/profile/phone`
- `PATCH /user/profile/email`
- `GET /user/profile/social`
- `POST /user/profile/social/:source`
- `DELETE /user/profile/social/:source`

Social APIs can return empty/success compatibility responses until actual providers are configured.

**Step 6: Verify**

Run:

```bash
cd backend-rust
cargo test system::user user_profile data_scope
cargo clippy --all-targets -- -D warnings
```

Expected: tests and clippy pass.

**Step 7: Commit**

```bash
git add backend-rust
git commit --only backend-rust -m "feat: implement user management APIs"
```

---

### Task 8: Implement Dict, Option, Storage, Client, File APIs

**Files:**
- Create: `backend-rust/src/interfaces/http/system/dict.rs`
- Create: `backend-rust/src/interfaces/http/system/option.rs`
- Create: `backend-rust/src/interfaces/http/system/storage.rs`
- Create: `backend-rust/src/interfaces/http/system/client.rs`
- Create: `backend-rust/src/interfaces/http/system/file.rs`
- Create: `backend-rust/src/application/system/dict_service.rs`
- Create: `backend-rust/src/application/system/option_service.rs`
- Create: `backend-rust/src/application/system/storage_service.rs`
- Create: `backend-rust/src/application/system/client_service.rs`
- Create: `backend-rust/src/application/system/file_service.rs`
- Create: `backend-rust/src/infrastructure/storage/local.rs`
- Create: `backend-rust/src/infrastructure/persistence/system_misc_repositories.rs`

**Step 1: Write CRUD compatibility tests**

For dict, option, storage, and client, assert:

- create persists
- update changes fields
- list returns Vue3 field names
- delete respects system/default protection

**Step 2: Implement dict APIs**

Implement:

- `GET /system/dict/list`
- `GET /system/dict/:id`
- `POST /system/dict`
- `PUT /system/dict/:id`
- `DELETE /system/dict`
- `DELETE /system/dict/cache/:code`
- `GET /system/dict/item`
- `GET /system/dict/item/:id`
- `POST /system/dict/item`
- `PUT /system/dict/item/:id`
- `DELETE /system/dict/item`

**Step 3: Implement option APIs**

Implement:

- `GET /system/option`
- `PUT /system/option`
- `PATCH /system/option/value`
- `GET /common/dict/option/site`

Update option values by `id` and `code`. Reset values to `default_value`.

**Step 4: Implement storage and client APIs**

Implement:

- `GET /system/storage/list`
- `GET /system/storage/:id`
- `POST /system/storage`
- `PUT /system/storage/:id`
- `DELETE /system/storage`
- `PUT /system/storage/:id/status`
- `PUT /system/storage/:id/default`
- `GET /system/client`
- `GET /system/client/:id`
- `POST /system/client`
- `PUT /system/client/:id`
- `DELETE /system/client`

Ensure only one storage row can be default.

**Step 5: Implement file APIs**

Implement local storage first:

- `POST /system/file/upload`
- `GET /system/file`
- `PUT /system/file/:id`
- `DELETE /system/file`
- `GET /system/file/statistics`
- `GET /system/file/check`
- `POST /system/file/dir`
- `GET /system/file/dir/:id/size`
- `POST /common/file`

Compute SHA-256 before writing file records. Store files under `backend-rust/data/file` by default.

**Step 6: Verify**

Run:

```bash
cd backend-rust
cargo test system::dict system::option system::storage system::client system::file
cargo clippy --all-targets -- -D warnings
```

Expected: tests and clippy pass.

**Step 7: Commit**

```bash
git add backend-rust
git commit --only backend-rust -m "feat: implement system config and file APIs"
```

---

### Task 9: Implement Captcha, Logging, and Online Monitor

**Files:**
- Create: `backend-rust/src/interfaces/http/captcha.rs`
- Create: `backend-rust/src/interfaces/http/monitor/log.rs`
- Create: `backend-rust/src/interfaces/http/monitor/online.rs`
- Create: `backend-rust/src/interfaces/http/monitor/mod.rs`
- Create: `backend-rust/src/interfaces/http/middleware/access_log.rs`
- Create: `backend-rust/src/application/monitor/log_service.rs`
- Create: `backend-rust/src/application/monitor/online_service.rs`
- Create: `backend-rust/src/infrastructure/persistence/log_repository.rs`
- Create: `backend-rust/src/infrastructure/persistence/online_repository.rs`

**Step 1: Write log middleware tests**

Assert that successful login writes a login log and authenticated mutation writes an operation log with:

- module
- description
- request method
- request URI
- status
- create user
- create time

**Step 2: Implement captcha compatibility**

Implement:

- `GET /captcha/image`
- `GET /captcha/behavior`
- `POST /captcha/behavior`
- `GET /captcha/mail`

For behavior captcha, return compatibility data accepted by the frontend. For email captcha, return success in local dev unless SMTP is configured.

**Step 3: Implement log APIs**

Implement:

- `GET /system/log`
- `GET /system/log/:id`
- `GET /system/log/export/login`
- `GET /system/log/export/operation`

Support filters used by Vue3 log pages. Apply data scope if logs are tied to `create_user`.

**Step 4: Implement online APIs**

Implement:

- `GET /monitor/online`
- `DELETE /monitor/online/:token`

Persist online sessions in `sys_online_user` first. Later Redis can replace this behind the same service trait.

**Step 5: Verify**

Run:

```bash
cd backend-rust
cargo test captcha monitor
cargo clippy --all-targets -- -D warnings
```

Expected: tests and clippy pass.

**Step 6: Commit**

```bash
git add backend-rust
git commit --only backend-rust -m "feat: implement monitor and captcha APIs"
```

---

### Task 10: Scaffold Next.js with shadcn/ui

**Files:**
- Create: `pc-admin-nextjs/package.json`
- Create: `pc-admin-nextjs/next.config.mjs`
- Create: `pc-admin-nextjs/tsconfig.json`
- Create: `pc-admin-nextjs/postcss.config.mjs`
- Create: `pc-admin-nextjs/tailwind.config.ts`
- Create: `pc-admin-nextjs/components.json`
- Create: `pc-admin-nextjs/app/globals.css`
- Create: `pc-admin-nextjs/app/layout.tsx`
- Create: `pc-admin-nextjs/app/page.tsx`
- Create: `pc-admin-nextjs/src/lib/utils.ts`
- Create: `pc-admin-nextjs/src/lib/api.ts`
- Create: `pc-admin-nextjs/src/lib/auth.ts`
- Create: `pc-admin-nextjs/src/types/api.ts`

**Step 1: Initialize package**

Use these scripts:

```json
{
  "scripts": {
    "dev": "next dev -p 3000",
    "build": "next build",
    "lint": "next lint",
    "typecheck": "tsc --noEmit"
  }
}
```

Dependencies:

- `next`
- `react`
- `react-dom`
- `lucide-react`
- `class-variance-authority`
- `clsx`
- `tailwind-merge`
- `zod`
- `react-hook-form`
- `@hookform/resolvers`
- `@tanstack/react-table`
- `sonner`

**Step 2: Add shadcn components**

Add or generate:

- button
- input
- textarea
- select
- checkbox
- switch
- dialog
- sheet
- dropdown-menu
- tabs
- table
- badge
- avatar
- toast/sonner
- form
- card only for repeated items and dialogs where needed

**Step 3: Implement API client**

`src/lib/api.ts` must:

- read `NEXT_PUBLIC_API_BASE_URL`
- attach `Authorization`
- unwrap `{ code, msg, data }`
- throw on non-`200` codes
- support `download` and `FormData`

**Step 4: Verify**

Run:

```bash
cd pc-admin-nextjs
pnpm install
pnpm typecheck
pnpm build
```

Expected: Next.js builds.

**Step 5: Commit**

```bash
git add pc-admin-nextjs
git commit --only pc-admin-nextjs -m "feat: scaffold nextjs shadcn admin"
```

---

### Task 11: Implement Next.js Auth and Main Layout

**Files:**
- Create: `pc-admin-nextjs/app/(auth)/login/page.tsx`
- Create: `pc-admin-nextjs/app/(main)/layout.tsx`
- Create: `pc-admin-nextjs/app/(main)/dashboard/workplace/page.tsx`
- Create: `pc-admin-nextjs/src/components/auth/login-form.tsx`
- Create: `pc-admin-nextjs/src/components/layout/app-sidebar.tsx`
- Create: `pc-admin-nextjs/src/components/layout/header-bar.tsx`
- Create: `pc-admin-nextjs/src/components/layout/user-menu.tsx`
- Create: `pc-admin-nextjs/src/components/permission/permission-gate.tsx`
- Create: `pc-admin-nextjs/src/hooks/use-current-user.ts`
- Create: `pc-admin-nextjs/src/hooks/use-permission.ts`
- Create: `pc-admin-nextjs/src/types/auth.ts`

**Step 1: Implement login form**

Use shadcn form, input, button, and password field. Submit to `/auth/login`, save token, then load `/auth/user/info` and route to `/dashboard/workplace`.

**Step 2: Implement route/menu store**

Load `/auth/user/route` and render the sidebar recursively. Use lucide icons mapped from menu icon names where possible.

**Step 3: Implement permission gate**

```tsx
export function PermissionGate({ permissions, children }: Props) {
  const allowed = useHasAnyPermission(permissions)
  if (!allowed) return null
  return <>{children}</>
}
```

**Step 4: Implement layout**

Use a fixed sidebar and header. Keep the UI dense and operational, with no hero or marketing section.

**Step 5: Verify**

Run:

```bash
cd pc-admin-nextjs
pnpm typecheck
pnpm build
```

Expected: build passes and authenticated layout renders.

**Step 6: Commit**

```bash
git add pc-admin-nextjs
git commit --only pc-admin-nextjs -m "feat: implement nextjs auth layout"
```

---

### Task 12: Implement Next.js System Pages, Group 1

**Files:**
- Create: `pc-admin-nextjs/app/(main)/system/user/page.tsx`
- Create: `pc-admin-nextjs/app/(main)/system/role/page.tsx`
- Create: `pc-admin-nextjs/app/(main)/system/menu/page.tsx`
- Create: `pc-admin-nextjs/app/(main)/system/dept/page.tsx`
- Create: `pc-admin-nextjs/src/api/system/user.ts`
- Create: `pc-admin-nextjs/src/api/system/role.ts`
- Create: `pc-admin-nextjs/src/api/system/menu.ts`
- Create: `pc-admin-nextjs/src/api/system/dept.ts`
- Create: `pc-admin-nextjs/src/components/system/user-form.tsx`
- Create: `pc-admin-nextjs/src/components/system/role-form.tsx`
- Create: `pc-admin-nextjs/src/components/system/permission-tree.tsx`
- Create: `pc-admin-nextjs/src/components/system/menu-form.tsx`
- Create: `pc-admin-nextjs/src/components/system/dept-form.tsx`

**Step 1: Implement API wrappers**

Match Vue3 API paths exactly:

- `/system/user`
- `/system/role`
- `/system/menu`
- `/system/dept`

**Step 2: Build shared table utilities**

Create reusable table shell around TanStack Table with:

- toolbar filters
- refresh
- column actions
- pagination
- empty state
- loading state

**Step 3: Implement user page**

Include:

- department tree filter
- username/description/status filters
- table
- create/edit drawer
- detail drawer
- delete action
- reset password dialog
- assign roles dialog
- import/export action buttons

**Step 4: Implement role page**

Include:

- list/table
- create/edit drawer
- permission tree dialog
- data-scope selector
- assigned user tab/dialog

**Step 5: Implement menu and dept pages**

Use tree tables. Include create/edit/delete and status display.

**Step 6: Verify**

Run:

```bash
cd pc-admin-nextjs
pnpm typecheck
pnpm build
```

Expected: build passes.

**Step 7: Commit**

```bash
git add pc-admin-nextjs
git commit --only pc-admin-nextjs -m "feat: add nextjs core system pages"
```

---

### Task 13: Implement Next.js System Pages, Group 2

**Files:**
- Create: `pc-admin-nextjs/app/(main)/system/dict/page.tsx`
- Create: `pc-admin-nextjs/app/(main)/system/file/page.tsx`
- Create: `pc-admin-nextjs/app/(main)/system/config/page.tsx`
- Create: `pc-admin-nextjs/src/api/system/dict.ts`
- Create: `pc-admin-nextjs/src/api/system/file.ts`
- Create: `pc-admin-nextjs/src/api/system/option.ts`
- Create: `pc-admin-nextjs/src/api/system/storage.ts`
- Create: `pc-admin-nextjs/src/api/system/client.ts`
- Create: `pc-admin-nextjs/src/components/system/dict-form.tsx`
- Create: `pc-admin-nextjs/src/components/system/file-manager.tsx`
- Create: `pc-admin-nextjs/src/components/system/config-site.tsx`
- Create: `pc-admin-nextjs/src/components/system/config-security.tsx`
- Create: `pc-admin-nextjs/src/components/system/config-login.tsx`
- Create: `pc-admin-nextjs/src/components/system/config-storage.tsx`
- Create: `pc-admin-nextjs/src/components/system/config-client.tsx`

**Step 1: Implement dict page**

Mirror Vue3 workflow:

- left dict list/tree
- right dict item table
- create/edit/delete for dict and item
- clear cache action

**Step 2: Implement file manager**

Include:

- statistics panel
- list/grid view toggle
- upload
- create folder
- rename
- delete
- detail preview metadata

**Step 3: Implement config page**

Use shadcn Tabs:

- site
- security
- login
- storage
- client

Hide tabs by permission.

**Step 4: Verify**

Run:

```bash
cd pc-admin-nextjs
pnpm typecheck
pnpm build
```

Expected: build passes.

**Step 5: Commit**

```bash
git add pc-admin-nextjs
git commit --only pc-admin-nextjs -m "feat: add nextjs config dict file pages"
```

---

### Task 14: Implement Next.js Monitor and Profile Pages

**Files:**
- Create: `pc-admin-nextjs/app/(main)/monitor/online/page.tsx`
- Create: `pc-admin-nextjs/app/(main)/monitor/log/page.tsx`
- Create: `pc-admin-nextjs/app/(main)/user/profile/page.tsx`
- Create: `pc-admin-nextjs/src/api/monitor/online.ts`
- Create: `pc-admin-nextjs/src/api/monitor/log.ts`
- Create: `pc-admin-nextjs/src/api/user/profile.ts`
- Create: `pc-admin-nextjs/src/components/profile/basic-info.tsx`
- Create: `pc-admin-nextjs/src/components/profile/security.tsx`
- Create: `pc-admin-nextjs/src/components/profile/social.tsx`
- Create: `pc-admin-nextjs/src/components/monitor/log-detail.tsx`

**Step 1: Implement online page**

Include table, filters, and kickout action.

**Step 2: Implement log page**

Include tabs for operation/login logs if the backend type field supports it. Include detail sheet and export actions.

**Step 3: Implement profile page**

Include:

- avatar upload
- basic info form
- password update
- phone/email update
- social account compatibility list

**Step 4: Verify**

Run:

```bash
cd pc-admin-nextjs
pnpm typecheck
pnpm build
```

Expected: build passes.

**Step 5: Commit**

```bash
git add pc-admin-nextjs
git commit --only pc-admin-nextjs -m "feat: add nextjs monitor and profile pages"
```

---

### Task 15: Add End-to-End Verification

**Files:**
- Create: `pc-admin-nextjs/playwright.config.ts`
- Create: `pc-admin-nextjs/tests/e2e/login.spec.ts`
- Create: `pc-admin-nextjs/tests/e2e/system-pages.spec.ts`
- Create: `pc-admin-nextjs/tests/e2e/permissions.spec.ts`
- Create: `backend-rust/tests/api_smoke_tests.rs`
- Create: `backend-rust/tests/data_permission_api_tests.rs`
- Modify: `README-startup.md`
- Modify: `README.md`

**Step 1: Add backend smoke tests**

Cover:

- login
- user info
- route tree
- user page query
- role list
- menu tree
- dept tree
- option list

**Step 2: Add backend data-permission tests**

Create test users/roles/depts and assert SQL-level behavior for data scopes `1` through `5`.

**Step 3: Add Playwright tests**

Cover:

- login with admin/admin123
- sidebar renders system and monitor sections
- user page loads table
- role page opens permission dialog
- menu page loads tree table
- dept page loads tree table
- config page switches tabs
- file page renders manager
- log page opens detail

**Step 4: Update startup docs**

Add Rust backend and Next.js/shadcn commands:

```bash
cd backend-rust
DATABASE_URL=postgres://postgres:123456@127.0.0.1:5432/nv_admin DB_AUTO_MIGRATE=true cargo run

cd pc-admin-nextjs
NEXT_PUBLIC_API_BASE_URL=http://localhost:4398 pnpm dev
```

**Step 5: Run full backend verification**

Run:

```bash
cd backend-rust
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Expected: all pass.

**Step 6: Run full frontend verification**

Run:

```bash
cd pc-admin-nextjs
pnpm lint
pnpm typecheck
pnpm build
pnpm exec playwright test
```

Expected: all pass.

**Step 7: Commit**

```bash
git add backend-rust pc-admin-nextjs README.md README-startup.md
git commit --only backend-rust pc-admin-nextjs README.md README-startup.md -m "test: verify rust nextjs admin flows"
```

---

### Task 16: Final Integration Review

**Files:**
- Modify only files needed to fix review findings.

**Step 1: Run repository status**

Run:

```bash
git status --short
```

Expected: only intentional changes remain. Existing unrelated staged deletions may still appear if they predated this work.

**Step 2: Compare API surface**

Run:

```bash
rg -n "export function" pc-admin-vue3/src/apis
rg -n "route\\(\"|\\.route\\(\" backend-rust/src
```

Expected: every Vue3 API used by active pages has a Rust route or documented compatibility placeholder.

**Step 3: Run final verification**

Run:

```bash
cd backend-rust && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cd ../pc-admin-nextjs && pnpm lint && pnpm typecheck && pnpm build
```

Expected: all pass.

**Step 4: Request review**

Use `superpowers:requesting-code-review` before declaring the branch complete.

**Step 5: Commit final fixes**

```bash
git add backend-rust pc-admin-nextjs README.md README-startup.md docs/plans
git commit --only backend-rust pc-admin-nextjs README.md README-startup.md docs/plans -m "chore: finalize rust nextjs admin integration"
```

Only run this commit if there are final fixes after review.

