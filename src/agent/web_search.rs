//! Web search as an ordinary function tool, through OpenAI's Responses API with its native
//! `web_search` tool (the `OPENAI_API_KEY` key). The main chat models stay on OpenRouter; only
//! the search call goes to OpenAI directly, so it works the same for both models.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;

/// Direct OpenAI model answering the search (the default chat model, via `OPENAI_API_KEY`).
pub const SEARCH_MODEL: &str = "gpt-6-luna";

/// Tool name in the UI stream, stored messages and history (a regular function tool now).
pub const WEB_SEARCH_TOOL: &str = "web_search";

const ENDPOINT: &str = "https://api.openai.com/v1/responses";
const MAX_OUTPUT_TOKENS: u32 = 1_500;
const TIMEOUT: Duration = Duration::from_secs(120);

const INSTRUCTIONS: &str = "Search the web and report what you find: key facts, names and dates, concisely, in Markdown. Note disagreements between sources. Do not speculate beyond the sources.";

#[derive(Clone, Debug)]
pub struct WebSearch {
  http: reqwest::Client,
  key: Arc<str>,
}

impl WebSearch {
  pub fn new(http: reqwest::Client, key: Arc<str>) -> Self {
    WebSearch { http, key }
  }

  /// Ask a specific question; returns a factual summary with the source URLs.
  pub async fn search(&self, query: &str) -> Result<WebSearchOutput, String> {
    let query: String = query.chars().take(500).collect();
    if query.trim().is_empty() {
      return Err("give a search query".into());
    }
    let resp = self
      .http
      .post(ENDPOINT)
      .bearer_auth(self.key.as_ref())
      .json(&json!({
        "model": SEARCH_MODEL,
        "instructions": INSTRUCTIONS,
        "input": query,
        "tools": [{ "type": "web_search" }],
        "max_output_tokens": MAX_OUTPUT_TOKENS,
        // One-shot call: don't retain it on OpenAI's side.
        "store": false,
      }))
      .timeout(TIMEOUT)
      .send()
      .await
      .map_err(|e| format!("web search request failed: {e}"))?;
    if !resp.status().is_success() {
      let status = resp.status();
      let text = resp.text().await.unwrap_or_default();
      let detail = serde_json::from_str::<Value>(&text)
        .ok()
        .and_then(|v| v["error"]["message"].as_str().map(str::to_owned))
        .unwrap_or(text);
      return Err(format!("web search failed (OpenAI {status}): {detail}"));
    }
    let body: Value = resp
      .json()
      .await
      .map_err(|e| format!("web search returned an unreadable response: {e}"))?;
    parse_response(&body)
  }
}

#[derive(Deserialize, JsonSchema)]
pub struct WebSearchInput {
  /// What to look up, as a specific question or search query.
  pub query: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct WebSource {
  pub url: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub title: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct WebSearchOutput {
  pub summary: String,
  pub sources: Vec<WebSource>,
}

/// Summary text plus `url_citation` annotations of a completed Responses-API call.
fn parse_response(body: &Value) -> Result<WebSearchOutput, String> {
  if body["status"] != "completed" {
    return Err(format!(
      "web search did not complete ({}).",
      body["status"].as_str().unwrap_or("unknown")
    ));
  }
  let mut summary = String::new();
  let mut sources = Vec::new();
  for item in body["output"].as_array().into_iter().flatten() {
    if item["type"] != "message" {
      continue;
    }
    for content in item["content"].as_array().into_iter().flatten() {
      if content["type"] != "output_text" {
        continue;
      }
      if let Some(text) = content["text"].as_str().filter(|t| !t.is_empty()) {
        if !summary.is_empty() {
          summary.push_str("\n\n");
        }
        summary.push_str(text);
      }
      for annotation in content["annotations"].as_array().into_iter().flatten() {
        if annotation["type"] != "url_citation" {
          continue;
        }
        let Some(url) = annotation["url"].as_str() else {
          continue;
        };
        if sources.iter().any(|s: &WebSource| s.url == url) {
          continue;
        }
        sources.push(WebSource {
          url: url.to_string(),
          title: annotation["title"]
            .as_str()
            .filter(|t| !t.is_empty())
            .map(str::to_owned),
        });
      }
    }
  }
  if summary.trim().is_empty() {
    return Err("web search returned no answer.".into());
  }
  Ok(WebSearchOutput { summary, sources })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_summary_and_citations() {
    let body = json!({
      "status": "completed",
      "output": [
        { "type": "reasoning", "encrypted_content": "x" },
        { "type": "web_search_call", "id": "ws1", "status": "completed",
          "action": { "query": "q" } },
        { "type": "message", "content": [
          { "type": "output_text", "text": "Sāyaṇa wrote it.", "annotations": [
            { "type": "url_citation", "url": "https://a.org", "title": "A" },
            { "type": "url_citation", "url": "https://a.org", "title": "A again" },
            { "type": "url_citation", "url": "https://b.org" },
          ] },
        ] },
      ],
    });
    let out = parse_response(&body).unwrap();
    assert_eq!(out.summary, "Sāyaṇa wrote it.");
    assert_eq!(out.sources.len(), 2);
    assert_eq!(out.sources[0].url, "https://a.org");
    assert_eq!(out.sources[0].title.as_deref(), Some("A"));
    assert_eq!(out.sources[1].title, None);
  }

  #[test]
  fn rejects_incomplete_or_empty_searches() {
    assert!(parse_response(&json!({ "status": "incomplete", "output": [] })).is_err());
    assert!(
      parse_response(&json!({ "status": "completed", "output": [
        { "type": "message", "content": [] },
      ] }))
      .is_err()
    );
  }
}
