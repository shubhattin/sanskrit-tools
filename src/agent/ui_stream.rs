//! aisdk stream → AI SDK UI message stream (SSE chunks `useChat` consumes), and the same reply as
//! a `UIMessage` to store. Both come from one pass, so what the reader saw live is what reloads.
//!
//! aisdk has no step events: a step starts at the first model chunk after the previous step's
//! tool results, and the step's [`StepRecord`] (cost, citations, reasoning items) is taken from
//! the [`ReplyMeter`] when it ends.

use crate::agent::models::AgentModel;
use crate::agent::openrouter::{ReplyMeter, SharedMeter, StepRecord, StepUsage};
use crate::agent::web_search::WEB_SEARCH_TOOL;
use aisdk::core::LanguageModelStreamChunkType as Chunk;
use futures_util::{Stream, StreamExt};
use serde_json::{Value, json};
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

/// One SSE frame (`data: …\n\n`).
pub fn sse(chunk: &Value) -> String {
  format!("data: {chunk}\n\n")
}

pub const SSE_DONE: &str = "data: [DONE]\n\n";

/// The finished reply.
pub struct Reply {
  /// `UIMessage.parts`, every part in a final state.
  pub parts: Vec<Value>,
  pub metadata: Value,
  /// Why the reply ended early, if it did (not set for Stop).
  pub error: Option<String>,
}

#[derive(Clone, Copy)]
enum Open {
  Reasoning(usize),
  Text(usize),
}

struct Encoder {
  out: UnboundedSender<String>,
  meter: SharedMeter,
  model: AgentModel,
  parts: Vec<Value>,
  in_step: bool,
  step_first_part: usize,
  step_has_results: bool,
  open: Option<Open>,
  next_id: usize,
  cost: f64,
  usage: StepUsage,
  /// Finished `web_search` tool calls in the reply.
  web_searches: u32,
}

impl Encoder {
  fn emit(&self, chunk: Value) {
    // The reader may have gone; the reply still finishes and is saved.
    let _ = self.out.send(sse(&chunk));
  }

  fn id(&mut self, prefix: &str) -> String {
    self.next_id += 1;
    format!("{prefix}{}", self.next_id)
  }

  fn metadata(&self) -> Value {
    json!({
      "model": self.model.id(),
      "cost": self.cost,
      "usage": {
        "inputTokens": self.usage.input_tokens,
        "outputTokens": self.usage.output_tokens,
        "cachedInputTokens": self.usage.cached_input_tokens,
        "reasoningTokens": self.usage.reasoning_tokens,
      },
      "webSearches": self.web_searches,
    })
  }

  /// Called before every chunk the model produces.
  fn model_chunk(&mut self) {
    if self.in_step && self.step_has_results {
      self.end_step();
    }
    if !self.in_step {
      self.in_step = true;
      self.step_has_results = false;
      self.step_first_part = self.parts.len();
      self.parts.push(json!({ "type": "step-start" }));
      self.emit(json!({ "type": "start-step" }));
    }
  }

  fn close_open(&mut self) {
    match self.open.take() {
      Some(Open::Reasoning(i)) => {
        self.parts[i]["state"] = "done".into();
        let id = self.parts[i]["_id"].take();
        self.emit(json!({ "type": "reasoning-end", "id": id }));
      }
      Some(Open::Text(i)) => {
        self.parts[i]["state"] = "done".into();
        let id = self.parts[i]["_id"].take();
        self.emit(json!({ "type": "text-end", "id": id }));
      }
      None => {}
    }
  }

  fn open_block(&mut self, reasoning: bool) -> usize {
    match (self.open, reasoning) {
      (Some(Open::Reasoning(i)), true) | (Some(Open::Text(i)), false) => return i,
      _ => self.close_open(),
    }
    let (kind, prefix) = if reasoning {
      ("reasoning", "r")
    } else {
      ("text", "t")
    };
    let id = self.id(prefix);
    self
      .parts
      .push(json!({ "type": kind, "text": "", "state": "streaming", "_id": id }));
    let i = self.parts.len() - 1;
    self.open = Some(if reasoning {
      Open::Reasoning(i)
    } else {
      Open::Text(i)
    });
    self.emit(json!({ "type": format!("{kind}-start"), "id": id }));
    i
  }

