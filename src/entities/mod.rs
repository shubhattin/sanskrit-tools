//! SeaORM entities (PostgreSQL `sans_dict`). Mirrors `schema/schema.ts`
//! (drizzle is the source of truth; columns are snake_case).
//!
//! Generated columns (`entries.search_en`) are not mapped: they are only used
//! inside full-text SQL.

pub mod agent_chats;
pub mod agent_messages;
pub mod dhatus;
pub mod dictionaries;
pub mod entries;
pub mod entry_keys;
pub mod enums;
pub mod paradigm_forms;
pub mod paradigm_layouts;
pub mod paradigms;
pub mod sutras;
