# Rust + Next.js Admin Design

Date: 2026-05-29

## Goal

Build a Rust + Next.js admin system that fully aligns with the existing Java + Vue3 admin project at the API, permission, database, and management-feature levels.

The system should become a reusable admin platform core for later ToC business development. Platform modules and future business domains must remain separated so C-side user identity and business rules do not leak into `sys_*` admin tables.

## Confirmed Decisions

- Backend: Rust with `axum`, `sqlx`, PostgreSQL, JWT, password hashing, and structured migrations.
- Frontend: Next.js App Router with shadcn/ui.
- API compatibility: Keep paths, response shapes, permission codes, menu route data, and table names compatible with the existing Java/Vue3 implementation where practical.
- Authorization: Do not introduce casbin-rs in the first version. Implement direct RBAC guards and SQL-level data-scope filtering aligned with the current ContiNew model.
- Architecture: Use lightweight DDD. Put real business rules in domain/application layers, but keep simple CRUD modules pragmatic.

## Backend Architecture

Use a modular Rust backend with clear boundaries:

```text
backend-rust/
  src/
    interfaces/http/
      auth/
      common/
      monitor/
      system/
      middleware/
    application/
      auth/
      rbac/
      data_scope/
      system/
      storage/
      monitor/
    domain/
      auth/
      rbac/
      data_scope/
      system/
      storage/
      toc_identity/
      toc_business/
    infrastructure/
      persistence/
      security/
      storage/
      cache/
    shared/
      config/
      error/
      id/
      pagination/
      response/
      time/
```

The `system`, `rbac`, and `data_scope` modules are platform capabilities. Future ToC modules should use separate bounded contexts such as `toc_identity` and `toc_business`, backed by `app_*` or business-prefixed tables instead of `sys_*`.

## Permission Model

Functional permissions use the existing RBAC tables:

- `sys_user`
- `sys_role`
- `sys_menu`
- `sys_user_role`
- `sys_role_menu`

Rust route handlers will declare required permission codes such as:

- `system:user:list`
- `system:role:updatePermission`
- `system:storage:setDefault`
- `monitor:log:export`

`PermissionGuard` validates JWT identity, loads user roles and permissions, treats the `admin` role as all permissions, and rejects unauthorized access at the backend. The Next.js frontend also hides inaccessible menus and buttons, but backend checks remain authoritative.

## Data Permission Model

Data permissions follow the existing role `data_scope` values:

- `1`: all data
- `2`: current department and child departments
- `3`: current department
- `4`: self-created data
- `5`: custom departments from `sys_role_dept`

Multiple roles are combined as a union. If any role grants all data, no data-scope filter is applied. Otherwise, `DataScopeResolver` resolves the visible department IDs and/or current user ID and produces a repository-level filter.

Repositories opt into data permission by providing the target columns:

```rust
DataPermissionTarget {
    dept_column: Some("dept_id"),
    user_column: Some("create_user"),
}
```

Filtering must be pushed down into PostgreSQL queries through `sqlx::QueryBuilder` parameter binding. The implementation must not fetch broad result sets and filter in memory, because that breaks pagination, exports, indexing, and statistics.

## Database Design

Use PostgreSQL with `sqlx migrate`. Do not rely on production startup auto-DDL. Development setup can run:

```bash
sqlx migrate run
cargo run
```

Core tables:

- `sys_user`
- `sys_role`
- `sys_menu`
- `sys_dept`
- `sys_user_role`
- `sys_role_menu`
- `sys_role_dept`
- `sys_dict`
- `sys_dict_item`
- `sys_option`
- `sys_storage`
- `sys_file`
- `sys_client`
- `sys_log`
- `sys_online_user` or Redis-backed online sessions

Seed data must include:

- admin user and default password
- root/default department
- admin and general roles
- all menus and button permissions used by the current Vue3 admin
- default dictionaries
- site, password, and login options
- default local storage
- default PC client

Future ToC tables must not reuse `sys_user`; use a separate C-side identity table such as `app_user` or `member`.

## Backend Feature Scope

Implement the API surface used by the current Vue3 frontend:

