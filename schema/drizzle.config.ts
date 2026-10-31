import type { Config } from 'drizzle-kit';
import { dbUrl } from './db-url';

// Local by default; `DB_MODE=PROD` targets production (see db-url.ts).
export default {
  schema: './schema.ts',
  out: './migrations',
  dialect: 'postgresql',
  // snake_case columns (matches the SeaORM entities in ../src/entities).
  casing: 'snake_case',
  dbCredentials: {
    url: dbUrl(),
  },
} satisfies Config;
