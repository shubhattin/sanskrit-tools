//! OpenRouter model (Responses API, `/api/v1/responses`) for aisdk's tool loop.
//!
//! aisdk's bundled OpenRouter provider drops what this agent needs, so this is a small
//! [`LanguageModel`] of our own:
//! - Reasoning streams as reasoning-text / summary deltas; the finished reasoning items (Anthropic
//!   signatures, OpenAI encrypted content) are replayed as input, which the providers require
//!   between tool-call steps.
//! - `usage.cost` is what OpenRouter actually charged (cache discounts included).
//!
//! What aisdk's stream chunks can't carry (cost, reasoning items) goes to a shared
//! [`ReplyMeter`]: one [`StepRecord`] per model call, in order.

use crate::agent::models::AgentModel;
use aisdk::core::capabilities::{ReasoningSupport, TextInputSupport, ToolCallSupport};
use aisdk::core::language_model::{
  LanguageModel, LanguageModelOptions, LanguageModelResponse, LanguageModelResponseContentType,
  LanguageModelStreamChunk, LanguageModelStreamChunkType, ProviderStream, ReasoningEffort, Usage,
};
use aisdk::core::messages::{AssistantMessage, Message};
use aisdk::core::tools::{ToolCallInfo, ToolDetails, ToolResultInfo};
use aisdk::{Error, Result};
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use serde_json::{Map, Value, json};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

const ENDPOINT: &str = "https://openrouter.ai/api/v1/responses";

/// Identifies the model call (step) a reasoning / tool-call message came from, so the items of
/// one step are replayed together. 0 = untagged.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct StepTag(pub u64);

static NEXT_STEP_TAG: AtomicU64 = AtomicU64::new(1);

pub fn new_step_tag() -> StepTag {
  StepTag(NEXT_STEP_TAG.fetch_add(1, Ordering::Relaxed))
}

/// Finished Responses-API reasoning items of one step, replayed as input.
#[derive(Clone, Default, Debug)]
pub struct ReasoningItems(pub Vec<Value>);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StepUsage {
  pub input_tokens: u64,
  pub output_tokens: u64,
  pub cached_input_tokens: u64,
  pub reasoning_tokens: u64,
}

/// What one model call reported besides its stream chunks.
#[derive(Clone, Debug, Default)]
pub struct StepRecord {
  /// USD charged by OpenRouter for the call.
  pub cost: f64,
  pub usage: StepUsage,
  pub reasoning_items: Vec<Value>,
}

/// Step records of one reply, in call order; the UI stream drains them at step boundaries.
#[derive(Debug, Default)]
pub struct ReplyMeter {
  steps: VecDeque<StepRecord>,
}

pub type SharedMeter = Arc<Mutex<ReplyMeter>>;

impl ReplyMeter {
  pub fn pop(meter: &SharedMeter) -> Option<StepRecord> {
    meter
      .lock()
      .unwrap_or_else(|p| p.into_inner())
      .steps
      .pop_front()
  }

  pub(crate) fn push(meter: &SharedMeter, record: StepRecord) {
    meter
      .lock()
      .unwrap_or_else(|p| p.into_inner())
      .steps
      .push_back(record);
  }
}

/// One OpenRouter model with its tools, for one reply.
#[derive(Clone, Debug)]
pub struct OpenRouterChat {
  http: reqwest::Client,
  api_key: Arc<str>,
  model: AgentModel,
  /// Function tools in Responses format (aisdk keeps its tool list private).
  tools: Arc<Vec<Value>>,
  /// Function tool calls allowed per user turn; past it the model must answer.
  max_tool_calls: usize,
  cancel: CancellationToken,
  meter: SharedMeter,
  /// Sticky-routing key (the chat id).
  session: Option<Arc<str>>,
}

impl ToolCallSupport for OpenRouterChat {}
impl TextInputSupport for OpenRouterChat {}
impl ReasoningSupport for OpenRouterChat {}

impl OpenRouterChat {
  pub fn new(http: reqwest::Client, api_key: Arc<str>, model: AgentModel) -> Self {
    OpenRouterChat {
      http,
      api_key,
      model,
      tools: Arc::new(Vec::new()),
      max_tool_calls: 0,
      cancel: CancellationToken::new(),
      meter: SharedMeter::default(),
      session: None,
    }
  }

  pub fn with_session(mut self, session: &str) -> Self {
    self.session = Some(session.chars().take(256).collect::<String>().into());
    self
  }

