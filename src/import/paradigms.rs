//! Inflection tables → `paradigms` + `paradigm_forms`.
//!
//! Sources:
//! - Sanskrit Heritage (Gérard Huet) StarDict HTML, Devanagari (`san-san`) files:
//!   declensions A1–A5 (nominal stems per gender), B (kṛdanta stems `stem@root`)
//!   and conjugations (`grammar-…`, every derivation / lakāra / voice, plus
//!   kṛdanta and avyaya lists).
//! - JNU tiṅanta TSV (one 3×3 paradigm per root/lakāra; deduplicated).
//!
//! Display structure (`tables` JSON), cells numbered row-major across tables:
//! `{"group"?, "title"?, "label"?, "cols": [..], "rows": [{"label", "cells": [[forms]]}]}`
//! or `{"group"?, "title"?, "list": [{"label", "text", "forms": [..]}]}` (one cell per item).

use super::markup::strip_tags;
use super::stardict;
use crate::translit::{deva_key, nfc};
use anyhow::Result;
use regex::Regex;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

pub struct Paradigm {
  pub lemma: String,
  pub root: Option<String>,
  pub label: Option<String>,
  pub tables: Value,
  /// SLP1 form → cell numbers
  pub forms: Vec<(String, Vec<i16>)>,
}

#[derive(Default)]
struct Builder {
  tables: Vec<Value>,
  forms: HashMap<String, Vec<i16>>,
  order: Vec<String>,
  next_cell: i16,
}

impl Builder {
  fn add_form(&mut self, form: &str, cell: i16) {
    let k = deva_key(form);
    if k.is_empty() {
      return;
    }
    let v = self.forms.entry(k.clone()).or_insert_with(|| {
      self.order.push(k);
      Vec::new()
    });
    if !v.contains(&cell) {
      v.push(cell);
    }
  }

  fn cell(&mut self) -> i16 {
    let c = self.next_cell;
    self.next_cell = self.next_cell.saturating_add(1);
    c
  }

  fn finish(mut self) -> (Value, Vec<(String, Vec<i16>)>) {
    let forms = self
      .order
      .iter()
      .map(|k| (k.clone(), self.forms.remove(k).unwrap_or_default()))
      .collect();
    (Value::Array(self.tables), forms)
  }
}

static TOKEN: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(
    r#"(?s)<table border="1"[^>]*>(?P<table>.*?)</table>|<font color="green"><b>(?P<group>[^<]+)</b></font>|<b>-\s*(?P<tense>[^<]*?)\s*-</b>"#,
  )
  .unwrap()
});
static TR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<tr>(.*?)</tr>").unwrap());
static TD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<td[^>]*>(.*?)</td>").unwrap());
static BR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<br\s*/?>").unwrap());
static LIST_ITEM: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(
    r#"(?s)<font color="blue"><b>([^<]*)</b></font>\s*<br\s*/?>\s*<font color="red">(.*?)</font>"#,
  )
  .unwrap()
});
static HEAD_LABEL: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r#"\[<font color="red">([^<]+)</font>\]"#).unwrap());

fn clean(s: &str) -> String {
  nfc(
    strip_tags(s)
      .replace("&nbsp;", " ")
      .split_whitespace()
      .collect::<Vec<_>>()
      .join(" ")
      .trim(),
  )
}

fn opt(s: &Option<String>) -> Value {
  s.as_ref()
    .map(|x| Value::String(x.clone()))
    .unwrap_or(Value::Null)
}

