# Next.js 4399 Rust CORS Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the Next.js admin dev server run on `4399` and let the Rust backend accept that origin.

**Architecture:** Keep the frontend API base URL pointed at the Rust backend on `4398`. Change only the frontend dev server port and Rust CORS allowed origins for local development.

**Tech Stack:** Next.js package scripts, Rust Axum/tower-http CORS, Cargo tests, README docs.

---

### Task 1: Rust Default CORS

**Files:**
- Modify: `backend-rust/src/shared/config.rs`
- Modify: `backend-rust/src/interfaces/http/mod.rs`

**Step 1: Write the failing test**

Add a Rust config test that parses `DEFAULT_CORS_ALLOWED_ORIGINS` and expects `http://localhost:4399` and `http://127.0.0.1:4399`.

**Step 2: Run test to verify it fails**

Run: `cargo test default_cors_allowed_origins_include_nextjs_dev_port --manifest-path backend-rust/Cargo.toml`

Expected: FAIL because the default still uses `3000`.

**Step 3: Write minimal implementation**

Change the default CORS origins and CORS tests from the Next.js `3000` origin to `4399`.

**Step 4: Run tests**

Run: `cargo test --manifest-path backend-rust/Cargo.toml`

Expected: PASS.

### Task 2: Frontend Port And Docs

**Files:**
- Modify: `pc-admin-nextjs/package.json`
- Modify: `backend-rust/.env.example`
- Modify: `backend-rust/.env`
- Modify: `README.md`
- Modify: `README-startup.md`

**Step 1: Update configuration**

Change `next dev -p 3000` to `next dev -p 4399`. Add the Rust CORS local origins for `4399`. Update docs that describe the Next.js default URL.

**Step 2: Verify static configuration**

Run:

```bash
rg -n "next dev -p 4399|localhost:4399|127\\.0\\.0\\.1:4399" pc-admin-nextjs/package.json backend-rust/.env backend-rust/.env.example README.md README-startup.md backend-rust/src
```

Expected: relevant files show `4399` and no active Next.js `3000` default remains.