  /// Function tools (`{ type: 'function', name, description, parameters }`).
  pub fn with_tools(mut self, tools: Vec<Value>, max_tool_calls: usize) -> Self {
    self.tools = Arc::new(tools);
    self.max_tool_calls = max_tool_calls;
    self
  }

  pub fn with_cancel(mut self, cancel: CancellationToken) -> Self {
    self.cancel = cancel;
    self
  }

  pub fn meter(&self) -> SharedMeter {
    self.meter.clone()
  }

  fn request_body(&self, options: &LanguageModelOptions) -> Value {
    let messages = options.messages();
    let mut input = Vec::new();
    if let Some(system) = options.system.as_deref().filter(|s| !s.is_empty()) {
      input.push(json!({
        "type": "message",
        "role": "system",
        "content": [{ "type": "input_text", "text": system }],
      }));
    }
    input.extend(input_items(&messages));

    let mut body = Map::new();
    body.insert("model".into(), self.model.provider_id().into());
    // Anthropic caches only when asked; the top-level form caches everything up to the latest
    // input and moves forward each step / turn. OpenAI models cache on their own.
    if self.model.needs_cache_control() {
      body.insert("cache_control".into(), json!({ "type": "ephemeral" }));
    }
    // Keeps a chat on one provider endpoint, where its cache lives.
    if let Some(session) = &self.session {
      body.insert("session_id".into(), session.as_ref().into());
    }
    body.insert("input".into(), Value::Array(input));
    body.insert("stream".into(), true.into());
    let tools = self.tools.as_ref().clone();
    if !tools.is_empty() {
      let budget_left = tool_calls_this_turn(&messages) < self.max_tool_calls;
      body.insert("tools".into(), Value::Array(tools));
      body.insert(
        "tool_choice".into(),
        if budget_left { "auto" } else { "none" }.into(),
      );
    }
    // No `reasoning` field: both models reason at their default effort. Only explicit
    // requests (chat titles) set one.
    if let Some(effort) = options.reasoning_effort {
      let effort = match effort {
        ReasoningEffort::None => "none",
        ReasoningEffort::Low => "low",
        ReasoningEffort::Medium => "medium",
        ReasoningEffort::High => "high",
        ReasoningEffort::XHigh => "xhigh",
      };
      body.insert("reasoning".into(), json!({ "effort": effort }));
    }
    if let Some(max) = options.max_output_tokens {
      body.insert("max_output_tokens".into(), max.into());
    }
    Value::Object(body)
  }

  /// POST with a short retry on rate limits, 5xx and connection errors.
  async fn send(&self, body: &Value) -> Result<reqwest::Response> {
    let mut attempt = 0u32;
    loop {
      let sent = self
        .http
        .post(ENDPOINT)
        .bearer_auth(self.api_key.as_ref())
        .header("X-Title", "Sanskrit Dictionary Agent")
        .json(body)
        .send()
        .await;
      let retryable = match &sent {
        Ok(resp) => resp.status().as_u16() == 429 || resp.status().is_server_error(),
        Err(e) => e.is_connect() || e.is_timeout(),
      };
      if retryable && attempt < 2 {
        attempt += 1;
        tokio::select! {
          _ = self.cancel.cancelled() => return Err(cancelled()),
          _ = tokio::time::sleep(Duration::from_millis(800 * u64::from(attempt))) => {}
        }
        continue;
      }
      let resp = sent.map_err(|e| Error::Other(format!("OpenRouter request failed: {e}")))?;
      if resp.status().is_success() {
        return Ok(resp);
      }
      let status = resp.status();
      let text = resp.text().await.unwrap_or_default();
      let detail = serde_json::from_str::<Value>(&text)
        .ok()
        .and_then(|v| v["error"]["message"].as_str().map(str::to_owned))
        .unwrap_or(text);
      return Err(Error::Other(format!("OpenRouter {status}: {detail}")));
    }
  }
}

fn cancelled() -> Error {
  Error::Other("Stopped".into())
}

#[async_trait]
impl LanguageModel for OpenRouterChat {
  fn name(&self) -> String {
    self.model.provider_id().to_string()
  }