/// Parse one Heritage payload into display tables + form index.
fn parse_heritage(body: &str) -> (Value, Vec<(String, Vec<i16>)>) {
  let mut b = Builder::default();
  let mut group: Option<String> = None;
  let mut title: Option<String> = None;
  for c in TOKEN.captures_iter(body) {
    if let Some(g) = c.name("group") {
      group = Some(clean(g.as_str()));
      title = None;
    } else if let Some(t) = c.name("tense") {
      title = Some(clean(t.as_str()));
    } else if let Some(t) = c.name("table") {
      let t = t.as_str();
      let rows: Vec<Vec<String>> = TR
        .captures_iter(t)
        .map(|r| TD.captures_iter(&r[1]).map(|d| d[1].to_string()).collect())
        .collect();
      if rows.is_empty() {
        // kṛdanta / avyaya list
        let mut items = Vec::new();
        for it in LIST_ITEM.captures_iter(t) {
          let cell = b.cell();
          let text = clean(&it[2]);
          let forms: Vec<String> = text
            .split_whitespace()
            .filter(|w| crate::translit::has_devanagari(w))
            .map(|w| w.to_string())
            .collect();
          for f in &forms {
            b.add_form(f, cell);
          }
          items.push(json!({ "label": clean(&it[1]), "text": text, "forms": forms }));
        }
        if !items.is_empty() {
          b.tables
            .push(json!({ "group": opt(&group), "title": opt(&title), "list": items }));
        }
        continue;
      }
      let has_cells = t.contains(r#"color="red""#);
      if !has_cells {
        // A one-cell header table (e.g. अप्रत्ययान्तधातु, णिच्)
        if let Some(first) = rows.first().and_then(|r| r.first()) {
          group = Some(clean(first));
          title = None;
        }
        continue;
      }
      let header = &rows[0];
      let label = header.first().map(|h| clean(h));
      let cols: Vec<String> = header.iter().skip(1).map(|h| clean(h)).collect();
      let mut out_rows = Vec::new();
      for r in rows.iter().skip(1) {
        if r.is_empty() {
          continue;
        }
        let row_label = clean(&r[0]);
        let mut cells = Vec::new();
        for d in r.iter().skip(1) {
          let cell = b.cell();
          let forms: Vec<String> = if d.contains(r#"color="red""#) {
            BR.split(d)
              .map(clean)
              .filter(|f| !f.is_empty() && f != "-")
              .collect()
          } else {
            vec![]
          };
          for f in &forms {
            b.add_form(f, cell);
          }
          cells.push(forms);
        }
        out_rows.push(json!({ "label": row_label, "cells": cells }));
      }
      b.tables.push(json!({
        "group": opt(&group), "title": opt(&title), "label": label, "cols": cols, "rows": out_rows
      }));
    }
  }
  b.finish()
}

pub const HERITAGE_FILES: &[(&str, &str)] = &[
  (
    "heritage_decl",
    "Grammar/Declension-heritage_du_sanskrit_san-eng/Declension-A1-heritage_du_sanskrit_san-san",
  ),
  (
    "heritage_decl",
    "Grammar/Declension-heritage_du_sanskrit_san-eng/Declension-A2-heritage_du_sanskrit_san-san",
  ),
  (
    "heritage_decl",
    "Grammar/Declension-heritage_du_sanskrit_san-eng/Declension-A3-heritage_du_sanskrit_san-san",
  ),
  (
    "heritage_decl",
    "Grammar/Declension-heritage_du_sanskrit_san-eng/Declension-A4-heritage_du_sanskrit_san-san",
  ),
  (
    "heritage_decl",
    "Grammar/Declension-heritage_du_sanskrit_san-eng/Declension-A5-heritage_du_sanskrit_san-san",
  ),
  (
    "heritage_kridanta",
    "Grammar/Declension-heritage_du_sanskrit_san-eng/Declension-B-heritage_du_sanskrit_san-san",
  ),
  (
    "heritage_conj",
    "Grammar/Grammar-heritage_du_sanskrit_san-eng/grammar-heritage_du_sanskrit_san-san",
  ),
];

/// Parse one Heritage StarDict file.
pub fn parse_heritage_file(dir: &Path, base: &str, conj: bool) -> Result<Vec<Paradigm>> {
  let sd = stardict::load(&dir.join(base))?;
  Ok(
    sd.entries
      .iter()
      .filter_map(|(w, body)| {
        let (lemma, root) = match w.split_once('@') {
          Some((l, r)) => (nfc(l.trim()), Some(nfc(r.trim()))),
          None => (nfc(w.trim()), None),
        };
        let head = HEAD_LABEL.captures(body).map(|c| clean(&c[1]));
        let label = match (conj, head) {
          (true, Some(g)) => Some(format!("गणः {g}")),
          (_, h) => h,
        };
        let (tables, forms) = parse_heritage(body);
        (!forms.is_empty()).then_some(Paradigm {
          lemma,
          root,
          label,
          tables,
          forms,
        })
      })
      .collect(),
  )
}

/// JNU tiṅanta: deduplicate (root, analysis) paradigms; each is a 3×3 grid.
pub fn parse_jnu(dir: &Path) -> Result<Vec<Paradigm>> {
  let rows = super::sources::read_tsv(dir, "Grammar/jnu-tiNanta/jnu-tiNanta.tsv")?;
  static HEAD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*(\S+)\s*\(([^)]*)\)").unwrap());
  let mut seen = HashMap::new();
  let mut out: Vec<Paradigm> = Vec::new();
  for (_, body) in rows {
    let body = super::markup::unescape_tsv(&body);
    let mut lines = body.split('\n');
    let Some(c) = lines.next().and_then(|l| HEAD.captures(l)) else {
      continue;
    };
    let (root, analysis) = (nfc(&c[1]), nfc(c[2].trim()));
    if seen.insert((root.clone(), analysis.clone()), ()).is_some() {
      continue;
    }
    let mut b = Builder::default();
    let persons = ["प्रथमपुरुषः", "मध्यमपुरुषः", "उत्तमपुरुषः"];
    let mut out_rows = Vec::new();
    for (i, line) in lines.filter(|l| !l.trim().is_empty()).enumerate() {
      let words: Vec<&str> = line.split_whitespace().collect();
      // Exactly three numbers per row; anything else is kept as one cell.
      let groups: Vec<Vec<String>> = if words.len() == 3 {
        words.iter().map(|w| vec![nfc(w)]).collect()
      } else {
        vec![words.iter().map(|w| nfc(w)).collect()]
      };
      let mut cells = Vec::new();
      for g in groups {
        let cell = b.cell();
        for f in &g {
          b.add_form(f, cell);
        }
        cells.push(g);
      }
      out_rows.push(json!({ "label": persons.get(i).copied().unwrap_or(""), "cells": cells }));
    }
    b.tables.push(json!({
      "group": null, "title": null, "label": analysis,
      "cols": ["एकवचनम्", "द्विवचनम्", "बहुवचनम्"], "rows": out_rows
    }));
    let (tables, forms) = b.finish();
    out.push(Paradigm {
      lemma: root,
      root: None,
      label: Some(analysis),
      tables,
      forms,
    });
  }
  Ok(out)
}

