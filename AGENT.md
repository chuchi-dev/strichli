# Agent Instructions

Instructions for anyone (human or coding agent) changing this repository.

- Keep it simple. Build the smallest version that solves the actual
  request, follow nearby patterns, no abstractions for theoretical cases.
- Privacy is the product: for visitors no cookies, nothing stored in
  their browser, no ip address, user agent or visitor id in the database
  (the ip is only used for rate limits). Anything new that
  is stored about visitors needs a reason in the README.
- `server/` is axum with chuchi-postgres (raw SQL, `FromRow`). Each area
  (`events`, `sites`, `users`) has `mod.rs` (data), `routes.rs` (api)
  and its `migrations`, registered in the area's `MIGRATIONS` list; they
  run at startup. Never change a migration that is released, add one.
- Every api route checks the session or token and, for a site, that the
  user may see it (admins see all sites, others only their own); writes
  need an admin, and an api token only writes with "can write". Public on
  purpose, guarded by rate limits instead: login, password links,
  `/api/version` and the tracking endpoints (`/e`, `/noscript.gif`,
  `/s.js`, `/latest.js`).
- `webui/` is Svelte 5 with the chuchi router (`lib/router.svelte.ts`,
  `session.svelte.ts`, `api.ts` with `ApiError` and `newest`); api calls
  go in the area's module in `src/api/`.
- Rust is formatted with `cargo fmt` (hard tabs, max width 80), the webui
  with prettier (tabs, single quotes).
- Tests are small, offline unit tests next to the code
  (`#[cfg(test)] mod tests`).
- Commits: a short imperative subject, then a body that says what changed
  and why.
- The README documents the tracking script, the api, the config and what
  is stored; update it with the change.

## Changes and checks

- Nothing goes to `main` directly: every change is a branch and a pull
  request, reviewed (code and security) before it is squash merged.
  Deploys are separate and start from `main`.
- The full checks: in `server/` `cargo fmt --check`, `cargo clippy
  --all-targets -- -D warnings`, `cargo test`; in `webui/` `npm ci`,
  `npx prettier --check src`, `npm run lint`, `npm run check`, `npm run
  build`.