  /// Collects [`Self::stream_text`] (used for chat titles).
  async fn generate_text(
    &mut self,
    options: LanguageModelOptions,
  ) -> Result<LanguageModelResponse> {
    let mut stream = self.stream_text(options).await?;
    let mut contents = Vec::new();
    let mut usage = None;
    while let Some(batch) = stream.next().await {
      for chunk in batch? {
        match chunk {
          LanguageModelStreamChunk::Done(msg) => {
            usage = msg.usage.clone().or(usage);
            contents.push(msg.content);
          }
          LanguageModelStreamChunk::Delta(LanguageModelStreamChunkType::Failed(e)) => {
            return Err(Error::Other(e));
          }
          LanguageModelStreamChunk::Delta(_) => {}
        }
      }
    }
    Ok(LanguageModelResponse { contents, usage })
  }

  async fn stream_text(&mut self, options: LanguageModelOptions) -> Result<ProviderStream> {
    if self.cancel.is_cancelled() {
      return Err(cancelled());
    }
    let body = self.request_body(&options);
    let resp = self.send(&body).await?;
    let (tx, rx) = futures_channel::mpsc::unbounded();
    let cancel = self.cancel.clone();
    let meter = self.meter.clone();
    tokio::spawn(async move {
      let mut step = StepStream::new(tx);
      let mut events = resp.bytes_stream().eventsource();
      loop {
        let event = tokio::select! {
          _ = cancel.cancelled() => return,
          event = events.next() => event,
        };
        match event {
          None => break,
          Some(Err(e)) => return step.fail(format!("OpenRouter stream error: {e}")),
          Some(Ok(event)) if event.data.trim() == "[DONE]" => break,
          Some(Ok(event)) => match serde_json::from_str::<Value>(&event.data) {
            Ok(event) => {
              if let Err(message) = step.event(&event) {
                return step.fail(message);
              }
            }
            Err(e) => return step.fail(format!("Unreadable OpenRouter event: {e}")),
          },
        }
      }
      step.finish(&meter);
    });
    Ok(Box::pin(rx))
  }
}

type ChunkSender = futures_channel::mpsc::UnboundedSender<Result<Vec<LanguageModelStreamChunk>>>;

// ---------------------------------------------------------------------------------------------
// Response events → aisdk chunks
// ---------------------------------------------------------------------------------------------

#[derive(PartialEq, Clone, Copy)]
enum Block {
  None,
  Reasoning,
  Text,
}

struct PendingCall {
  item_id: String,
  call_id: String,
  name: String,
  args: String,
}

/// Turns one model call's events into aisdk deltas, then one batch of `Done` messages.
struct StepStream {
  tx: ChunkSender,
  tag: StepTag,
  block: Block,
  reasoning: String,
  reasoning_items: Vec<Value>,
  text: String,
  calls: Vec<PendingCall>,
  usage: Value,
  finished: bool,
}

impl StepStream {
  fn new(tx: ChunkSender) -> Self {
    StepStream {
      tx,
      tag: new_step_tag(),
      block: Block::None,
      reasoning: String::new(),
      reasoning_items: Vec::new(),
      text: String::new(),
      calls: Vec::new(),
      usage: Value::Null,
      finished: false,
    }
  }

  fn send(&self, chunks: Vec<LanguageModelStreamChunk>) {
    if !chunks.is_empty() {
      let _ = self.tx.unbounded_send(Ok(chunks));
    }
  }

  fn fail(&self, message: String) {
    self.send(vec![delta(LanguageModelStreamChunkType::Failed(message))]);
  }

  /// Close the open reasoning/text block and open `next`.
  fn switch_block(&mut self, next: Block, out: &mut Vec<LanguageModelStreamChunk>) {
    if self.block == next {
      return;
    }
    match self.block {
      Block::Reasoning => out.push(delta(LanguageModelStreamChunkType::ReasoningEnd)),
      Block::Text => out.push(delta(LanguageModelStreamChunkType::TextEnd)),
      Block::None => {}
    }
    match next {
      Block::Reasoning => out.push(delta(LanguageModelStreamChunkType::ReasoningStart)),
      Block::Text => out.push(delta(LanguageModelStreamChunkType::TextStart)),
      Block::None => {}
    }
    self.block = next;
  }

