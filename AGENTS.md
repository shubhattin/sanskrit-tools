# Stack

- Rust JSON API at the repo root (axum + SeaORM 2, entities in `src/entities/`); data importer in the same crate (bulk COPY via sqlx 0.9, the same sqlx SeaORM runs on).
- Drizzle schema (source of truth for the DB) in `schema/`; Postgres database `sans_dict`.
- TanStack Start (React 19, React Compiler, shadcn base-nova, Tailwind 4) SPA in `frontend/`.
- Raw data: `dict_data/` (StarDict + TSV, git-ignored). Analysis of every source: `docs/data.md`.

# Structure

- `src/translit.rs` – SLP1 / loose search keys (vidyut-lipi, lipilekhika "Normal" input).
- `src/import/` – `sources.rs` (registry + per-work parsers), `paradigms.rs` (Heritage/JNU inflection tables), `markup.rs` (source text → sanitized HTML), `stardict.rs`, `mod.rs` (COPY loader).
- `src/entities/` – SeaORM entities mirroring `schema/schema.ts` (drizzle stays the only migration system; update both together).
- `src/auth.rs` – Better Auth session-cookie gate for the agent API (same pattern as
  `tsc-links`): forwards `Cookie` to `${VITE_BETTER_AUTH_URL}/api/auth/get-session`, requires the
  admin role, blocks cross-site writes.
- `src/agent/` – dictionary agent (`/agent`): aisdk (git rev) tool loop over our OpenRouter Responses-API model (`openrouter.rs`: reasoning-item replay, top-level `cache_control` for Anthropic, cost), dictionary tools calling the API handlers (`tools.rs`), web search through OpenAI's native `web_search` tool (`web_search.rs`), AI SDK UI stream encoder (`ui_stream.rs`), storage (`store.rs`). Routes in `src/routes/agent.rs` (outside OpenAPI, behind `require_admin`); contract `docs/agent.md` (keep in sync with `frontend/src/lib/agent/`).
- `src/routes/` – API handlers (entity queries; Postgres-specific SQL as typed raw `Statement`s + `FromQueryResult`); contract in `docs/api.md` (keep it in sync with the frontend types in `frontend/src/lib/api/`).
- `schema/schema.ts` – tables; migrations in `schema/migrations/` (snake_case columns).

# Commands

- `bun run db:generate` / `bun run db:migrate` – drizzle migrations (`schema/`). Prefix `DB_MODE=PROD` to target production (`PG_DATABASE_URL1`; see `schema/db-url.ts`).
- `bun run db:push` exists (installs pg_trgm first) but **use `migrate` for production**: drizzle-kit cannot introspect `text COLLATE "C"`, so every push re-issues `ALTER COLUMN … TYPE` on 7 columns and rebuilds the `paradigm_forms` primary key (table rewrites + locks on a populated DB).
- Production roles (PlanetScale): migrations / seeding / `pg_restore` need a role inheriting `postgres` (DDL, CREATE EXTENSION, `drizzle` schema); the API only needs `pg_read_all_data`; `pg_dump` works with `pg_read_all_data`.
- `bun run import [-- --force] [slug ...]` – (re)import works; unchanged sources are skipped by hash. Bump `IMPORTER_VERSION` in `src/import/mod.rs` when parsing changes.
- `bun run seed` – reproducible full seed of `PG_DATABASE_URL` (migrations + import + count check against `scripts/expected-counts.tsv`); `--url <db>` for another database, `--dump <file>` to restore a `bun run db:dump` file instead (no `dict_data` needed). After an intentional data change: `bash scripts/seed.sh --write-expected`.
- `bun run dev:api` (axum on :5780), `bun run dev` (Vite on :3010, proxies `/api`).
- `cargo test --lib`, `cargo fmt`, `bun run check`, `bun run lint`, `bun run format`.

# Deploy (Vercel, mirrors ../tsc-links)

- `vercel.json`: `bun run --cwd frontend build` → static SPA in `frontend/.output/public` (nitro preset pinned to `node-server`; only `public/` is deployed). `/api/*` → Rust function `api/vercel.rs` (`vercel_runtime`); every other path → `/_shell.html` (files win first).
- Set `PG_DATABASE_URL`, `OPENROUTER_API_KEY` and `OPENAI_API_KEY` (agent chat models + web search) in the Vercel project. Prod already has its schema: load data with `bun run db:dump && DB_MODE=PROD bun run seed -- --dump db/sans_dict.dump --data-only` (TRUNCATE + data in FK order + drizzle log + count check), using a role inheriting `postgres`. A plain `--dump` (with `--clean`) only suits an empty database whose extensions the role owns.
- Set `VITE_BETTER_AUTH_URL` (Better Auth server) in the Vercel project env too: it gates `/api/agent/*` to admin users, and the frontend auth client talks to it directly. Same value goes in `frontend/.env.local` for local dev.
- `sslrootcert=system` in connection strings (PlanetScale) is libpq-only: `schema/db-url.ts` and `src/db_url.rs` drop it for postgres.js / sqlx (still `sslmode=verify-full`); psql/pg_dump use the URL as is.
- Self-host instead: `bun run build && bun run start` (axum serves SPA + API on :5780).

# Conventions

- Package manager: bun (`bunx`, never npx).
- Search keys are always produced by `src/translit.rs`; never compare Devanagari strings in SQL for lookup.
- Entry HTML is built by escaping source text first and adding only the tags listed in `docs/api.md`; HTML sources go through `markup::sanitize_html`.
- Lipi Lekhika typing on inputs: default off, Alt+X / Alt+C toggles.
- Never run git commit on your own.
