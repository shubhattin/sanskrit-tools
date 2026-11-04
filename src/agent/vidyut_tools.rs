//! The agent's grammar tools over [`crate::vidyut`] (Ambuda's vidyut): conjugation, declension,
//! kṛdantas, step-by-step derivations, word analysis (kosha + cheda), sandhi and meter. They call the same core as
//! `/api/vidyut/*` and return compact text rows for the model, plus a `page` link to the matching
//! `/vidyut/*` tool on the site. Shapes: `docs/agent.md`.

use super::tools::ToolCtx;
use crate::routes;
use crate::translit::{has_devanagari, query_keys, slp1_to_deva};
use crate::vidyut::chandas;
use crate::vidyut::prakriya::{
  self, DhatuArgs, LakaraId, LingaId, PrayogaId, PurushaId, RootInfo, SanadiId, StemArgs, VacanaId,
  VibhaktiId, WordSpec,
};
use crate::vidyut::{kosha, sandhi};
use axum::Json;
use axum::extract::State;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

const PURUSHA: [&str; 3] = ["प्रथमपुरुषः", "मध्यमपुरुषः", "उत्तमपुरुषः"];
const VIBHAKTI: [&str; 8] = [
  "प्रथमा",
  "द्वितीया",
  "तृतीया",
  "चतुर्थी",
  "पञ्चमी",
  "षष्ठी",
  "सप्तमी",
  "सम्बोधनम्",
];
const MAX_STEPS: usize = 80;

/// Any script → Devanagari, word by word (the vidyut core reads Devanagari only).
fn deva(text: &str) -> String {
  if has_devanagari(text) {
    return text.trim().to_string();
  }
  text
    .lines()
    .map(|line| {
      line
        .split_whitespace()
        .filter_map(|w| query_keys(w).into_iter().next())
        .map(|k| slp1_to_deva(&k))
        .collect::<Vec<_>>()
        .join(" ")
    })
    .collect::<Vec<_>>()
    .join("\n")
}

fn cell(forms: &[String]) -> String {
  if forms.is_empty() {
    "—".into()
  } else {
    forms.join(" / ")
  }
}

fn rows(labels: &[&str], forms: &[Vec<Vec<String>>]) -> Vec<String> {
  forms
    .iter()
    .zip(labels)
    .map(|(r, l)| {
      format!(
        "{l}: {}",
        r.iter().map(|c| cell(c)).collect::<Vec<_>>().join(" | ")
      )
    })
    .collect()
}

fn root_json(r: &RootInfo) -> Value {
  json!({ "code": r.code, "upadesha": r.aupadeshika, "root": r.roots.join(" / "),
          "artha": r.artha, "gana": r.gana })
}

/// The root a model names: a Dhātupāṭha code, or a root in any script (+ optional gaṇa).
/// Returns it and the other roots with the same form (homonyms in other gaṇas).
fn resolve_root(root: &str, gana: Option<u8>) -> Result<(RootInfo, Vec<RootInfo>), String> {
  let q = root.trim();
  if q.chars().next().is_some_and(|c| c.is_ascii_digit()) {
    return prakriya::root(q)
      .map(|r| (r, vec![]))
      .ok_or_else(|| format!("no root with code {q}"));
  }
  let hits = prakriya::search_roots(&deva(q), gana, 12).map_err(|e| e.0)?;
  let mut hits = hits.into_iter();
  let first = hits
    .next()
    .ok_or_else(|| format!("no root {q} in vidyut's Dhātupāṭha"))?;
  let same: Vec<RootInfo> = hits.filter(|r| r.roots == first.roots).collect();
  Ok((first, same))
}

/// `/vidyut/dhatu?code=…` with the same modifiers (comma lists).
fn dhatu_page(args: &DhatuArgs, extra: &str) -> String {
  let mut url = format!("/vidyut/dhatu?code={}", args.code);
  if !args.sanadi.is_empty() {
    let s: Vec<&str> = args
      .sanadi
      .iter()
      .map(|s| match s {
        SanadiId::Nic => "nic",
        SanadiId::San => "san",
        SanadiId::Yan => "yan",
        SanadiId::YanLuk => "yan_luk",
      })
      .collect();
    url.push_str(&format!("&sanadi={}", s.join(",")));
  }
  if !args.upasargas.is_empty() {
    url.push_str(&format!("&upa={}", args.upasargas.join(",")));
  }
  url.push_str(extra);
  url
}