  fn delta(&mut self, reasoning: bool, delta: String) {
    let i = self.open_block(reasoning);
    let text = format!("{}{}", self.parts[i]["text"].as_str().unwrap_or(""), delta);
    self.parts[i]["text"] = text.into();
    let kind = if reasoning { "reasoning" } else { "text" };
    let id = self.parts[i]["_id"].clone();
    self.emit(json!({ "type": format!("{kind}-delta"), "id": id, "delta": delta }));
  }

  fn tool_part(&mut self, tool_call_id: &str) -> Option<usize> {
    self
      .parts
      .iter()
      .rposition(|p| p["toolCallId"] == tool_call_id)
  }

  fn tool_start(&mut self, id: String, name: String) {
    self.close_open();
    self.parts.push(json!({
      "type": format!("tool-{name}"),
      "toolCallId": id,
      "state": "input-streaming",
    }));
    self.emit(json!({ "type": "tool-input-start", "toolCallId": id, "toolName": name }));
  }

  fn tool_input(&mut self, id: String, name: String, input: Value) {
    let i = match self.tool_part(&id) {
      Some(i) => i,
      None => {
        self.model_chunk();
        self.tool_start(id.clone(), name.clone());
        self.parts.len() - 1
      }
    };
    self.parts[i]["state"] = "input-available".into();
    self.parts[i]["input"] = input.clone();
    self.emit(json!({
      "type": "tool-input-available", "toolCallId": id, "toolName": name, "input": input,
    }));
  }

  fn tool_output(&mut self, id: String, name: &str, output: &Result<Value, aisdk::Error>) {
    let Some(i) = self.tool_part(&id) else { return };
    // A tool result ends the step: the next model chunk opens a new one.
    self.step_has_results = true;
    if name == WEB_SEARCH_TOOL && tool_outcome(output).is_ok() {
      self.web_searches += 1;
    }
    let chunk = match tool_outcome(output) {
      Ok(value) => {
        self.parts[i]["state"] = "output-available".into();
        self.parts[i]["output"] = value.clone();
        json!({ "type": "tool-output-available", "toolCallId": id, "output": value })
      }
      Err(error) => {
        self.parts[i]["state"] = "output-error".into();
        self.parts[i]["errorText"] = error.clone().into();
        json!({ "type": "tool-output-error", "toolCallId": id, "errorText": error })
      }
    };
    self.emit(chunk);
  }

  fn apply_record(&mut self, record: StepRecord) {
    self.cost += record.cost;
    self.usage.input_tokens += record.usage.input_tokens;
    self.usage.output_tokens += record.usage.output_tokens;
    self.usage.cached_input_tokens += record.usage.cached_input_tokens;
    self.usage.reasoning_tokens += record.usage.reasoning_tokens;

    if !record.reasoning_items.is_empty() {
      let details = json!({ "openrouter": { "reasoning_items": record.reasoning_items } });
      let reasoning = (self.step_first_part..self.parts.len())
        .rev()
        .find(|&i| self.parts[i]["type"] == "reasoning");
      match reasoning {
        Some(i) => self.parts[i]["providerMetadata"] = details,
        // Encrypted reasoning without a readable summary: kept (empty) for replay only.
        None => self.parts.push(json!({
          "type": "reasoning", "text": "", "state": "done", "providerMetadata": details,
        })),
      }
    }
  }

  fn end_step(&mut self) {
    self.close_open();
    if let Some(record) = ReplyMeter::pop(&self.meter) {
      self.apply_record(record);
    }
    self.emit(json!({ "type": "finish-step" }));
    self.emit(json!({ "type": "message-metadata", "messageMetadata": self.metadata() }));
    self.in_step = false;
    self.step_has_results = false;
  }

  /// Every part in a final state: open blocks done, unfinished tool calls errored (a stored tool
  /// call without a result can't be replayed to the model).
  fn finalize_parts(&mut self, why: &str) {
    for part in &mut self.parts {
      if let Value::Object(map) = part {
        map.remove("_id");
      }
      if part["state"] == "streaming" {
        part["state"] = "done".into();
      }
      let state = part["state"].as_str().unwrap_or("");
      if part["type"]
        .as_str()
        .is_some_and(|t| t.starts_with("tool-"))
        && matches!(state, "input-streaming" | "input-available")
      {
        part["state"] = "output-error".into();
        part["errorText"] = why.into();
        if part.get("input").is_none() {
          part["input"] = json!({});
        }
      }
    }
  }
}

