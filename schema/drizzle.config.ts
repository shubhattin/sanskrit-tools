import type { Config } from 'drizzle-kit';
import dotenv from 'dotenv';

dotenv.config({ path: '../.env.local' });

export default {
  schema: './schema.ts',
  out: './migrations',
  dialect: 'postgresql',
  // snake_case columns: the Rust side uses hand-written SQL.
  casing: 'snake_case',
  dbCredentials: {
    url: process.env.PG_DATABASE_URL!,
  },
} satisfies Config;
