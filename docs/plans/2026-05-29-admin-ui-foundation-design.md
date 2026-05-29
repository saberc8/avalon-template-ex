# Admin UI Foundation Design

## Goal

Upgrade the Next.js admin frontend to a shadcn/ui-based product shell that feels consistent with the referenced shadcn presets and blocks while preserving the existing backend APIs and business pages.

## Scope

- Replace the custom fixed sidebar shell with a shadcn `SidebarProvider`/`SidebarInset` layout inspired by `sidebar-08`.
- Keep lucide-react as the only icon library and map backend menu icon names to lucide icons.
- Add a user-facing appearance switcher for style, primary color, neutral color, icon color, and radius.
- Rework login to the `login-04` split-card pattern while keeping the existing username/password/captcha login flow.
- Upgrade the shared table surface and dashboard page toward the denser `dashboard-01` style.
- Preserve current routes, permissions, API wrappers, and system pages.

## Architecture

Theme state will live in a small client-side provider under `src/components/theme`. It writes selected values to `localStorage`, applies classes/data attributes to `document.documentElement`, and exposes the current settings to a header dropdown. CSS variables in `app/globals.css` define the actual palettes and radius tokens.

The admin shell will reuse current `useCurrentUser()` data but swap structural primitives to shadcn sidebar components. Business pages continue to render inside the same `(main)` layout and should not need API changes.

## Decisions

- Use official registry code as reference, not direct block replacement, because direct replacement would introduce example data, English copy, and unused dependencies.
- Keep the existing auth behavior and default credentials.
- Keep the initial admin protection logic from backend work unchanged; this design only affects frontend UI.

## Verification

- `pnpm test`
- `pnpm typecheck`
- `pnpm lint`
- `pnpm build`
- Browser verification for login, dashboard, user list, collapsed sidebar, mobile sidebar, and appearance settings persistence.
