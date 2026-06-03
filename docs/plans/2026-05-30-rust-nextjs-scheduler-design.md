# Rust + Next.js Scheduler Design

Date: 2026-05-30

## Goal

Add a complete scheduled-job capability to the Rust + Next.js admin stack. The feature must support second-level cron expressions, HTTP callback jobs, future built-in Rust jobs, RabbitMQ-based execution, PostgreSQL-backed management and audit records, and both embedded local mode and independent production worker mode.

## Confirmed Decisions

- Use RabbitMQ plus PostgreSQL.
- PostgreSQL is the source of truth for job definitions, trigger instances, execution logs, and audit data.
- RabbitMQ is the execution transport for due triggers, retries, and dead-letter handling.
- Support both HTTP callback jobs and a built-in Rust job registry.
- Support second-level cron expressions.
- Allow local development to run scheduler and worker inside the API process.
- Prefer independent Scheduler/Worker processes in production.
- Protect HTTP jobs with an allowlist by default, with environment switches for strict or open modes.

## Architecture

The Rust backend adds a `scheduler` bounded context:

- Schedule API: CRUD, enable/disable, manual run, log query, and permission checks.
- Scheduler loop: scans enabled jobs, computes next fire time, creates durable trigger rows, and publishes pending triggers to RabbitMQ.
- Worker loop: consumes RabbitMQ messages, executes HTTP or built-in tasks, writes logs, and sends failed attempts to retry or dead-letter flow.
- Built-in registry: maps stable task keys to Rust handlers. The first implementation can ship with a health-check/noop task and leave extension points for future business jobs.

PostgreSQL stores all durable state. RabbitMQ stores only transport state. If RabbitMQ is temporarily down, pending trigger rows remain visible and can be published after recovery.

## Database

Add a migration with:

- `sys_job`: task definition.
- `sys_job_trigger`: each scheduled or manual trigger instance.
- `sys_job_log`: each execution attempt.

`sys_job` fields include name, group, task type, cron expression, status, HTTP method/url/headers/body, built-in task key, retry limit, timeout, concurrency switch, misfire policy, last fire time, next fire time, description, and create/update metadata.

`sys_job_trigger` fields include job id, source, scheduled fire time, status, attempt count, max attempts, payload snapshot, queued/running/finished timestamps, error message, and trace id.

`sys_job_log` fields include trigger id, job id, attempt, status, executor, request snapshot, response status/body excerpt, error message, started/finished timestamps, and duration.

## RabbitMQ

Use a direct exchange and three queues:

- Exchange: `avalon.scheduler`
- Execute queue: `avalon.scheduler.execute`, routing key `scheduler.execute`
- Retry queue: `avalon.scheduler.retry`, dead-letters back to the execute route after TTL
- Dead queue: `avalon.scheduler.dead`, routing key `scheduler.dead`

Message payload:

```json
{
  "triggerId": 1780000000000,
  "jobId": 1780000000001,
  "taskType": "HTTP",
  "attempt": 1,
  "maxAttempts": 3
}
```

The database remains authoritative. Workers load the latest trigger and job snapshots before execution and update status transactionally around each attempt.

## HTTP Job Safety

HTTP jobs validate URL scheme and host before save and before execution. The default mode allows local development hosts and hosts configured by `SCHEDULER_HTTP_ALLOWLIST`. Production can use strict allowlist mode. Open mode exists for development only.

Blocked URLs fail fast with a clear error and no outbound request.

## Rust Modules

Add:

```text
backend-rust/src/application/scheduler/
backend-rust/src/infrastructure/persistence/scheduler_repository.rs
backend-rust/src/infrastructure/mq/
backend-rust/src/interfaces/http/scheduler.rs
backend-rust/src/bin/scheduler_worker.rs
```

Configuration additions include RabbitMQ URL, scheduler toggles, tick interval, queue names, HTTP allowlist mode, and worker identity.

## Next.js Management

Add:

```text
pc-admin-nextjs/app/(main)/schedule/job/page.tsx
pc-admin-nextjs/src/api/schedule/job.ts
pc-admin-nextjs/src/types/schedule.ts
pc-admin-nextjs/src/components/schedule/job-form.tsx
```

The page should provide filters, a job table, status badges, create/edit dialogs, enable/disable, manual run, delete, and a log panel.

## Permissions

Seed a menu under system monitoring or a new schedule section:

- `schedule:job:list`
- `schedule:job:get`
- `schedule:job:create`
- `schedule:job:update`
- `schedule:job:delete`
- `schedule:job:updateStatus`
- `schedule:job:run`
- `schedule:job:log:list`

Admin receives all permissions through existing role-menu seed behavior.

## Verification

Backend:

```bash
cd backend-rust
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Frontend:

```bash
cd pc-admin-nextjs
pnpm lint
pnpm typecheck
pnpm test
```

The first implementation should test cron calculation, URL safety, command validation, repository SQL shape where practical, API route envelopes, and Next API path generation.