// --- vidyut_conjugate ----------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub(super) struct ConjugateInput {
  /// Verbal root in any script (भू, gam, kR) or a vidyut Dhātupāṭha code ("08.0010").
  root: String,
  /// Gaṇa 1–10, to pick between homonymous roots.
  gana: Option<u8>,
  /// Default kartari.
  prayoga: Option<PrayogaId>,
  /// Only this lakāra (default: all ten).
  lakara: Option<LakaraId>,
  /// Derived roots, in order: ["nic"] causative, ["san"] desiderative, ["nic","san"] …
  sanadi: Option<Vec<SanadiId>>,
  /// Upasargas in Devanagari, outermost first: ["प्र"], ["उप","सम्"]; आ for आङ्.
  upasargas: Option<Vec<String>>,
}

pub(super) async fn conjugate(_: ToolCtx, input: ConjugateInput) -> Result<Value, String> {
  let (root, homonyms) = resolve_root(&input.root, input.gana)?;
  let args = DhatuArgs {
    code: root.code.clone(),
    sanadi: input.sanadi.unwrap_or_default(),
    upasargas: input.upasargas.unwrap_or_default(),
  };
  let prayoga = input.prayoga.unwrap_or(PrayogaId::Kartari);
  let c = prakriya::conjugate(&args, prayoga).map_err(|e| e.0)?;
  let tables: Vec<Value> = c
    .tables
    .iter()
    .flat_map(|t| {
      t.lakaras
        .iter()
        .filter(|l| input.lakara.is_none_or(|x| x == l.lakara))
        .map(move |l| {
          json!({ "pada": t.pada, "lakara": l.lakara,
                  "rows": rows(&PURUSHA, &l.forms) })
        })
    })
    .collect();
  let prayoga_q = match prayoga {
    PrayogaId::Kartari => "",
    PrayogaId::Karmani => "&prayoga=karmani",
    PrayogaId::Bhave => "&prayoga=bhave",
  };
  Ok(json!({
    "root": root_json(&root),
    "homonyms": homonyms.iter().map(root_json).collect::<Vec<_>>(),
    "stems": c.stems,
    "prayoga": prayoga,
    "columns": "eka | dvi | bahu",
    "tables": tables,
    "page": dhatu_page(&args, prayoga_q),
  }))
}

// --- vidyut_decline ------------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub(super) struct DeclineInput {
  /// Nominal stem (prātipadika) in any script: राम, mati, rAjan.
  stem: String,
  linga: LingaId,
}

pub(super) async fn decline(_: ToolCtx, input: DeclineInput) -> Result<Value, String> {
  let stem = deva(&input.stem);
  let d =
    prakriya::decline(&StemArgs::Basic { stem: stem.clone() }, input.linga).map_err(|e| e.0)?;
  let linga_q = match input.linga {
    LingaId::Pum => "",
    LingaId::Stri => "&linga=stri",
    LingaId::Napumsaka => "&linga=napumsaka",
  };
  Ok(json!({
    "stem": stem,
    "linga": input.linga,
    "columns": "eka | dvi | bahu",
    "rows": rows(&VIBHAKTI, &d.forms),
    "page": format!("/vidyut/nama?stem={stem}{linga_q}"),
  }))
}

// --- vidyut_krdantas -----------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub(super) struct KrdantasInput {
  /// Verbal root in any script or a vidyut Dhātupāṭha code.
  root: String,
  gana: Option<u8>,
  sanadi: Option<Vec<SanadiId>>,
  /// Devanagari upasargas, outermost first.
  upasargas: Option<Vec<String>>,
}