/// Tool result as aisdk reports it → output JSON, or the error text. aisdk turns a failed tool
/// into the string `Error: <error>`; our tools return JSON objects otherwise.
fn tool_outcome(output: &Result<Value, aisdk::Error>) -> Result<Value, String> {
  match output {
    Err(e) => Err(e.to_string()),
    Ok(Value::String(s)) => match s.strip_prefix("Error: ") {
      Some(err) => Err(err.strip_prefix("Tool error: ").unwrap_or(err).to_string()),
      None => Ok(serde_json::from_str(s).unwrap_or_else(|_| Value::String(s.clone()))),
    },
    Ok(other) => Ok(other.clone()),
  }
}

/// Encode the whole reply. Returns once the model loop is done (or stopped / failed).
pub async fn encode(
  mut stream: impl Stream<Item = Chunk> + Unpin,
  message_id: &str,
  model: AgentModel,
  meter: SharedMeter,
  cancel: CancellationToken,
  out: UnboundedSender<String>,
) -> Reply {
  let mut e = Encoder {
    out,
    meter,
    model,
    parts: Vec::new(),
    in_step: false,
    step_first_part: 0,
    step_has_results: false,
    open: None,
    next_id: 0,
    cost: 0.0,
    usage: StepUsage::default(),
    web_searches: 0,
  };
  e.emit(json!({
    "type": "start",
    "messageId": message_id,
    "messageMetadata": { "model": model.id() },
  }));

  let mut error = None;
  while let Some(chunk) = stream.next().await {
    match chunk {
      Chunk::ReasoningStart => {
        e.model_chunk();
        e.open_block(true);
      }
      Chunk::ReasoningDelta(d) => {
        e.model_chunk();
        e.delta(true, d);
      }
      Chunk::TextStart => {
        e.model_chunk();
        e.open_block(false);
      }
      Chunk::TextDelta(d) => {
        e.model_chunk();
        e.delta(false, d);
      }
      Chunk::ReasoningEnd | Chunk::TextEnd => e.close_open(),
      Chunk::ToolCallStart(details) => {
        e.model_chunk();
        e.tool_start(details.id, details.name);
      }
      Chunk::ToolCallDelta { id, delta } => {
        e.emit(json!({ "type": "tool-input-delta", "toolCallId": id, "inputTextDelta": delta }));
      }
      Chunk::ToolCallAvailable(info) => e.tool_input(info.tool.id, info.tool.name, info.input),
      Chunk::ToolCallEnd(result) => {
        e.tool_output(result.tool.id, &result.tool.name, &result.output)
      }
      Chunk::Failed(message) => {
        if !cancel.is_cancelled() {
          error = Some(clean_error(&message));
        }
        break;
      }
      Chunk::Incomplete(_) | Chunk::NotSupported(_) => {}
    }
  }

  if e.in_step {
    e.end_step();
  }
  // Records of calls that produced no visible chunk (or ended the loop) still count.
  while let Some(record) = ReplyMeter::pop(&e.meter) {
    e.apply_record(record);
  }
  let why = if cancel.is_cancelled() {
    "Stopped"
  } else {
    "The reply ended before this tool finished"
  };
  e.finalize_parts(why);
  if let Some(message) = &error {
    e.emit(json!({ "type": "error", "errorText": message }));
  }
  let metadata = e.metadata();
  e.emit(json!({
    "type": "finish",
    "finishReason": if error.is_some() { "error" } else { "stop" },
    "messageMetadata": metadata,
  }));
  let _ = e.out.send(SSE_DONE.to_string());
  Reply {
    parts: e.parts,
    metadata,
    error,
  }
}

/// aisdk wraps provider errors ("Model streaming failed: AI SDK error: …"); show the cause.
fn clean_error(message: &str) -> String {
  let mut m = message.trim();
  for prefix in ["Model streaming failed: ", "AI SDK error: "] {
    m = m.strip_prefix(prefix).unwrap_or(m);
  }
  m.to_string()
}

#[cfg(test)]
mod tests {
  use super::*;
  use aisdk::core::tools::{ToolCallInfo, ToolDetails, ToolResultInfo};
  use futures_util::stream;

  fn frames(rx: &mut tokio::sync::mpsc::UnboundedReceiver<String>) -> Vec<Value> {
    let mut out = Vec::new();
    while let Ok(f) = rx.try_recv() {
      let data = f.trim_start_matches("data: ").trim();
      if data != "[DONE]" {
        out.push(serde_json::from_str(data).unwrap());
      }
    }
    out
  }