### Auth and Common

- `POST /auth/login`
- `POST /auth/logout`
- `GET /auth/user/info`
- `GET /auth/user/route`
- `GET /captcha/image`
- behavior captcha compatibility endpoints
- email captcha compatibility endpoint
- `GET /common/tree/dept`
- `GET /common/tree/menu`
- `GET /common/dict/user`
- `GET /common/dict/role`
- `GET /common/dict/{code}`
- `GET /common/dict/option/site`
- `POST /common/file`

### System

- User: page, list, detail, create, update, delete, export, import template, import parse, import, reset password, update roles
- Role: list, detail, create, update, delete, update menu permission, data scope, assigned users, assign users, unassign users, assigned user IDs
- Menu: tree, detail, create, update, delete, clear cache compatibility
- Dept: tree, detail, create, update, delete, export
- Dict and dict item: list/page, detail, create, update, delete, clear cache compatibility
- File: upload, list, update/rename, delete, statistics, hash check, create directory, directory size
- Option: list by category, update values, reset values
- Storage: list, detail, create, update, delete, update status, set default
- Client: page, detail, create, update, delete
- User profile: avatar, basic info, password, phone, email, social account compatibility

### Monitor

- Online user: page, kickout
- Log: page, detail, login export, operation export

### Reserved Placeholders

Keep route/module boundaries for these Vue3 directories, but do not make them first-wave complete CRUD unless the implementation plan explicitly expands scope:

- schedule job and schedule log
- system notice
- user message

## Frontend Architecture

Next.js uses App Router and shadcn/ui. The first screen is the login page when unauthenticated and the admin workspace when authenticated.

Recommended structure:

```text
pc-admin-nextjs/
  app/
    (auth)/login/
    (main)/dashboard/workplace/
    (main)/system/user/
    (main)/system/role/
    (main)/system/menu/
    (main)/system/dept/
    (main)/system/dict/
    (main)/system/file/
    (main)/system/config/
    (main)/monitor/online/
    (main)/monitor/log/
    (main)/user/profile/
  src/
    api/
    components/
      layout/
      permission/
      table/
      form/
      file/
    hooks/
    lib/
    stores/
    types/
```

Use shadcn/ui for buttons, inputs, dialogs, drawers/sheets, tabs, menus, dropdowns, forms, tables, toasts, and side navigation. Use TanStack Table for data tables and Zod/react-hook-form for form validation.

The UI should be an operational admin tool: dense but readable, restrained styling, predictable navigation, and no landing-page treatment.

## Frontend Feature Scope

Match Vue3 management workflows:

- Login and logout
- Permission-driven sidebar and buttons
- Dashboard workplace
- User management with department tree, filters, table, drawers/modals, role assignment, password reset, import/export entry points
- Role management with permission tree, data-scope fields, assigned-user management
- Menu management tree/table
- Department management tree/table
- Dict and dict-item management
- File manager with list/grid concepts, upload, rename, delete, directory operations, statistics
- Config page with tabs: site, security, login, storage, client
- Online user page
- Log page with login/operation detail and export actions
- User profile page

## Testing and Verification

Backend verification:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
sqlx migrate run
```

Add API smoke tests for:

- login
- user info
- route tree
- user, role, menu, and dept CRUD
- data-scope filtering

Frontend verification:

```bash
pnpm lint
pnpm typecheck
pnpm build
```

Add Playwright checks for:

- login
- main layout and sidebar
- user page
- role page
- menu page
- dept page
- config page
- file page
- monitor log page

Permission-specific acceptance:

- admin can see all data.
- a current-dept-and-child role can see only its department subtree.
- a current-dept role can see only its department.
- a self-only role can see only self-created rows.
- a custom-dept role can see only departments bound through `sys_role_dept`.
- unauthorized frontend buttons are hidden.
- unauthorized backend requests are rejected.

## Implementation Notes

Implement in vertical slices. Start with backend foundation, migrations, auth, RBAC, and data-scope tests before broad CRUD. Then add system modules and Next.js pages in feature groups. This reduces the risk of building UI over unstable contracts.
