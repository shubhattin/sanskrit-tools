# Stack

- Rust (axum + sqlx) JSON API at the repo root; data importer in the same crate.
- Drizzle schema (source of truth for the DB) in `schema/`; Postgres database `sans_dict`.
- TanStack Start (React 19, React Compiler, shadcn base-nova, Tailwind 4) SPA in `frontend/`.
- Raw data: `dict_data/` (StarDict + TSV, git-ignored). Analysis of every source: `docs/data.md`.

# Structure

- `src/translit.rs` – SLP1 / loose search keys (vidyut-lipi, lipilekhika "Normal" input).
- `src/import/` – `sources.rs` (registry + per-work parsers), `markup.rs` (source text → sanitized HTML), `stardict.rs`, `mod.rs` (COPY loader).
- `src/routes/` – API handlers; contract in `docs/api.md` (keep it in sync with the frontend types in `frontend/src/lib/api/`).
- `schema/schema.ts` – tables; migrations in `schema/migrations/` (snake_case columns).

# Commands

- `bun run db:generate` / `bun run db:migrate` – drizzle migrations (`schema/`).
- `bun run import [-- --force] [slug ...]` – (re)import works; unchanged sources are skipped by hash. Bump `IMPORTER_VERSION` in `src/import/mod.rs` when parsing changes.
- `bun run dev:api` (axum on :5780), `bun run dev` (Vite on :3010, proxies `/api`).
- `cargo test --lib`, `cargo fmt`, `bun run check`, `bun run lint`, `bun run format`.

# Conventions

- Package manager: bun (`bunx`, never npx).
- Search keys are always produced by `src/translit.rs`; never compare Devanagari strings in SQL for lookup.
- Entry HTML is built by escaping source text first and adding only the tags listed in `docs/api.md`; HTML sources go through `markup::sanitize_html`.
- Lipi Lekhika typing on inputs: default off, Alt+X / Alt+C toggles.
- Never run git commit on your own.
