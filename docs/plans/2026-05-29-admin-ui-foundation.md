# Admin UI Foundation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a polished shadcn/ui admin foundation with configurable appearance, a collapsible sidebar, login-04-inspired login, and dashboard/table visual upgrades.

**Architecture:** Add reusable theme primitives and shadcn sidebar components, then adapt the current layout and pages to those primitives. Keep all existing auth, route, permission, and API logic intact.

**Tech Stack:** Next.js 16, React 19, shadcn/ui, Radix UI primitives, Tailwind CSS, lucide-react, TanStack Table, Vitest.

---

### Task 1: Theme Configuration Core

**Files:**
- Create: `pc-admin-nextjs/src/lib/theme.ts`
- Test: `pc-admin-nextjs/src/lib/theme.test.ts`
- Modify: `pc-admin-nextjs/app/globals.css`

**Steps:**
1. Add tests for default theme, persisted theme parsing, and invalid value fallback.
2. Run `pnpm test src/lib/theme.test.ts` and confirm the new tests fail.
3. Implement typed theme options for style, primary color, neutral color, icon color, and radius.
4. Add CSS variable classes/data selectors for the supported options.
5. Re-run the focused test and then full `pnpm test`.

### Task 2: Theme Provider and Appearance Switcher

**Files:**
- Create: `pc-admin-nextjs/src/components/theme/theme-provider.tsx`
- Create: `pc-admin-nextjs/src/components/theme/appearance-switcher.tsx`
- Modify: `pc-admin-nextjs/app/layout.tsx`
- Modify: `pc-admin-nextjs/src/components/layout/header-bar.tsx`

**Steps:**
1. Add provider that loads localStorage settings and applies root classes/attributes.
2. Add a header dropdown using existing shadcn controls for style/color/radius choices.
3. Wire provider at the root layout and place the switcher in the admin header.
4. Verify persistence manually in the browser.

### Task 3: Sidebar-08 Admin Shell

**Files:**
- Add via shadcn or create locally: `pc-admin-nextjs/src/components/ui/sidebar.tsx`
- Add via shadcn or create locally: `pc-admin-nextjs/src/components/ui/collapsible.tsx`
- Add via shadcn or create locally: `pc-admin-nextjs/src/components/ui/separator.tsx`
- Add via shadcn or create locally: `pc-admin-nextjs/src/components/ui/breadcrumb.tsx`
- Modify: `pc-admin-nextjs/src/components/layout/admin-shell.tsx`
- Modify: `pc-admin-nextjs/src/components/layout/app-sidebar.tsx`
- Modify: `pc-admin-nextjs/src/components/layout/header-bar.tsx`
- Modify: `pc-admin-nextjs/src/components/layout/user-menu.tsx`

**Steps:**
1. Install the needed shadcn primitives.
2. Replace the custom sidebar layout with `SidebarProvider`, `SidebarInset`, and `SidebarTrigger`.
3. Convert menu rendering to `SidebarMenuButton`, nested collapsibles, active states, and tooltips.
4. Keep mobile behavior through shadcn sidebar instead of the old custom sheet wiring.
5. Verify collapsed and expanded sidebar states.

### Task 4: Login-04 Visual Refresh

**Files:**
- Modify: `pc-admin-nextjs/app/(auth)/login/page.tsx`
- Modify: `pc-admin-nextjs/src/components/auth/login-form.tsx`

**Steps:**
1. Rework page structure into the login-04 split card.
2. Keep username, password, captcha, validation, token storage, and route redirect unchanged.
3. Use lucide icons and a lightweight visual panel rather than external images.
4. Verify login with `admin/admin123` and `test1/123456`.

### Task 5: Dashboard and Table Visual Foundation

**Files:**
- Modify: `pc-admin-nextjs/src/components/table/data-table.tsx`
- Modify: `pc-admin-nextjs/app/(main)/dashboard/workplace/page.tsx`
- Optionally modify common page containers where table overflow needs cleanup.

**Steps:**
1. Make the shared table denser and closer to dashboard-01: rounded container, sticky header-ready structure, row hover, loading state, empty state.
2. Upgrade the workplace dashboard cards and action area using current user/route data.
3. Keep existing page APIs unchanged.

### Task 6: Verification and Commit

**Files:**
- All changed frontend files.

**Steps:**
1. Run `pnpm test`.
2. Run `pnpm typecheck`.
3. Run `pnpm lint`.
4. Run `pnpm build`.
5. Use browser checks/screenshots for login, dashboard, user list, collapsed sidebar, and theme settings.
6. Commit the completed UI foundation.