/// Split grid-only tables into a shared layout (cells emptied, structure and
/// labels kept) and the cell contents as text: cells joined by `|`, forms
/// within a cell by `,`. `None` when the tables contain lists or a form uses
/// a separator character (then the full JSON is stored).
pub fn split_layout(tables: &Value) -> Option<(Value, String)> {
  let mut layout = tables.clone();
  let mut cells: Vec<String> = Vec::new();
  for t in layout.as_array_mut()? {
    if t.get("list").is_some() {
      return None;
    }
    for r in t.get_mut("rows")?.as_array_mut()? {
      for c in r.get_mut("cells")?.as_array_mut()? {
        let forms: Vec<&str> = c.as_array()?.iter().filter_map(|f| f.as_str()).collect();
        if forms.iter().any(|f| f.contains('|') || f.contains(',')) {
          return None;
        }
        cells.push(forms.join(","));
        *c = Value::Array(vec![]);
      }
    }
  }
  Some((layout, cells.join("|")))
}

/// Inverse of [`split_layout`].
pub fn expand_layout(layout: &Value, forms: &str) -> Value {
  let mut out = layout.clone();
  let mut it = forms.split('|');
  if let Some(ts) = out.as_array_mut() {
    for t in ts {
      for r in t
        .get_mut("rows")
        .and_then(|r| r.as_array_mut())
        .into_iter()
        .flatten()
      {
        for c in r
          .get_mut("cells")
          .and_then(|c| c.as_array_mut())
          .into_iter()
          .flatten()
        {
          let cell = it.next().unwrap_or("");
          *c = Value::Array(
            cell
              .split(',')
              .filter(|f| !f.is_empty())
              .map(|f| Value::String(f.to_string()))
              .collect(),
          );
        }
      }
    }
  }
  out
}

