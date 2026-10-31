/**
 * `drizzle-kit push` only diffs tables; it does not run the custom SQL in
 * migrations/0000_extensions.sql. Install the extensions the schema needs
 * (pg_trgm for the `gin_trgm_ops` index) before pushing.
 */
import postgres from 'postgres';
import { dbUrl } from './db-url';

const sql = postgres(dbUrl(), { max: 1, onnotice: () => {} });
try {
  await sql`CREATE EXTENSION IF NOT EXISTS pg_trgm`;
  console.error('[db] extension pg_trgm ok');
} finally {
  await sql.end();
}