  fn event(&mut self, event: &Value) -> std::result::Result<(), String> {
    let mut out = Vec::new();
    match event["type"].as_str().unwrap_or("") {
      "response.reasoning_text.delta" | "response.reasoning_summary_text.delta" => {
        if let Some(d) = event["delta"].as_str().filter(|d| !d.is_empty()) {
          self.switch_block(Block::Reasoning, &mut out);
          self.reasoning.push_str(d);
          out.push(delta(LanguageModelStreamChunkType::ReasoningDelta(
            d.to_string(),
          )));
        }
      }
      // Separate summary paragraphs of one reasoning item.
      "response.reasoning_summary_part.added" if !self.reasoning.is_empty() => {
        self.switch_block(Block::Reasoning, &mut out);
        self.reasoning.push_str("\n\n");
        out.push(delta(LanguageModelStreamChunkType::ReasoningDelta(
          "\n\n".into(),
        )));
      }
      "response.output_text.delta" => {
        if let Some(d) = event["delta"].as_str().filter(|d| !d.is_empty()) {
          self.switch_block(Block::Text, &mut out);
          self.text.push_str(d);
          out.push(delta(LanguageModelStreamChunkType::TextDelta(
            d.to_string(),
          )));
        }
      }
      "response.output_item.added" => {
        let item = &event["item"];
        if item["type"] == "function_call" {
          self.switch_block(Block::None, &mut out);
          let call = PendingCall {
            item_id: item["id"].as_str().unwrap_or_default().to_string(),
            call_id: item["call_id"].as_str().unwrap_or_default().to_string(),
            name: item["name"].as_str().unwrap_or_default().to_string(),
            args: String::new(),
          };
          out.push(delta(LanguageModelStreamChunkType::ToolCallStart(
            ToolDetails {
              id: call.call_id.clone(),
              name: call.name.clone(),
            },
          )));
          self.calls.push(call);
        }
      }
      "response.function_call_arguments.delta" => {
        let item_id = event["item_id"].as_str().unwrap_or_default();
        if let (Some(call), Some(d)) = (
          self.calls.iter_mut().find(|c| c.item_id == item_id),
          event["delta"].as_str().filter(|d| !d.is_empty()),
        ) {
          call.args.push_str(d);
          out.push(delta(LanguageModelStreamChunkType::ToolCallDelta {
            id: call.call_id.clone(),
            delta: d.to_string(),
          }));
        }
      }
      "response.output_item.done" => {
        let item = &event["item"];
        match item["type"].as_str().unwrap_or("") {
          "reasoning" => {
            if self.block == Block::Reasoning {
              self.switch_block(Block::None, &mut out);
            }
            self.reasoning_items.push(item.clone());
          }
          "message" => {
            if self.block == Block::Text {
              self.switch_block(Block::None, &mut out);
            }
          }
          "function_call" => {
            let item_id = item["id"].as_str().unwrap_or_default();
            if let Some(call) = self.calls.iter_mut().find(|c| c.item_id == item_id)
              && let Some(args) = item["arguments"].as_str()
            {
              call.args = args.to_string();
            }
          }
          _ => {}
        }
      }
      "response.completed" | "response.incomplete" => {
        self.usage = event["response"]["usage"].clone();
        self.finished = true;
      }
      "response.failed" => {
        return Err(
          event["response"]["error"]["message"]
            .as_str()
            .unwrap_or("The model's response failed.")
            .to_string(),
        );
      }
      "error" => {
        return Err(
          event["message"]
            .as_str()
            .or_else(|| event["error"]["message"].as_str())
            .unwrap_or("OpenRouter reported an error.")
            .to_string(),
        );
      }
      _ => {}
    }
    self.send(out);
    Ok(())
  }

