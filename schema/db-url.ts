import dotenv from 'dotenv';

dotenv.config({ path: '../.env.local', quiet: true });

/**
 * Target database for drizzle-kit and the schema scripts:
 * `DB_MODE=PROD` → `PG_DATABASE_URL1` (production), otherwise `PG_DATABASE_URL` (local).
 * Prints the target (host/db only) to stderr so a prod run is never silent.
 */
export function dbUrl(): string {
  const prod = process.env.DB_MODE === 'PROD';
  const name = prod ? 'PG_DATABASE_URL1' : 'PG_DATABASE_URL';
  const url = process.env[name];
  if (!url) throw new Error(`${name} is not set (../.env.local)`);
  const u = new URL(url);
  console.error(`[db] ${prod ? 'PROD' : 'local'} → ${u.host}${u.pathname} (${name})`);
  return forNodeDriver(u);
}

/**
 * postgres.js sends unknown URL query params to the server as runtime
 * settings, so libpq-only options must go. `sslrootcert=system` (libpq ≥ 16:
 * "use the OS CA store") is dropped; `sslmode=verify-full` stays, and Node
 * verifies against its bundled CA roots.
 */
function forNodeDriver(u: URL): string {
  if (u.searchParams.get('sslrootcert') === 'system') u.searchParams.delete('sslrootcert');
  return u.toString();
}
