import { sql } from 'drizzle-orm';
import {
  customType,
  index,
  integer,
  jsonb,
  pgEnum,
  pgTable,
  primaryKey,
  smallint,
  text,
  timestamp,
  uniqueIndex,
  varchar,
} from 'drizzle-orm/pg-core';

/**
 * Sanskrit dictionary schema (database `sans_dict`).
 *
 * Search keys are SLP1 (one ASCII char per phoneme) stored with `COLLATE "C"`
 * so a single b-tree serves exact match and `LIKE 'prefix%'`. `key_loose` is a
 * lowercase, diacritic/aspiration-tolerant fold of SLP1 used for fuzzy trigram
 * search (pg_trgm lowercases its input, so it must never decide exact hits).
 * All keys are produced by the Rust importer (`src/translit.rs`).
 */

/** `text COLLATE "C"`: byte-wise ordering, needed for SLP1 prefix search. */
const textC = customType<{ data: string }>({
  dataType: () => 'text COLLATE "C"',
});

const tsvector = customType<{ data: string }>({
  dataType: () => 'tsvector',
});

export const dictKind = pgEnum('dict_kind', [
  'lexicon',
  'thesaurus',
  'dhatu_work',
  'sutra_commentary',
  'concordance',
]);

export const lang = pgEnum('lang', ['sa', 'en', 'hi']);

export const keyKind = pgEnum('key_kind', [
  /** The display headword itself */
  'headword',
  /** Stem (prātipadika) form, e.g. Apte ज्ञानम् → ज्ञान */
  'pratipadika',
  /** Other spelling / variant of the headword from a sibling file */
  'variant',
  /** Alternate lookup key from a StarDict `.syn` file */
  'syn',
]);

/** One row per *work* (not per StarDict file). */
export const dictionaries = pgTable('dictionaries', {
  id: integer().primaryKey().generatedAlwaysAsIdentity(),
  /** Stable id used in URLs and API filters: `mw`, `apte`, `kashika`, … */
  slug: varchar({ length: 64 }).notNull().unique(),
  name: text().notNull(),
  shortName: varchar({ length: 32 }).notNull(),
  kind: dictKind().notNull(),
  srcLang: lang().notNull(),
  tgtLang: lang().notNull(),
  description: text(),
  attribution: text(),
  /** Display order in search results (lower first) */
  sortOrder: integer().notNull().default(100),
  entryCount: integer().notNull().default(0),
  /** sha256 of the source files of the last import (idempotent re-imports) */
  sourceHash: text(),
  importedAt: timestamp({ withTimezone: true }),
});

/** Aṣṭādhyāyī sūtras; id is `a.p.nnn`, e.g. `1.1.001`. */
export const sutras = pgTable(
  'sutras',
  {
    id: varchar({ length: 12 }).primaryKey(),
    adhyaya: smallint().notNull(),
    pada: smallint().notNull(),
    num: smallint().notNull(),
    /** Sūtra text in Devanagari */
    text: text().notNull(),
    textSlp1: textC().notNull(),
  },
  (t) => [index('sutras_order_idx').on(t.adhyaya, t.pada, t.num)],
);

/**
 * Dhātupāṭha roots. `number` (`gaṇa.index`, e.g. `1.1147`) is not unique in
 * the source (a few roots are listed twice), so it is only indexed.
 */
export const dhatus = pgTable(
  'dhatus',
  {
    id: integer().primaryKey().generatedAlwaysAsIdentity(),
    number: varchar({ length: 16 }).notNull(),
    /** Root as taught, with anubandhas where given (e.g. आपॢ) */
    upadesha: text().notNull(),
    /** Clean root used as headword (e.g. आप्) */
    root: text().notNull(),
    rootSlp1: textC().notNull(),
    artha: text().notNull(),
    gana: smallint().notNull(),
    /** परस्मैपद / आत्मनेपद / उभयपद as P | A | U */
    pada: varchar({ length: 1 }),
    /** सेट् / अनिट् / वेट् */
    it: varchar({ length: 8 }),
    /** Remaining source text (upasarga, variant readings, sūtra refs) */
    extra: text(),
    seq: integer().notNull(),
  },
  (t) => [
    index('dhatus_root_slp1_idx').on(t.rootSlp1),
    index('dhatus_number_idx').on(t.number),
    index('dhatus_gana_idx').on(t.gana, t.seq),
  ],
);

/**
 * Generic dictionary entry: one headword group of a work, in book order.
 * `senses` keeps one item per source sense line (sanitized HTML fragments).
 */
export const entries = pgTable(
  'entries',
  {
    id: integer().primaryKey().generatedAlwaysAsIdentity(),
    dictionaryId: integer()
      .notNull()
      .references(() => dictionaries.id, { onDelete: 'cascade' }),
    /** 0-based position in the book */
    seq: integer().notNull(),
    /** Display headword, Devanagari NFC (Latin for English/BHS headwords) */
    headword: text().notNull(),
    senses: text().array().notNull(),
    /** Parsed structure (Amara relations, RV location, …) */
    data: jsonb(),
    sutraId: varchar({ length: 12 }).references(() => sutras.id, { onDelete: 'set null' }),
    dhatuId: integer().references(() => dhatus.id, { onDelete: 'set null' }),
    /** Plain English text of the entry (Sanskrit/markup stripped); sa→en works only */
    glossEn: text(),
    searchEn: tsvector().generatedAlwaysAs(
      (): ReturnType<typeof sql> => sql`to_tsvector('english', coalesce(gloss_en, ''))`,
    ),
  },
  (t) => [
    uniqueIndex('entries_dict_seq_idx').on(t.dictionaryId, t.seq),
    index('entries_sutra_idx').on(t.sutraId),
    index('entries_dhatu_idx').on(t.dhatuId),
    index('entries_search_en_idx')
      .using('gin', t.searchEn)
      .where(sql`gloss_en is not null`),
  ],
);

/** Search keys. Many per entry (headword, stem, variants, syn keys). */
export const entryKeys = pgTable(
  'entry_keys',
  {
    entryId: integer()
      .notNull()
      .references(() => entries.id, { onDelete: 'cascade' }),
    /** Denormalized for filtering without a join */
    dictionaryId: integer().notNull(),
    keySlp1: textC().notNull(),
    keyLoose: textC().notNull(),
    kind: keyKind().notNull(),
  },
  (t) => [
    primaryKey({ columns: [t.entryId, t.keySlp1] }),
    index('entry_keys_slp1_idx').on(t.keySlp1, t.dictionaryId),
    index('entry_keys_loose_idx').on(t.keyLoose),
    index('entry_keys_loose_trgm_idx').using('gin', t.keyLoose.op('gin_trgm_ops')),
  ],
);

/** Inflected form → lemma (JNU tiṅanta now; Heritage declensions later). */
export const wordForms = pgTable(
  'word_forms',
  {
    id: integer().primaryKey().generatedAlwaysAsIdentity(),
    /** e.g. `jnu_tinanta` */
    source: varchar({ length: 32 }).notNull(),
    form: text().notNull(),
    formSlp1: textC().notNull(),
    lemma: text().notNull(),
    lemmaSlp1: textC().notNull(),
    /** Human-readable analysis, e.g. `सत्तायाम्, भ्वादिगण, परस्मै, लट्` */
    analysis: text().notNull(),
    /** Full paradigm as rows of forms (3×3 for tiṅanta) */
    paradigm: jsonb(),
  },
  (t) => [
    index('word_forms_form_idx').on(t.formSlp1),
    index('word_forms_lemma_idx').on(t.lemmaSlp1),
  ],
);