pub(super) async fn krdantas(_: ToolCtx, input: KrdantasInput) -> Result<Value, String> {
  let (root, homonyms) = resolve_root(&input.root, input.gana)?;
  let args = DhatuArgs {
    code: root.code.clone(),
    sanadi: input.sanadi.unwrap_or_default(),
    upasargas: input.upasargas.unwrap_or_default(),
  };
  let groups = prakriya::krdantas(&args).map_err(|e| e.0)?;
  let groups: Vec<Value> = groups
    .iter()
    .map(|g| {
      let items: Vec<String> = g
        .items
        .iter()
        .map(|k| {
          let mut label = k.label.clone();
          if k.lakara == Some(LakaraId::Lrt) {
            label.push_str(" (लृट्)");
          }
          if k.prayoga == Some(PrayogaId::Karmani) {
            label.push_str(" (कर्मणि)");
          }
          match &k.gloss {
            Some(g) => format!("{label} — {g}: {}", k.forms.join(" / ")),
            None => format!("{label}: {}", k.forms.join(" / ")),
          }
        })
        .collect();
      json!({ "group": g.label, "items": items })
    })
    .collect();
  Ok(json!({
    "root": root_json(&root),
    "homonyms": homonyms.iter().map(root_json).collect::<Vec<_>>(),
    "groups": groups,
    "page": dhatu_page(&args, "&tab=krt"),
  }))
}

// --- vidyut_derivation ---------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub(super) struct DerivationInput {
  /// For a verb form: the root (any script or code), with lakara, purusha, vacana.
  root: Option<String>,
  gana: Option<u8>,
  sanadi: Option<Vec<SanadiId>>,
  upasargas: Option<Vec<String>>,
  prayoga: Option<PrayogaId>,
  lakara: Option<LakaraId>,
  purusha: Option<PurushaId>,
  /// For a noun form: the stem, with linga, vibhakti, vacana.
  stem: Option<String>,
  linga: Option<LingaId>,
  vibhakti: Option<VibhaktiId>,
  vacana: Option<VacanaId>,
  /// The form to explain. Alone (or with `analysis`), it is derived as vidyut_analyze reads it;
  /// with root / stem, it picks among several results.
  form: Option<String>,
  /// One of the form's analyses from vidyut_analyze (its text, e.g. "उद्भवन (उद्भू + ल्युट्)
  /// napumsaka prathama eka"); default: the likeliest.
  analysis: Option<String>,
}

/// A form's analysis (by its key, else the likeliest) as a derivable word.
async fn spec_of_form(form: &str, analysis: Option<&str>) -> Result<WordSpec, String> {
  let slp = crate::vidyut::to_slp1(form).map_err(|e| e.0)?;
  let key = sandhi::word_key(&slp);
  let analyses = tokio::task::spawn_blocking(move || {
    kosha::load();
    kosha::analyses(&key)
  })
  .await
  .map_err(|e| e.to_string())?;
  if analyses.is_empty() {
    return Err(format!(
      "vidyut's grammar has no form {form}: give root / stem instead"
    ));
  }
  let chosen = match analysis {
    Some(a) => analyses.iter().find(|x| x.key == a.trim()).ok_or_else(|| {
      let keys: Vec<&str> = analyses.iter().map(|x| x.key.as_str()).collect();
      format!(
        "no such analysis of {form}; it can be: {}",
        keys.join(" | ")
      )
    })?,
    None => &analyses[0],
  };
  chosen.word.clone().ok_or_else(|| {
    format!(
      "{} has nothing to derive (a plain indeclinable or root outside the Dhātupāṭha)",
      chosen.key
    )
  })
}

pub(super) async fn derivation(_: ToolCtx, i: DerivationInput) -> Result<Value, String> {
  let spec = if let (None, None, Some(form)) = (&i.root, &i.stem, &i.form) {
    spec_of_form(&deva(form), i.analysis.as_deref()).await?
  } else if let Some(root) = &i.root {
    let (root, _) = resolve_root(root, i.gana)?;
    WordSpec::Tinanta {
      dhatu: DhatuArgs {
        code: root.code,
        sanadi: i.sanadi.unwrap_or_default(),
        upasargas: i.upasargas.unwrap_or_default(),
      },
      prayoga: i.prayoga.unwrap_or(PrayogaId::Kartari),
      lakara: i.lakara.ok_or("give the lakara")?,
      purusha: i.purusha.ok_or("give the purusha")?,
      vacana: i.vacana.ok_or("give the vacana")?,
      pada: None,
    }
  } else if let Some(stem) = &i.stem {
    WordSpec::Subanta {
      stem: StemArgs::Basic { stem: deva(stem) },
      linga: i.linga.ok_or("give the linga")?,
      vibhakti: i.vibhakti.ok_or("give the vibhakti")?,
      vacana: i.vacana.ok_or("give the vacana")?,
    }
  } else {
    return Err("give a root (verb form), a stem (noun form), or a form (+ analysis)".into());
  };
  let ds = prakriya::derive(&spec).map_err(|e| e.0)?;
  let want = i.form.as_deref().map(deva);
  let d = ds
    .iter()
    .find(|d| want.as_deref() == Some(d.text.as_str()))
    .or(ds.first())
    .ok_or("vidyut produced no derivation")?;
  let steps: Vec<Value> = d
    .steps
    .iter()
    .take(MAX_STEPS)
    .map(|s| {
      let result: Vec<&str> = s.terms.iter().map(|t| t.text.as_str()).collect();
      json!({ "rule": s.rule.code, "sutra_id": s.rule.sutra_id,
              "text": s.rule.text, "result": result.join(" + ") })
    })
    .collect();
  Ok(json!({
    "results": ds.iter().map(|d| d.text.clone()).collect::<Vec<_>>(),
    "form": d.text,
    "optional_rules": d.optional_rules.iter()
      .map(|o| json!({ "rule": o.rule.code, "applied": o.applied }))
      .collect::<Vec<_>>(),
    "steps": steps,
    "truncated": d.steps.len() > MAX_STEPS,
  }))
}