  fn finish(mut self, meter: &SharedMeter) {
    if !self.finished {
      return self.fail("The model's response ended unexpectedly.".into());
    }
    let mut out = Vec::new();
    self.switch_block(Block::None, &mut out);

    let u = &self.usage;
    let n = |v: &Value| v.as_u64().unwrap_or(0);
    let step_usage = StepUsage {
      input_tokens: n(&u["input_tokens"]),
      output_tokens: n(&u["output_tokens"]),
      cached_input_tokens: n(&u["input_tokens_details"]["cached_tokens"]),
      reasoning_tokens: n(&u["output_tokens_details"]["reasoning_tokens"]),
    };
    // Recorded before the `Done` batch is sent, so the record exists by the time the UI stream
    // sees this step end.
    ReplyMeter::push(
      meter,
      StepRecord {
        cost: u["cost"].as_f64().unwrap_or(0.0),
        usage: step_usage,
        reasoning_items: self.reasoning_items.clone(),
      },
    );
    let sdk_usage = Some(Usage {
      input_tokens: Some(step_usage.input_tokens as usize),
      output_tokens: Some(step_usage.output_tokens as usize),
      reasoning_tokens: Some(step_usage.reasoning_tokens as usize),
      cached_tokens: Some(step_usage.cached_input_tokens as usize),
    });

    // All `Done` messages of the step in one batch: aisdk then runs the tool calls and continues
    // instead of treating the text/reasoning as the final answer.
    if !self.reasoning.is_empty() || !self.reasoning_items.is_empty() {
      let extensions = aisdk::extensions::Extensions::default();
      extensions.insert(self.tag);
      extensions.insert(ReasoningItems(std::mem::take(&mut self.reasoning_items)));
      out.push(done(
        LanguageModelResponseContentType::Reasoning {
          content: std::mem::take(&mut self.reasoning),
          extensions,
        },
        sdk_usage.clone(),
      ));
    }
    if !self.text.is_empty() || self.calls.is_empty() {
      out.push(done(
        LanguageModelResponseContentType::Text(std::mem::take(&mut self.text)),
        sdk_usage.clone(),
      ));
    }
    for call in std::mem::take(&mut self.calls) {
      let mut info = ToolCallInfo::new(call.name);
      info.id(call.call_id);
      info.input(parse_arguments(&call.args));
      info.extensions.insert(self.tag);
      out.push(done(
        LanguageModelResponseContentType::ToolCall(info),
        sdk_usage.clone(),
      ));
    }
    self.send(out);
  }
}

fn delta(chunk: LanguageModelStreamChunkType) -> LanguageModelStreamChunk {
  LanguageModelStreamChunk::Delta(chunk)
}

fn done(
  content: LanguageModelResponseContentType,
  usage: Option<Usage>,
) -> LanguageModelStreamChunk {
  LanguageModelStreamChunk::Done(AssistantMessage { content, usage })
}

/// Tool arguments as JSON; malformed or empty arguments become `{}` (the tool then reports the
/// missing fields to the model).
fn parse_arguments(args: &str) -> Value {
  match serde_json::from_str::<Value>(args) {
    Ok(v @ Value::Object(_)) => v,
    _ => Value::Object(Map::new()),
  }
}

// ---------------------------------------------------------------------------------------------
// aisdk messages → Responses input items
// ---------------------------------------------------------------------------------------------

/// The items of one model call, plus the outputs answering its function calls.
#[derive(Default)]
struct StepGroup {
  tag: u64,
  reasoning_items: Vec<Value>,
  text: String,
  calls: Vec<Value>,
  outputs: Vec<Value>,
}

impl StepGroup {
  /// Whether a message of step `tag` (0 = unknown) belongs here. aisdk runs a step's calls one
  /// by one (call, result, call, result), so a tagged call joins its step even after results;
  /// anything untagged after results is the next model call.
  fn accepts(&self, tag: u64) -> bool {
    (tag != 0 && self.tag == tag) || (self.outputs.is_empty() && (tag == 0 || self.tag == 0))
  }

  /// Reasoning, then the assistant message, then all calls, then all their outputs.
  fn flush_into(self, out: &mut Vec<Value>) {
    out.extend(self.reasoning_items);
    if !self.text.is_empty() {
      out.push(json!({
        "type": "message",
        "role": "assistant",
        "content": [{ "type": "output_text", "text": self.text, "annotations": [] }],
      }));
    }
    out.extend(self.calls);
    out.extend(self.outputs);
  }
}

fn tool_result_content(result: &ToolResultInfo) -> String {
  match &result.output {
    Ok(Value::String(s)) => s.clone(),
    Ok(other) => other.to_string(),
    Err(e) => format!("Error: {e}"),
  }
}

