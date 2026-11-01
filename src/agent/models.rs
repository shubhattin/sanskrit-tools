//! The two chat models, both through OpenRouter at their default reasoning.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AgentModel {
  #[default]
  #[serde(rename = "gpt-6-luna")]
  GptLuna,
  #[serde(rename = "claude-sonnet-5.5")]
  SonnetFive,
}

impl AgentModel {
  pub const ALL: [AgentModel; 2] = [AgentModel::GptLuna, AgentModel::SonnetFive];

  pub fn id(self) -> &'static str {
    match self {
      AgentModel::GptLuna => "gpt-6-luna",
      AgentModel::SonnetFive => "claude-sonnet-5.5",
    }
  }

  pub fn from_id(id: &str) -> Option<Self> {
    match id {
      "gpt-6-luna" => Some(AgentModel::GptLuna),
      "claude-sonnet-5.5" => Some(AgentModel::SonnetFive),
      _ => None,
    }
  }

  /// OpenRouter model slug.
  pub fn provider_id(self) -> &'static str {
    match self {
      AgentModel::GptLuna => "openai/gpt-6-luna",
      AgentModel::SonnetFive => "anthropic/claude-sonnet-5.5",
    }
  }

  /// Anthropic caches only when the request asks (OpenAI caches on its own).
  pub fn needs_cache_control(self) -> bool {
    matches!(self, AgentModel::SonnetFive)
  }
}

/// Cheap model for chat titles.
pub const UTILITY_MODEL: AgentModel = AgentModel::GptLuna;
