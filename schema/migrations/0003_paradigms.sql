CREATE TABLE "paradigm_forms" (
	"paradigm_id" integer NOT NULL,
	"form_slp1" text COLLATE "C" NOT NULL,
	"cells" smallint[] NOT NULL,
	CONSTRAINT "paradigm_forms_paradigm_id_form_slp1_pk" PRIMARY KEY("paradigm_id","form_slp1")
);
--> statement-breakpoint
CREATE TABLE "paradigms" (
	"id" integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY (sequence name "paradigms_id_seq" INCREMENT BY 1 MINVALUE 1 MAXVALUE 2147483647 START WITH 1 CACHE 1),
	"source" varchar(32) NOT NULL,
	"seq" integer NOT NULL,
	"lemma" text NOT NULL,
	"lemma_slp1" text COLLATE "C" NOT NULL,
	"root" text,
	"root_slp1" text COLLATE "C",
	"label" text,
	"tables" jsonb NOT NULL
);
--> statement-breakpoint
ALTER TABLE "paradigm_forms" ADD CONSTRAINT "paradigm_forms_paradigm_id_paradigms_id_fk" FOREIGN KEY ("paradigm_id") REFERENCES "public"."paradigms"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
CREATE INDEX "paradigm_forms_form_idx" ON "paradigm_forms" USING btree ("form_slp1");--> statement-breakpoint
CREATE UNIQUE INDEX "paradigms_source_seq_idx" ON "paradigms" USING btree ("source","seq");--> statement-breakpoint
CREATE INDEX "paradigms_lemma_idx" ON "paradigms" USING btree ("lemma_slp1");--> statement-breakpoint
CREATE INDEX "paradigms_root_idx" ON "paradigms" USING btree ("root_slp1");