/// aisdk keeps every reasoning block, text and tool call as its own assistant message. They are
/// regrouped per model call (by [`StepTag`]; text carries none and joins the open call) so each
/// call's reasoning precedes its message and calls, as the providers expect.
pub fn input_items(messages: &[Message]) -> Vec<Value> {
  let mut out = Vec::new();
  let mut group: Option<StepGroup> = None;

  fn open<'a>(
    group: &'a mut Option<StepGroup>,
    out: &mut Vec<Value>,
    tag: u64,
  ) -> &'a mut StepGroup {
    if !group.as_ref().is_some_and(|g| g.accepts(tag)) {
      if let Some(g) = group.take() {
        g.flush_into(out);
      }
      *group = Some(StepGroup::default());
    }
    let g = group.as_mut().expect("group was just opened");
    if g.tag == 0 {
      g.tag = tag;
    }
    g
  }

  for message in messages {
    match message {
      Message::System(_) => {}
      Message::User(user) => {
        if let Some(g) = group.take() {
          g.flush_into(&mut out);
        }
        out.push(json!({
          "type": "message",
          "role": "user",
          "content": [{ "type": "input_text", "text": user.content }],
        }));
      }
      Message::Developer(text) => {
        if let Some(g) = group.take() {
          g.flush_into(&mut out);
        }
        out.push(json!({
          "type": "message",
          "role": "developer",
          "content": [{ "type": "input_text", "text": text }],
        }));
      }
      Message::Assistant(assistant) => match &assistant.content {
        LanguageModelResponseContentType::Text(text) => {
          if !text.is_empty() {
            open(&mut group, &mut out, 0).text.push_str(text);
          }
        }
        LanguageModelResponseContentType::Reasoning { extensions, .. } => {
          let tag = extensions.get::<StepTag>().0;
          let items = extensions.get::<ReasoningItems>().0.clone();
          open(&mut group, &mut out, tag)
            .reasoning_items
            .extend(items);
        }
        LanguageModelResponseContentType::ToolCall(info) => {
          let tag = info.extensions.get::<StepTag>().0;
          open(&mut group, &mut out, tag).calls.push(json!({
            "type": "function_call",
            "call_id": info.tool.id,
            "name": info.tool.name,
            "arguments": info.input.to_string(),
          }));
        }
        LanguageModelResponseContentType::NotSupported(_) => {}
      },
      Message::Tool(result) => {
        let item = json!({
          "type": "function_call_output",
          "call_id": result.tool.id,
          "output": tool_result_content(result),
        });
        match group.as_mut() {
          Some(g) => g.outputs.push(item),
          None => out.push(item),
        }
      }
    }
  }
  if let Some(g) = group.take() {
    g.flush_into(&mut out);
  }
  out
}

/// Function tool calls made since the latest user message.
fn tool_calls_this_turn(messages: &[Message]) -> usize {
  messages
    .iter()
    .rev()
    .take_while(|m| !matches!(m, Message::User(_)))
    .filter(|m| {
      matches!(m, Message::Assistant(a)
        if matches!(a.content, LanguageModelResponseContentType::ToolCall(_)))
    })
    .count()
}

#[cfg(test)]
mod tests {
  use super::*;
  use aisdk::core::messages::UserMessage;

  fn reasoning(tag: StepTag, items: Vec<Value>) -> Message {
    let extensions = aisdk::extensions::Extensions::default();
    extensions.insert(tag);
    extensions.insert(ReasoningItems(items));
    Message::Assistant(AssistantMessage::new(
      LanguageModelResponseContentType::Reasoning {
        content: "thinking".into(),
        extensions,
      },
      None,
    ))
  }

  fn call(tag: StepTag, id: &str) -> Message {
    let mut info = ToolCallInfo::new("search_dictionaries");
    info.id(id);
    info.input(json!({ "query": "dharma" }));
    info.extensions.insert(tag);
    Message::Assistant(AssistantMessage::new(
      LanguageModelResponseContentType::ToolCall(info),
      None,
    ))
  }

  fn result(id: &str) -> Message {
    let mut r = ToolResultInfo::new("search_dictionaries");
    r.id(id);
    r.output(Value::String("{\"ok\":true}".into()));
    Message::Tool(r)
  }

  fn text(t: &str) -> Message {
    Message::Assistant(AssistantMessage::new(
      LanguageModelResponseContentType::Text(t.into()),
      None,
    ))
  }

  fn kinds(items: &[Value]) -> Vec<String> {
    items
      .iter()
      .map(|i| match i["type"].as_str().unwrap() {
        "message" => format!("message:{}", i["role"].as_str().unwrap()),
        t => t.to_string(),
      })
      .collect()
  }

