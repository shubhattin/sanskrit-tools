//! Dictionary agent (`/agent`): chat models, tools over the dictionary data, UI streaming and
//! storage. Contract: `docs/agent.md`.

pub mod history;
pub mod live;
pub mod models;
pub mod openrouter;
pub mod prompt;
pub mod store;
pub mod text;
pub mod tools;
pub mod turn;
pub mod ui_stream;
pub mod vidyut_tools;
pub mod web_search;
