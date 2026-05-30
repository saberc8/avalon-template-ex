# Dynamic Login Captcha Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the login form use the backend captcha image API instead of hardcoded local captcha values.

**Architecture:** Add a typed auth API wrapper for `GET /captcha/image`. The login form fetches captcha data on mount, displays the returned image, stores the returned UUID, submits that UUID with login, and refreshes captcha data on image click or login failure.

**Tech Stack:** Next.js client components, React Hook Form, Vitest, Testing Library, shared fetch API client.

---

### Task 1: Auth API Captcha Wrapper

**Files:**
- Modify: `pc-admin-nextjs/src/types/auth.ts`
- Modify: `pc-admin-nextjs/src/api/auth.ts`
- Test: `pc-admin-nextjs/src/api/auth.test.ts`

**Step 1: Write the failing test**

Create an auth API test that calls `getImageCaptcha()` and expects fetch to request `http://localhost:4398/captcha/image`.

**Step 2: Run the focused test**

Run: `pnpm --dir pc-admin-nextjs test src/api/auth.test.ts`

Expected: FAIL because `getImageCaptcha` does not exist yet.

**Step 3: Implement minimal wrapper and type**

Add `ImageCaptchaResponse` and `getImageCaptcha()`.

**Step 4: Run the focused test**

Run: `pnpm --dir pc-admin-nextjs test src/api/auth.test.ts`

Expected: PASS.

### Task 2: Login Form Dynamic Captcha

**Files:**
- Modify: `pc-admin-nextjs/src/components/auth/login-form.tsx`
- Test: `pc-admin-nextjs/src/components/auth/login-form.test.tsx`

**Step 1: Write the failing tests**

Mock the auth API and assert that the login form:

- Calls `getImageCaptcha()` on mount.
- Renders the captcha image from the returned `img`.
- Submits the returned `uuid` to `accountLogin()` instead of `"local"`.
- Refreshes captcha after a failed login.

**Step 2: Run the focused test**

Run: `pnpm --dir pc-admin-nextjs test src/components/auth/login-form.test.tsx`

Expected: FAIL because the form does not fetch captcha data yet.

**Step 3: Implement form behavior**

Use local state for captcha metadata, fetch on mount, show image button, submit dynamic UUID, and refresh after failure.

**Step 4: Run the focused test**

Run: `pnpm --dir pc-admin-nextjs test src/components/auth/login-form.test.tsx`

Expected: PASS.

### Task 3: Verification

**Files:**
- Read: `pc-admin-nextjs/src/components/auth/login-form.tsx`
- Read: `pc-admin-nextjs/src/api/auth.ts`

**Step 1: Run frontend verification**

Run:

```bash
pnpm --dir pc-admin-nextjs test
pnpm --dir pc-admin-nextjs typecheck
pnpm --dir pc-admin-nextjs lint
```

Expected: all commands pass.

**Step 2: Static search**

Run:

```bash
rg -n 'captcha: "local"|uuid: .*"local"|captcha.*local' pc-admin-nextjs/src pc-admin-nextjs/app
```

Expected: no login captcha hardcoded local fallback remains.