// --- vidyut_sandhi -------------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub(super) struct SandhiInput {
  /// Words to join, in order (any script): ["रामः", "अस्ति"].
  join: Option<Vec<String>>,
  /// A joined word or phrase to split (any script): "तथैव".
  split: Option<String>,
}

pub(super) async fn sandhi(ctx: ToolCtx, input: SandhiInput) -> Result<Value, String> {
  if let Some(words) = input.join {
    let words: Vec<String> = words.iter().map(|w| deva(w)).collect();
    let joined = prakriya::join_words(&words).map_err(|e| e.0)?;
    return Ok(json!({ "words": words, "joined": joined,
                      "page": format!("/vidyut/sandhi?q={}", words.join(" ")) }));
  }
  let Some(text) = input.split else {
    return Err("give `join` (words) or `split` (text)".into());
  };
  let text = deva(&text);
  let Json(chunks) = routes::vidyut::sandhi_split(
    State(ctx.state),
    Json(routes::vidyut::TextRequest { text: text.clone() }),
  )
  .await
  .map_err(|e| e.1)?;
  let chunks: Vec<Value> = to_value(&chunks)?
    .as_array()
    .cloned()
    .unwrap_or_default()
    .into_iter()
    .map(|c| {
      let splits: Vec<String> = c["splits"]
        .as_array()
        .into_iter()
        .flatten()
        .take(12)
        .map(|s| {
          let mark = |k: &str| {
            if s[k].as_bool() == Some(true) {
              "✓"
            } else {
              ""
            }
          };
          format!(
            "{}{} + {}{}",
            s["left"].as_str().unwrap_or_default(),
            mark("left_known"),
            s["right"].as_str().unwrap_or_default(),
            mark("right_known")
          )
        })
        .collect();
      let readings: Vec<Value> = c["readings"]
        .as_array()
        .into_iter()
        .flatten()
        .take(3)
        .map(|r| {
          // What each word is (vidyut's kosha: the likeliest analysis).
          let words: Vec<String> = r["parts"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|p| {
              let key = p["analyses"].get(0)?["key"].as_str()?;
              Some(format!("{} = {key}", p["text"].as_str()?))
            })
            .collect();
          json!({ "reading": r["label"], "cheda": r["cheda"], "words": words })
        })
        .collect();
      json!({ "chunk": c["chunk"], "readings": readings, "splits": splits })
    })
    .collect();
  Ok(json!({
    "note": "readings: best whole readings (all words), best first; cheda = the reading of \
             vidyut's statistical segmenter; words = each known form's likeliest analysis. splits: \
             two-word phonetic candidates. ✓ = a dictionary word or a form the grammar generates.",
    "chunks": chunks,
    "page": format!("/vidyut/sandhi?tab=split&q={text}"),
  }))
}

// --- vidyut_analyze ------------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub(super) struct AnalyzeInput {
  /// A word, phrase or verse (any script); sandhi is undone first.
  text: String,
}

