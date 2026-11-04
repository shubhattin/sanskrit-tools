//! Jev (TypeSafe's decision model) through OpenRouter's Decisions API
//! (`POST /api/alpha/decisions`): given a state and labelled options, it returns the chosen label
//! with a probability per option. Output tokens are free and input costs ~$0.04 / M tokens, so the
//! vidyut tools use it publicly to pick among the candidates the grammar leaves open (sandhi
//! readings, meters, spelling variants). The server always builds the options itself.

use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::{LazyLock, Mutex};
use utoipa::ToSchema;

const URL: &str = "https://openrouter.ai/api/alpha/decisions";
const MODEL: &str = "typesafe/jev-1.14";
/// Options per question / questions per request (cost and answer quality).
pub const MAX_OPTIONS: usize = 8;
const MAX_QUESTIONS: usize = 8;
const CACHE_MAX: usize = 4_000;

/// Jev's answer to one question: the chosen option (index into the options given) and how sure.
#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct JevPick {
  /// The options as they were put to Jev.
  pub options: Vec<String>,
  pub choice: usize,
  /// Probability per option (same order).
  pub probabilities: Vec<f64>,
  pub confidence: Option<f64>,
}

pub struct Question {
  pub name: String,
  pub instructions: String,
  pub options: Vec<String>,
}

#[derive(Debug)]
pub enum JevError {
  /// No `OPENROUTER_API_KEY`.
  NotConfigured,
  Upstream(String),
}

/// Answers are deterministic enough to reuse: same state + questions → cached answer.
static CACHE: LazyLock<Mutex<HashMap<u64, HashMap<String, JevPick>>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

fn key() -> Option<String> {
  std::env::var("OPENROUTER_API_KEY")
    .ok()
    .map(|k| k.trim().to_string())
    .filter(|k| !k.is_empty())
}

pub fn configured() -> bool {
  key().is_some()
}

/// Most questions answered per call (larger sets go in parallel batches of [`MAX_QUESTIONS`]).
const MAX_TOTAL: usize = 48;

/// Asks Jev to choose an option for each question (questions with fewer than two options are
/// skipped). Returns picks by question name.
pub async fn choose(
  http: &reqwest::Client,
  state: Value,
  questions: Vec<Question>,
) -> Result<HashMap<String, JevPick>, JevError> {
  let mut questions: Vec<Question> = questions
    .into_iter()
    .filter(|q| q.options.len() >= 2)
    .take(MAX_TOTAL)
    .map(|mut q| {
      q.options.truncate(MAX_OPTIONS);
      q
    })
    .collect();
  if questions.len() <= MAX_QUESTIONS {
    return choose_batch(http, &state, questions).await;
  }
  let mut batches = Vec::new();
  while !questions.is_empty() {
    let rest = questions.split_off(questions.len().min(MAX_QUESTIONS));
    batches.push(std::mem::replace(&mut questions, rest));
  }
  let answers =
    futures_util::future::join_all(batches.into_iter().map(|b| choose_batch(http, &state, b)))
      .await;
  let mut out = HashMap::new();
  for a in answers {
    out.extend(a?);
  }
  Ok(out)
}

async fn choose_batch(
  http: &reqwest::Client,
  state: &Value,
  questions: Vec<Question>,
) -> Result<HashMap<String, JevPick>, JevError> {
  if questions.is_empty() {
    return Ok(HashMap::new());
  }
  let api_key = key().ok_or(JevError::NotConfigured)?;
  let qs: serde_json::Map<String, Value> = questions
    .iter()
    .map(|q| {
      // Labels are indexes; the option text is the guidance for that label.
      let criteria: serde_json::Map<String, Value> = q
        .options
        .iter()
        .enumerate()
        .map(|(i, o)| (i.to_string(), Value::String(o.clone())))
        .collect();
      (
        q.name.clone(),
        json!({ "type": "choice", "instructions": q.instructions, "criteria": criteria }),
      )
    })
    .collect();
  let body = json!({ "model": MODEL, "state": state, "questions": qs });

  let mut h = DefaultHasher::new();
  body.to_string().hash(&mut h);
  let cache_key = h.finish();
  if let Some(hit) = CACHE.lock().ok().and_then(|c| c.get(&cache_key).cloned()) {
    return Ok(hit);
  }

  let resp = http
    .post(URL)
    .bearer_auth(api_key)
    .json(&body)
    .send()
    .await
    .map_err(|e| JevError::Upstream(e.to_string()))?;
  let status = resp.status();
  let v: Value = resp
    .json()
    .await
    .map_err(|e| JevError::Upstream(e.to_string()))?;
  if !status.is_success() {
    let msg = v["error"]["message"].as_str().unwrap_or("request failed");
    return Err(JevError::Upstream(format!("{status}: {msg}")));
  }

  let mut out = HashMap::new();
  for q in questions {
    let a = &v["answers"][&q.name];
    let Some(choice) = a["choice"].as_str().and_then(|c| c.parse::<usize>().ok()) else {
      continue;
    };
    if choice >= q.options.len() {
      continue;
    }
    let probabilities = (0..q.options.len())
      .map(|i| a["probabilities"][i.to_string()].as_f64().unwrap_or(0.0))
      .collect();
    out.insert(
      q.name,
      JevPick {
        options: q.options,
        choice,
        probabilities,
        confidence: a["confidence"].as_f64(),
      },
    );
  }
  if let Ok(mut c) = CACHE.lock() {
    if c.len() >= CACHE_MAX {
      c.clear();
    }
    c.insert(cache_key, out.clone());
  }
  Ok(out)
}