  #[tokio::test]
  async fn encodes_steps_and_tools() {
    // Two model calls: one with a dictionary tool call, one with a web search, then the answer.
    let meter = SharedMeter::default();
    let first = StepRecord {
      cost: 0.01,
      reasoning_items: vec![json!({ "type": "reasoning", "signature": "s" })],
      ..Default::default()
    };
    let second = StepRecord {
      cost: 0.02,
      ..Default::default()
    };
    let mut call = ToolCallInfo::new("get_entry");
    call.id("c1");
    call.input(json!({ "id": 1 }));
    let mut result = ToolResultInfo::new("get_entry");
    result.id("c1");
    result.output(Value::String("{\"id\":1}".into()));
    let mut search_call = ToolCallInfo::new("web_search");
    search_call.id("c2");
    search_call.input(json!({ "query": "Sāyaṇa dates" }));
    let mut search_result = ToolResultInfo::new("web_search");
    search_result.id("c2");
    search_result.output(Value::String(
      "{\"summary\":\"Sāyaṇa lived in the 14th century.\",\"sources\":[]}".into(),
    ));
    let chunks = vec![
      Chunk::ReasoningStart,
      Chunk::ReasoningDelta("hmm".into()),
      Chunk::ReasoningEnd,
      Chunk::ToolCallStart(ToolDetails {
        id: "c1".into(),
        name: "get_entry".into(),
      }),
      Chunk::ToolCallDelta {
        id: "c1".into(),
        delta: "{\"id\":1}".into(),
      },
      Chunk::ToolCallAvailable(call),
      Chunk::ToolCallEnd(result),
      Chunk::ToolCallStart(ToolDetails {
        id: "c2".into(),
        name: "web_search".into(),
      }),
      Chunk::ToolCallAvailable(search_call),
      Chunk::ToolCallEnd(search_result),
      Chunk::TextStart,
      Chunk::TextDelta("धर्म".into()),
      Chunk::TextEnd,
    ];
    // Records are pushed by the model before its batch; emulate both up front.
    for r in [first, second] {
      ReplyMeter::push(&meter, r);
    }
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let reply = encode(
      stream::iter(chunks),
      "m1",
      AgentModel::GptLuna,
      meter,
      CancellationToken::new(),
      tx,
    )
    .await;
    let types: Vec<String> = frames(&mut rx)
      .iter()
      .map(|f| f["type"].as_str().unwrap().to_string())
      .collect();
    assert_eq!(
      types,
      [
        "start",
        "start-step",
        "reasoning-start",
        "reasoning-delta",
        "reasoning-end",
        "tool-input-start",
        "tool-input-delta",
        "tool-input-available",
        "tool-output-available",
        "finish-step",
        "message-metadata",
        "start-step",
        "tool-input-start",
        "tool-input-available",
        "tool-output-available",
        "finish-step",
        "message-metadata",
        "start-step",
        "text-start",
        "text-delta",
        "text-end",
        "finish-step",
        "message-metadata",
        "finish",
      ]
    );
    let part_types: Vec<&str> = reply
      .parts
      .iter()
      .map(|p| p["type"].as_str().unwrap())
      .collect();
    assert_eq!(
      part_types,
      [
        "step-start",
        "reasoning",
        "tool-get_entry",
        "step-start",
        "tool-web_search",
        "step-start",
        "text",
      ]
    );
    assert_eq!(
      reply.parts[1]["providerMetadata"]["openrouter"]["reasoning_items"][0]["signature"],
      "s"
    );
    assert_eq!(reply.parts[2]["state"], "output-available");
    assert_eq!(reply.parts[2]["output"]["id"], 1);
    assert_eq!(reply.parts[4]["state"], "output-available");
    assert_eq!(
      reply.parts[4]["output"]["summary"],
      "Sāyaṇa lived in the 14th century."
    );
    assert_eq!(reply.parts[6]["text"], "धर्म");
    assert!(reply.parts.iter().all(|p| p.get("_id").is_none()));
    assert!((reply.metadata["cost"].as_f64().unwrap() - 0.03).abs() < 1e-9);
    assert_eq!(reply.metadata["webSearches"], 1);
  }

  #[test]
  fn tool_errors_are_unwrapped() {
    let out = Ok(Value::String(
      "Error: Tool error: unknown dictionary".into(),
    ));
    assert_eq!(tool_outcome(&out), Err("unknown dictionary".to_string()));
  }
}
