CREATE TYPE "public"."dict_kind" AS ENUM('lexicon', 'thesaurus', 'dhatu_work', 'sutra_commentary', 'concordance');--> statement-breakpoint
CREATE TYPE "public"."key_kind" AS ENUM('headword', 'pratipadika', 'variant', 'syn');--> statement-breakpoint
CREATE TYPE "public"."lang" AS ENUM('sa', 'en', 'hi');--> statement-breakpoint
CREATE TABLE "dhatus" (
	"id" integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY (sequence name "dhatus_id_seq" INCREMENT BY 1 MINVALUE 1 MAXVALUE 2147483647 START WITH 1 CACHE 1),
	"number" varchar(16) NOT NULL,
	"upadesha" text NOT NULL,
	"root" text NOT NULL,
	"root_slp1" text COLLATE "C" NOT NULL,
	"artha" text NOT NULL,
	"gana" smallint NOT NULL,
	"pada" varchar(1),
	"it" varchar(8),
	"extra" text,
	"seq" integer NOT NULL
);
--> statement-breakpoint
CREATE TABLE "dictionaries" (
	"id" integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY (sequence name "dictionaries_id_seq" INCREMENT BY 1 MINVALUE 1 MAXVALUE 2147483647 START WITH 1 CACHE 1),
	"slug" varchar(64) NOT NULL,
	"name" text NOT NULL,
	"short_name" varchar(32) NOT NULL,
	"kind" "dict_kind" NOT NULL,
	"src_lang" "lang" NOT NULL,
	"tgt_lang" "lang" NOT NULL,
	"description" text,
	"attribution" text,
	"sort_order" integer DEFAULT 100 NOT NULL,
	"entry_count" integer DEFAULT 0 NOT NULL,
	"source_hash" text,
	"imported_at" timestamp with time zone,
	CONSTRAINT "dictionaries_slug_unique" UNIQUE("slug")
);
--> statement-breakpoint
CREATE TABLE "entries" (
	"id" integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY (sequence name "entries_id_seq" INCREMENT BY 1 MINVALUE 1 MAXVALUE 2147483647 START WITH 1 CACHE 1),
	"dictionary_id" integer NOT NULL,
	"seq" integer NOT NULL,
	"headword" text NOT NULL,
	"senses" text[] NOT NULL,
	"data" jsonb,
	"sutra_id" varchar(12),
	"dhatu_id" integer,
	"gloss_en" text,
	"search_en" "tsvector" GENERATED ALWAYS AS (to_tsvector('english', coalesce(gloss_en, ''))) STORED
);
--> statement-breakpoint
CREATE TABLE "entry_keys" (
	"entry_id" integer NOT NULL,
	"dictionary_id" integer NOT NULL,
	"key_slp1" text COLLATE "C" NOT NULL,
	"key_loose" text COLLATE "C" NOT NULL,
	"kind" "key_kind" NOT NULL,
	CONSTRAINT "entry_keys_entry_id_key_slp1_pk" PRIMARY KEY("entry_id","key_slp1")
);
--> statement-breakpoint
CREATE TABLE "sutras" (
	"id" varchar(12) PRIMARY KEY NOT NULL,
	"adhyaya" smallint NOT NULL,
	"pada" smallint NOT NULL,
	"num" smallint NOT NULL,
	"text" text NOT NULL,
	"text_slp1" text COLLATE "C" NOT NULL
);
--> statement-breakpoint
CREATE TABLE "word_forms" (
	"id" integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY (sequence name "word_forms_id_seq" INCREMENT BY 1 MINVALUE 1 MAXVALUE 2147483647 START WITH 1 CACHE 1),
	"source" varchar(32) NOT NULL,
	"form" text NOT NULL,
	"form_slp1" text COLLATE "C" NOT NULL,
	"lemma" text NOT NULL,
	"lemma_slp1" text COLLATE "C" NOT NULL,
	"analysis" text NOT NULL,
	"paradigm" jsonb
);
--> statement-breakpoint
ALTER TABLE "entries" ADD CONSTRAINT "entries_dictionary_id_dictionaries_id_fk" FOREIGN KEY ("dictionary_id") REFERENCES "public"."dictionaries"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "entries" ADD CONSTRAINT "entries_sutra_id_sutras_id_fk" FOREIGN KEY ("sutra_id") REFERENCES "public"."sutras"("id") ON DELETE set null ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "entries" ADD CONSTRAINT "entries_dhatu_id_dhatus_id_fk" FOREIGN KEY ("dhatu_id") REFERENCES "public"."dhatus"("id") ON DELETE set null ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "entry_keys" ADD CONSTRAINT "entry_keys_entry_id_entries_id_fk" FOREIGN KEY ("entry_id") REFERENCES "public"."entries"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
CREATE INDEX "dhatus_root_slp1_idx" ON "dhatus" USING btree ("root_slp1");--> statement-breakpoint
CREATE INDEX "dhatus_number_idx" ON "dhatus" USING btree ("number");--> statement-breakpoint
CREATE INDEX "dhatus_gana_idx" ON "dhatus" USING btree ("gana","seq");--> statement-breakpoint
CREATE UNIQUE INDEX "entries_dict_seq_idx" ON "entries" USING btree ("dictionary_id","seq");--> statement-breakpoint
CREATE INDEX "entries_sutra_idx" ON "entries" USING btree ("sutra_id");--> statement-breakpoint
CREATE INDEX "entries_dhatu_idx" ON "entries" USING btree ("dhatu_id");--> statement-breakpoint
CREATE INDEX "entries_search_en_idx" ON "entries" USING gin ("search_en") WHERE gloss_en is not null;--> statement-breakpoint
CREATE INDEX "entry_keys_slp1_idx" ON "entry_keys" USING btree ("key_slp1","dictionary_id");--> statement-breakpoint
CREATE INDEX "entry_keys_loose_idx" ON "entry_keys" USING btree ("key_loose");--> statement-breakpoint
CREATE INDEX "entry_keys_loose_trgm_idx" ON "entry_keys" USING gin ("key_loose" gin_trgm_ops);--> statement-breakpoint
CREATE INDEX "sutras_order_idx" ON "sutras" USING btree ("adhyaya","pada","num");--> statement-breakpoint
CREATE INDEX "word_forms_form_idx" ON "word_forms" USING btree ("form_slp1");--> statement-breakpoint
CREATE INDEX "word_forms_lemma_idx" ON "word_forms" USING btree ("lemma_slp1");