/// Human-readable analysis of the given cells (e.g. `पुमान् · तृतीया · एकः`).
pub fn describe_cells(tables: &Value, cells: &[i16]) -> String {
  let mut idx: i16 = 0;
  let mut out: Vec<String> = Vec::new();
  let Some(ts) = tables.as_array() else {
    return String::new();
  };
  for t in ts {
    let s = |k: &str| t.get(k).and_then(|v| v.as_str()).filter(|s| !s.is_empty());
    if let Some(list) = t.get("list").and_then(|v| v.as_array()) {
      for it in list {
        if cells.contains(&idx) {
          let parts: Vec<&str> = [s("group"), it.get("label").and_then(|v| v.as_str())]
            .into_iter()
            .flatten()
            .collect();
          out.push(parts.join(" · "));
        }
        idx += 1;
      }
      continue;
    }
    let cols: Vec<&str> = t
      .get("cols")
      .and_then(|v| v.as_array())
      .map(|a| a.iter().filter_map(|c| c.as_str()).collect())
      .unwrap_or_default();
    for r in t
      .get("rows")
      .and_then(|v| v.as_array())
      .into_iter()
      .flatten()
    {
      let rl = r.get("label").and_then(|v| v.as_str()).unwrap_or("");
      for (ci, _) in r
        .get("cells")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .enumerate()
      {
        if cells.contains(&idx) {
          let parts: Vec<&str> = [
            s("group"),
            s("title"),
            s("label"),
            Some(rl),
            cols.get(ci).copied(),
          ]
          .into_iter()
          .flatten()
          .filter(|p| !p.is_empty())
          .collect();
          out.push(parts.join(" · "));
        }
        idx += 1;
      }
    }
  }
  out.join("; ")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn declension_grid() {
    let html = r#"सुबन्तावली [<font color="red">पुमान्</font>] <b><i>rāma</i></b><p></p><div><table border="1" align="center"> <tr><td align="center"><font color="green"><b><i>पुमान्</i></b></font></td><td align="center"><font color="blue">एकः</font></td><td align="center"><font color="blue">द्वौ</font></td></tr> <tr><td align="center"><font color="blue">तृतीया</font></td><td><font color="red">रामेण </font></td><td><font color="red">रामाभ्याम् <br />रामभ्याम् </font></td></tr></table>"#;
    let (tables, forms) = parse_heritage(html);
    assert_eq!(forms[0], ("rAmeRa".to_string(), vec![0]));
    assert_eq!(forms.len(), 3);
    assert_eq!(describe_cells(&tables, &[0]), "पुमान् · तृतीया · एकः");
    assert_eq!(describe_cells(&tables, &[1]), "पुमान् · तृतीया · द्वौ");
  }

  #[test]
  fn layout_roundtrip() {
    let html = r#"[<font color="red">पुमान्</font>]<table border="1"> <tr><td><font color="green"><b><i>पुमान्</i></b></font></td><td><font color="blue">एकः</font></td><td><font color="blue">द्वौ</font></td></tr> <tr><td><font color="blue">प्रथमा</font></td><td><font color="red">रामः </font></td><td><font color="red">रामौ <br />रामाउ </font></td></tr> <tr><td><font color="blue">x</font></td><td></td><td><font color="red">-</font></td></tr></table>"#;
    let (tables, _) = parse_heritage(html);
    let (layout, forms) = split_layout(&tables).unwrap();
    assert_eq!(forms, "रामः|रामौ,रामाउ||");
    assert_eq!(expand_layout(&layout, &forms), tables);
  }

  #[test]
  fn conj_groups_and_lists() {
    let html = r#"<table border="1" align="center"><tr><td align="center"><font color="green"><b>णिच्</b></font></td></tr></table><b>- लट् -</b><table border="1" align="center"> <tr><td><font color="green"><b><i>परस्मैपदे</i></b></font></td><td><font color="blue">एकः</font></td></tr> <tr><td><font color="blue">प्रथम</font></td><td><font color="red">भावयति </font></td></tr></table><table border="0"><tr><td><font color="green"><b>अव्यय</b></font></td></tr></table><table border="1" align="center"><td align="center"><font color="blue"><b>क्त्वा</b></font><br /><font color="red"> भूत्वा</font><p></p></td></table>"#;
    let (tables, forms) = parse_heritage(html);
    assert_eq!(forms.len(), 2);
    assert_eq!(
      describe_cells(&tables, &[0]),
      "णिच् · लट् · परस्मैपदे · प्रथम · एकः"
    );
    assert_eq!(describe_cells(&tables, &[1]), "अव्यय · क्त्वा");
  }
}
