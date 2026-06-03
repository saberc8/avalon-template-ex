# Rust + Next.js Scheduler Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build scheduled-job management and execution for the Rust backend and Next.js admin frontend.

**Architecture:** PostgreSQL stores job definitions, trigger rows, and execution logs. RabbitMQ transports due trigger messages to workers for HTTP or built-in execution. The API, scheduler loop, and worker loop share application services but can run embedded locally or as independent production processes.

**Tech Stack:** Rust, axum, sqlx, PostgreSQL, RabbitMQ, lapin, cron, reqwest, Next.js App Router, React, shadcn/ui, TanStack Table, Vitest.

---

### Task 1: Scheduler Core Tests

**Files:**
- Create: `backend-rust/src/application/scheduler/cron.rs`
- Create: `backend-rust/src/application/scheduler/http_safety.rs`
- Modify: `backend-rust/src/application/mod.rs`

**Steps:**
1. Write tests for second-level cron validation and next fire time calculation.
2. Run `cd backend-rust && cargo test application::scheduler::cron`.
3. Implement the cron helper using the `cron` crate.
4. Write tests for HTTP allowlist modes.
5. Run `cd backend-rust && cargo test application::scheduler::http_safety`.
6. Implement URL safety validation using parsed URLs.

### Task 2: Database Migration And Repository

**Files:**
- Create: `backend-rust/migrations/202605300001_create_scheduler.sql`
- Create: `backend-rust/src/infrastructure/persistence/scheduler_repository.rs`
- Modify: `backend-rust/src/infrastructure/persistence/mod.rs`

**Steps:**
1. Add migration for `sys_job`, `sys_job_trigger`, `sys_job_log`, menu seed, and admin role menu bindings.
2. Write repository tests for pure query helpers and status transitions where database-free helpers are available.
3. Implement repository methods for job CRUD, due job claiming, pending trigger listing, trigger status updates, and log inserts.
4. Run `cd backend-rust && cargo fmt`.

### Task 3: Scheduler Application Service

**Files:**
- Create: `backend-rust/src/application/scheduler/mod.rs`
- Create: `backend-rust/src/application/scheduler/service.rs`
- Modify: `backend-rust/src/application/mod.rs`

**Steps:**
1. Write tests for command validation: required cron, HTTP URL, built-in key, retry and timeout ranges.
2. Implement DTOs, normalization, and validation.
3. Implement CRUD, status update, manual trigger, due trigger creation, and pending trigger publication preparation.
4. Run focused Rust tests.

### Task 4: RabbitMQ Infrastructure And Worker

**Files:**
- Create: `backend-rust/src/infrastructure/mq/mod.rs`
- Create: `backend-rust/src/infrastructure/mq/rabbitmq.rs`
- Create: `backend-rust/src/application/scheduler/executor.rs`
- Create: `backend-rust/src/bin/scheduler_worker.rs`
- Modify: `backend-rust/src/shared/config.rs`
- Modify: `backend-rust/src/main.rs`

**Steps:**
1. Add configuration parsing tests for scheduler and RabbitMQ settings.
2. Implement RabbitMQ publisher/consumer setup, exchange/queue declaration, retry, and dead-letter publishing.
3. Implement HTTP and built-in executors.
4. Add embedded startup toggles in `main.rs` and independent worker entrypoint.
5. Run `cd backend-rust && cargo test`.

### Task 5: HTTP API

**Files:**
- Create: `backend-rust/src/interfaces/http/scheduler.rs`
- Modify: `backend-rust/src/interfaces/http/mod.rs`

**Steps:**
1. Add route tests for permission-protected API envelopes where existing router tests allow.
2. Implement routes:
   - `GET /schedule/job/page`
   - `GET /schedule/job/:id`
   - `POST /schedule/job`
   - `PUT /schedule/job/:id`
   - `DELETE /schedule/job`
   - `PATCH /schedule/job/:id/status`
   - `POST /schedule/job/:id/run`
   - `GET /schedule/job/:id/log`
3. Run backend tests.

### Task 6: Next.js API And UI

**Files:**
- Create: `pc-admin-nextjs/src/types/schedule.ts`
- Create: `pc-admin-nextjs/src/api/schedule/job.ts`
- Create: `pc-admin-nextjs/src/components/schedule/job-form.tsx`
- Create: `pc-admin-nextjs/app/(main)/schedule/job/page.tsx`
- Modify: `pc-admin-nextjs/src/components/layout/app-sidebar.tsx`

**Steps:**
1. Write API path tests for schedule endpoints.
2. Implement TypeScript types and API helpers.
3. Build a dense admin page with filters, table actions, status controls, manual run, and logs.
4. Add a form dialog for HTTP and built-in jobs.
5. Add sidebar icon support for the seeded menu.
6. Run `cd pc-admin-nextjs && pnpm typecheck`.

### Task 7: Verification

**Files:**
- All changed files.

**Steps:**
1. Run `cd backend-rust && cargo fmt --check`.
2. Run `cd backend-rust && cargo clippy --all-targets -- -D warnings`.
3. Run `cd backend-rust && cargo test`.
4. Run `cd pc-admin-nextjs && pnpm lint`.
5. Run `cd pc-admin-nextjs && pnpm typecheck`.
6. Run `cd pc-admin-nextjs && pnpm test`.
