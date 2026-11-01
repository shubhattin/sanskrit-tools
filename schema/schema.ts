import { sql } from 'drizzle-orm';
import {
  boolean,
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

/**
 * Shared table structure (labels, columns, row labels, empty cell arrays) for
 * paradigms that store only their forms (`paradigms.forms`). Declension and
 * kṛdanta tables use a handful of layouts, so labels are not repeated per row.
 */
export const paradigmLayouts = pgTable('paradigm_layouts', {
  id: integer().primaryKey().generatedAlwaysAsIdentity(),
  /** sha256 (first 32 hex chars) of the layout JSON, the dedup key */
  key: varchar({ length: 32 }).notNull().unique(),
  layout: jsonb().notNull(),
});

/**
 * One inflection table set: a declined stem (per gender), a kṛdanta stem
 * (with its root) or a conjugated root (all derivations/lakāras/voices).
 * Display structure (`docs/api.md`, `Paradigm.tables`) is either stored whole
 * in `tables`, or as `layout_id` + `forms` (cells in order joined by `|`,
 * forms within a cell by `,`). Cells are numbered row-major across all tables;
 * `paradigm_forms.cells` points at those numbers.
 */
export const paradigms = pgTable(
  'paradigms',
  {
    id: integer().primaryKey().generatedAlwaysAsIdentity(),
    /** `heritage_decl` | `heritage_kridanta` | `heritage_conj` | `jnu_tinanta` */
    source: varchar({ length: 32 }).notNull(),
    /** Order within the source */
    seq: integer().notNull(),
    lemma: text().notNull(),
    lemmaSlp1: textC().notNull(),
    /** Verbal root for kṛdanta stems */
    root: text(),
    rootSlp1: textC(),
    /** Gender (पुमान् / स्त्री / नपुंसकम्), gaṇa (गणः १) or JNU analysis */
    label: text(),
    /** Full display structure (when not layout-based) */
    tables: jsonb(),
    layoutId: integer().references(() => paradigmLayouts.id),
    /** Cell contents for layout-based paradigms */
    forms: text(),
  },
  (t) => [
    uniqueIndex('paradigms_source_seq_idx').on(t.source, t.seq),
    index('paradigms_lemma_idx').on(t.lemmaSlp1),
    index('paradigms_root_idx').on(t.rootSlp1),
  ],
);

/** Inflected form → paradigm, with the cell numbers where it occurs. */
export const paradigmForms = pgTable(
  'paradigm_forms',
  {
    paradigmId: integer()
      .notNull()
      .references(() => paradigms.id, { onDelete: 'cascade' }),
    formSlp1: textC().notNull(),
    cells: smallint().array().notNull(),
  },
  (t) => [
    primaryKey({ columns: [t.paradigmId, t.formSlp1] }),
    index('paradigm_forms_form_idx').on(t.formSlp1),
  ],
);

/** One conversation with the dictionary agent (`/agent/$chatId`). */
export const agentChats = pgTable(
  'agent_chats',
  {
    /** Same id `useChat` uses on the client. */
    id: text().primaryKey(),
    title: text().notNull().default(''),
    /** Model of the latest turn (`gpt-6-luna` | `claude-sonnet-5.5`); can change mid-chat. */
    model: text().notNull(),
    /** Dictionary slugs the agent searches by default (the reader's search preference). */
    dicts: text().array().notNull().default(sql`'{}'::text[]`),
    /** True while a reply is being generated (possibly after the tab was closed). */
    streaming: boolean().notNull().default(false),
    /** Set by Stop; the generating request (on whichever instance) polls it and aborts. */
    stopRequestedAt: timestamp({ withTimezone: true }),
    createdAt: timestamp({ withTimezone: true }).notNull().defaultNow(),
    /** Bumped on every turn; sorts the sidebar. */
    updatedAt: timestamp({ withTimezone: true }).notNull().defaultNow(),
  },
  (t) => [index('agent_chats_updated_at_idx').on(t.updatedAt)],
);

/**
 * AI SDK `UIMessage`s, stored as-is: text, reasoning (with the provider's
 * `reasoning_details` for replay), tool calls with input + output, sources.
 */
export const agentMessages = pgTable(
  'agent_messages',
  {
    id: text().primaryKey(),
    chatId: text()
      .notNull()
      .references(() => agentChats.id, { onDelete: 'cascade' }),
    /** `user` | `assistant` */
    role: text().notNull(),
    /** `UIMessage.parts` */
    parts: jsonb().notNull(),
    /** `UIMessage.metadata`: model, cost (USD), token usage, web searches */
    metadata: jsonb(),
    /** Order within the chat */
    position: integer().notNull(),
    createdAt: timestamp({ withTimezone: true }).notNull().defaultNow(),
  },
  (t) => [index('agent_messages_chat_id_position_idx').on(t.chatId, t.position)],
);
