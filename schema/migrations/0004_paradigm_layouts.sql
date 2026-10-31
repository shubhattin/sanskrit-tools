CREATE TABLE "paradigm_layouts" (
	"id" integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY (sequence name "paradigm_layouts_id_seq" INCREMENT BY 1 MINVALUE 1 MAXVALUE 2147483647 START WITH 1 CACHE 1),
	"key" varchar(32) NOT NULL,
	"layout" jsonb NOT NULL,
	CONSTRAINT "paradigm_layouts_key_unique" UNIQUE("key")
);
--> statement-breakpoint
ALTER TABLE "paradigms" ALTER COLUMN "tables" DROP NOT NULL;--> statement-breakpoint
ALTER TABLE "paradigms" ADD COLUMN "layout_id" integer;--> statement-breakpoint
ALTER TABLE "paradigms" ADD COLUMN "forms" text;--> statement-breakpoint
ALTER TABLE "paradigms" ADD CONSTRAINT "paradigms_layout_id_paradigm_layouts_id_fk" FOREIGN KEY ("layout_id") REFERENCES "public"."paradigm_layouts"("id") ON DELETE no action ON UPDATE no action;