//! Stored `UIMessage`s → aisdk messages for the next turn.

use crate::agent::openrouter::{ReasoningItems, new_step_tag};
use aisdk::core::language_model::LanguageModelResponseContentType as Content;
use aisdk::core::messages::{AssistantMessage, Message, UserMessage};
use aisdk::core::tools::{ToolCallInfo, ToolResultInfo};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A stored message (`agent_messages` row / AI SDK `UIMessage`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiMessage {
  pub id: String,
  pub role: String,
  pub parts: Vec<Value>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub metadata: Option<Value>,
}

/// Text of a user message's text parts.
pub fn user_text(message: &UiMessage) -> String {
  message
    .parts
    .iter()
    .filter(|p| p["type"] == "text")
    .filter_map(|p| p["text"].as_str())
    .collect::<Vec<_>>()
    .join("\n")
}

pub fn to_model_messages(messages: &[UiMessage]) -> Vec<Message> {
  let mut out = Vec::new();
  for message in messages {
    match message.role.as_str() {
      "user" => {
        let text = user_text(message);
        if !text.trim().is_empty() {
          out.push(Message::User(UserMessage::new(text)));
        }
      }
      "assistant" => assistant_messages(&message.parts, &mut out),
      _ => {}
    }
  }
  out
}

/// One assistant `UIMessage` holds every step of a reply; each step becomes reasoning, text,
/// then (call, result) pairs, tagged with one step tag — the order aisdk itself produces.
fn assistant_messages(parts: &[Value], out: &mut Vec<Message>) {
  for step in parts.split(|p| p["type"] == "step-start") {
    let tag = new_step_tag();
    let mut details = Vec::new();
    let mut text = String::new();
    let mut calls = Vec::new();
    for part in step {
      let kind = part["type"].as_str().unwrap_or("");
      match kind {
        // Only the opaque details are replayed; providers ignore reasoning text.
        "reasoning" => {
          if let Some(d) = part["providerMetadata"]["openrouter"]["reasoning_items"].as_array() {
            details.extend(d.iter().cloned());
          }
        }
        "text" => text.push_str(part["text"].as_str().unwrap_or("")),
        _ if kind.starts_with("tool-") => {
          let output = match part["state"].as_str() {
            Some("output-available") => match &part["output"] {
              Value::String(s) => s.clone(),
              other => other.to_string(),
            },
            Some("output-error") => {
              format!("Error: {}", part["errorText"].as_str().unwrap_or("failed"))
            }
            // Never finished: leave the call out rather than send it without a result.
            _ => continue,
          };
          let Some(id) = part["toolCallId"].as_str() else {
            continue;
          };
          calls.push((
            id.to_string(),
            kind["tool-".len()..].to_string(),
            part["input"].clone(),
            output,
          ));
        }
        _ => {}
      }
    }
    if !details.is_empty() {
      let extensions = aisdk::extensions::Extensions::default();
      extensions.insert(tag);
      extensions.insert(ReasoningItems(details));
      out.push(assistant(Content::Reasoning {
        content: String::new(),
        extensions,
      }));
    }
    if !text.is_empty() {
      out.push(assistant(Content::Text(text)));
    }
    for (id, name, input, output) in calls {
      let mut call = ToolCallInfo::new(&name);
      call.id(&id);
      call.input(if input.is_object() {
        input
      } else {
        Value::Object(Default::default())
      });
      call.extensions.insert(tag);
      out.push(assistant(Content::ToolCall(call)));
      let mut result = ToolResultInfo::new(&name);
      result.id(&id);
      result.output(Value::String(output));
      out.push(Message::Tool(result));
    }
  }
}

fn assistant(content: Content) -> Message {
  Message::Assistant(AssistantMessage::new(content, None))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::agent::openrouter::input_items;
  use serde_json::json;

  #[test]
  fn replays_steps_as_provider_messages() {
    let messages = vec![
      UiMessage {
        id: "u1".into(),
        role: "user".into(),
        parts: vec![json!({ "type": "text", "text": "धर्म?" })],
        metadata: None,
      },
      UiMessage {
        id: "a1".into(),
        role: "assistant".into(),
        parts: vec![
          json!({ "type": "step-start" }),
          json!({ "type": "reasoning", "text": "look up", "state": "done",
                  "providerMetadata": { "openrouter": { "reasoning_items": [{ "type": "reasoning", "encrypted_content": "x" }] } } }),
          json!({ "type": "tool-web_search", "toolCallId": "ws1", "state": "output-available",
                  "input": { "query": "q" }, "output": { "summary": "Sāyaṇa.", "sources": [] } }),
          json!({ "type": "tool-search_dictionaries", "toolCallId": "c1", "state": "output-available",
                  "input": { "query": "धर्म" }, "output": { "results": [] } }),
          json!({ "type": "tool-get_entry", "toolCallId": "c2", "state": "output-error",
                  "input": { "id": 9 }, "errorText": "not found" }),
          json!({ "type": "tool-get_entry", "toolCallId": "c3", "state": "input-available", "input": {} }),
          json!({ "type": "step-start" }),
          json!({ "type": "text", "text": "It means law.", "state": "done" }),
        ],
        metadata: None,
      },
    ];
    let items = input_items(&to_model_messages(&messages));
    let kinds: Vec<&str> = items.iter().map(|i| i["type"].as_str().unwrap()).collect();
    assert_eq!(
      kinds,
      [
        "message",
        "reasoning",
        "function_call",
        "function_call",
        "function_call",
        "function_call_output",
        "function_call_output",
        "function_call_output",
        "message",
      ]
    );
    assert_eq!(items[1]["encrypted_content"], "x");
    assert_eq!(items[2]["call_id"], "ws1");
    assert_eq!(items[3]["call_id"], "c1");
    assert_eq!(
      items[5]["output"],
      "{\"sources\":[],\"summary\":\"Sāyaṇa.\"}"
    );
    assert_eq!(items[6]["output"], "{\"results\":[]}");
    assert_eq!(items[7]["output"], "Error: not found");
    assert_eq!(items[8]["content"][0]["text"], "It means law.");
  }
}