/// Parses text word by word: each chunk's best reading (cheda's, else the optimizer's), every
/// analysis of each word, and the other readings.
pub(super) async fn analyze(ctx: ToolCtx, input: AnalyzeInput) -> Result<Value, String> {
  let text = deva(&input.text);
  let Json(chunks) = routes::vidyut::sandhi_split(
    State(ctx.state),
    Json(routes::vidyut::TextRequest { text: text.clone() }),
  )
  .await
  .map_err(|e| e.1)?;
  let chunks: Vec<Value> = chunks
    .iter()
    .map(|c| {
      let best = c.readings.iter().find(|r| r.cheda).or(c.readings.first());
      let words: Vec<Value> = best
        .into_iter()
        .flat_map(|r| &r.parts)
        .map(|p| {
          json!({
            "word": p.text,
            "in_dictionaries": p.dictionary,
            "analyses": p.analyses.iter().take(5).map(|a| &a.key).collect::<Vec<_>>(),
          })
        })
        .collect();
      json!({
        "chunk": c.chunk,
        "reading": best.map(|r| &r.label),
        "by": best.map(|r| if r.cheda { "cheda" } else { "optimizer" }),
        "words": words,
        "other_readings": c.readings.iter()
          .filter(|r| Some(&r.label) != best.map(|b| &b.label))
          .take(3)
          .map(|r| &r.label)
          .collect::<Vec<_>>(),
      })
    })
    .collect();
  Ok(json!({
    "note": "Per chunk: the likeliest reading (cheda = vidyut's statistical segmenter) and each \
             word's analyses from Pāṇini's grammar, likeliest first (stem (root + kṛt) gender case \
             number; verbs: root lakāra person number voice). Pick the analysis that fits the \
             sentence. vidyut_derivation with `form` + `analysis` explains one. ✓ = known word.",
    "chunks": chunks,
    "page": format!("/vidyut/sandhi?tab=split&q={text}"),
  }))
}

fn to_value<T: serde::Serialize>(v: &T) -> Result<Value, String> {
  serde_json::to_value(v).map_err(|e| e.to_string())
}

// --- vidyut_meter --------------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub(super) struct MeterInput {
  /// The verse, one pāda or half-verse per line (any script; daṇḍas also end lines).
  verse: String,
}

pub(super) async fn meter(_: ToolCtx, input: MeterInput) -> Result<Value, String> {
  let s = chandas::identify(&deva(&input.verse)).map_err(|e| e.0)?;
  let lines: Vec<Value> = s
    .lines
    .iter()
    .map(|l| {
      let syl: Vec<&str> = l.syllables.iter().map(|x| x.text.as_str()).collect();
      json!({ "syllables": syl.join(" "), "pattern": l.pattern,
              "ganas": l.ganas.join(" "), "matras": l.matras })
    })
    .collect();
  Ok(json!({
    "meter": to_value(&s.best)?,
    "lines": lines,
    "also_compatible": s.others.iter().map(|o| o.name.clone()).collect::<Vec<_>>(),
    "page": "/vidyut/chandas",
  }))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn derives_a_form_from_its_analysis() {
    let spec = spec_of_form("उद्भवनम्", None).await.unwrap();
    assert!(matches!(
      spec,
      WordSpec::Subanta {
        stem: StemArgs::Krdanta { .. },
        ..
      }
    ));
    let d = prakriya::derive(&spec).unwrap();
    assert_eq!(d[0].text, "उद्भवनम्");
    let err = spec_of_form("गच्छति", Some("nonsense")).await.unwrap_err();
    assert!(err.contains("गम् lat prathama eka kartari"), "{err}");
  }

  #[test]
  fn reads_any_script_and_resolves_roots() {
    assert_eq!(deva("rAmaH asti"), "रामः अस्ति");
    assert_eq!(deva("गम्"), "गम्");
    let (r, _) = resolve_root("gam", None).unwrap();
    assert_eq!(r.code, "01.1137");
    let (r, homonyms) = resolve_root("kf", Some(8)).unwrap();
    assert_eq!(r.code, "08.0010");
    assert!(homonyms.is_empty());
    assert_eq!(resolve_root("08.0010", None).unwrap().0.code, "08.0010");
    let args = DhatuArgs {
      code: "01.0001".into(),
      sanadi: vec![SanadiId::Nic],
      upasargas: vec!["प्र".into()],
    };
    assert_eq!(
      dhatu_page(&args, "&tab=krt"),
      "/vidyut/dhatu?code=01.0001&sanadi=nic&upa=प्र&tab=krt"
    );
  }
}
