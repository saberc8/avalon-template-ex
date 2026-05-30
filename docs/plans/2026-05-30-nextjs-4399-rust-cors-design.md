# Next.js 4399 Rust CORS Design

## Goal

Run the Next.js admin frontend on port `4399` while keeping the Rust backend on `4398`.

## Approach

Update `pc-admin-nextjs` so `pnpm dev` starts Next.js on `http://localhost:4399`. Update the Rust backend CORS defaults and local environment example so requests from `http://localhost:4399` and `http://127.0.0.1:4399` are allowed by default.

## Scope

- Change the Next.js dev script from `3000` to `4399`.
- Change Rust default CORS origins from the old Next.js `3000` origin to `4399`.
- Update Rust tests that assert allowed CORS origins.
- Update local `.env.example`, current local `.env`, and README startup text.

## Non-Goals

- Do not change the Rust backend HTTP port; it remains `4398`.
- Do not change API base URL behavior; the Next.js frontend still defaults to `http://localhost:4398`.
- Do not touch other backend implementations.

## Testing

Run Rust tests that cover config and CORS behavior, then run Next.js package metadata checks with a focused grep.