  #[test]
  fn regroups_each_step() {
    let a = new_step_tag();
    let b = new_step_tag();
    let items = input_items(&[
      Message::User(UserMessage::new("hi")),
      reasoning(a, vec![json!({ "type": "reasoning", "signature": "s" })]),
      text("Let me look."),
      call(a, "c1"),
      result("c1"),
      call(a, "c2"),
      result("c2"),
      reasoning(b, vec![]),
      call(b, "c3"),
      result("c3"),
      text("Done."),
    ]);
    assert_eq!(
      kinds(&items),
      [
        "message:user",
        "reasoning",
        "message:assistant",
        "function_call",
        "function_call",
        "function_call_output",
        "function_call_output",
        "function_call",
        "function_call_output",
        "message:assistant",
      ]
    );
    assert_eq!(items[1]["signature"], "s");
    assert_eq!(items[2]["content"][0]["text"], "Let me look.");
    assert_eq!(items[3]["call_id"], "c1");
    assert_eq!(items[5]["output"], "{\"ok\":true}");
    assert_eq!(items[9]["content"][0]["text"], "Done.");
  }

  #[test]
  fn counts_tool_calls_of_the_current_turn_only() {
    let a = new_step_tag();
    let messages = vec![
      Message::User(UserMessage::new("one")),
      call(a, "c1"),
      result("c1"),
      text("ok"),
      Message::User(UserMessage::new("two")),
      call(a, "c2"),
      result("c2"),
    ];
    assert_eq!(tool_calls_this_turn(&messages), 1);
  }

  fn run(events: &[Value]) -> Vec<LanguageModelStreamChunk> {
    let (tx, mut rx) = futures_channel::mpsc::unbounded();
    let mut step = StepStream::new(tx);
    for e in events {
      step.event(e).unwrap();
    }
    step.finish(&SharedMeter::default());
    let mut out = Vec::new();
    while let Ok(batch) = rx.try_recv() {
      out.extend(batch.unwrap());
    }
    out
  }

  #[test]
  fn streams_calls_and_reasoning() {
    let chunks = run(&[
      json!({ "type": "response.output_item.added", "item": { "type": "reasoning", "id": "rs1" } }),
      json!({ "type": "response.reasoning_text.delta", "delta": "hmm" }),
      json!({ "type": "response.output_item.done",
              "item": { "type": "reasoning", "id": "rs1", "signature": "sig" } }),
      json!({ "type": "response.output_item.added",
              "item": { "type": "function_call", "id": "fc1", "call_id": "c1", "name": "get_entry" } }),
      json!({ "type": "response.function_call_arguments.delta", "item_id": "fc1", "delta": "{\"id\":" }),
      json!({ "type": "response.function_call_arguments.delta", "item_id": "fc1", "delta": "1}" }),
      json!({ "type": "response.output_item.done",
              "item": { "type": "function_call", "id": "fc1", "call_id": "c1", "name": "get_entry",
                        "arguments": "{\"id\":1}" } }),
      json!({ "type": "response.completed",
              "response": { "usage": { "input_tokens": 10, "output_tokens": 2, "cost": 0.5 } } }),
    ]);
    let names: Vec<String> = chunks
      .iter()
      .map(|c| match c {
        LanguageModelStreamChunk::Delta(d) => match d {
          LanguageModelStreamChunkType::ReasoningStart => "reasoning-start".into(),
          LanguageModelStreamChunkType::ReasoningDelta(_) => "reasoning-delta".into(),
          LanguageModelStreamChunkType::ReasoningEnd => "reasoning-end".into(),
          LanguageModelStreamChunkType::ToolCallStart(t) => format!("start:{}", t.name),
          LanguageModelStreamChunkType::ToolCallDelta { .. } => "args".into(),
          LanguageModelStreamChunkType::ToolCallAvailable(t) => format!("input:{}", t.tool.name),
          LanguageModelStreamChunkType::ToolCallEnd(t) => format!("output:{}", t.tool.name),
          other => format!("{other:?}"),
        },
        LanguageModelStreamChunk::Done(m) => match &m.content {
          LanguageModelResponseContentType::Reasoning { extensions, .. } => {
            format!(
              "done:reasoning:{}",
              extensions.get::<ReasoningItems>().0.len()
            )
          }
          LanguageModelResponseContentType::ToolCall(t) => {
            format!("done:call:{}", t.input)
          }
          other => format!("done:{other:?}"),
        },
      })
      .collect();
    assert_eq!(
      names,
      [
        "reasoning-start",
        "reasoning-delta",
        "reasoning-end",
        "start:get_entry",
        "args",
        "args",
        "done:reasoning:1",
        "done:call:{\"id\":1}",
      ]
    );
  }
}
