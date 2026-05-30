# System Admin UX Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Improve the system dictionary, file, and config pages, and make every system table action show visible button text next to its icon.

**Architecture:** Keep the existing Next.js App Router and shadcn-style component patterns. Add a small table action button helper, then update the system pages to use it consistently; target poor interactions by replacing browser prompts/confirms with in-app dialogs where the workflow is most visible.

**Tech Stack:** Next.js, React, TypeScript, TanStack Table, lucide-react, Vitest, Testing Library.

---

### Task 1: Table Action Button Helper

**Files:**
- Create: `pc-admin-nextjs/src/components/table/table-actions.test.tsx`
- Create: `pc-admin-nextjs/src/components/table/table-actions.tsx`

**Steps:**
1. Write a failing test asserting a table action renders a button with visible label text.
2. Run `pnpm test src/components/table/table-actions.test.tsx` and confirm it fails because the helper does not exist.
3. Implement `TableActions` and `TableActionButton` as compact `Button size="sm"` wrappers.
4. Re-run the focused test.

### Task 2: Replace System Table Icon-Only Actions

**Files:**
- Modify: `pc-admin-nextjs/app/(main)/system/user/page.tsx`
- Modify: `pc-admin-nextjs/app/(main)/system/role/page.tsx`
- Modify: `pc-admin-nextjs/app/(main)/system/dept/page.tsx`
- Modify: `pc-admin-nextjs/app/(main)/system/menu/page.tsx`
- Modify: `pc-admin-nextjs/app/(main)/system/dict/page.tsx`
- Modify: `pc-admin-nextjs/src/components/system/file-manager.tsx`
- Modify: `pc-admin-nextjs/src/components/system/config-storage.tsx`
- Modify: `pc-admin-nextjs/src/components/system/config-client.tsx`

**Steps:**
1. Replace each action column `Button size="icon"` with `TableActionButton`.
2. Keep existing permissions and disabled states.
3. Use destructive styling for delete actions.
4. Run `rg 'size="icon"' pc-admin-nextjs/app/'(main)'/system pc-admin-nextjs/src/components/system` to verify no system table action remains icon-only.

### Task 3: Improve Target Page Interactions

**Files:**
- Modify: `pc-admin-nextjs/app/(main)/system/dict/page.tsx`
- Modify: `pc-admin-nextjs/src/components/system/file-manager.tsx`
- Modify: `pc-admin-nextjs/src/components/system/config-option-panel.tsx`
- Modify: `pc-admin-nextjs/src/components/system/config-storage.tsx`
- Modify: `pc-admin-nextjs/src/components/system/config-client.tsx`

**Steps:**
1. Add focused tests for file rename dialog behavior.
2. Replace file manager prompt/confirm flows with dialogs for rename, folder creation, and delete confirmation.
3. Add clearer empty/loading states, placeholders, and status summaries to dictionary and file pages.
4. Make config pages show category titles, changed state, and safer reset/save controls.
5. Run focused tests, typecheck, and lint.
