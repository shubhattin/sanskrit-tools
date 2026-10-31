//! Minimal StarDict reader (`.ifo` / `.idx` / `.dict[.dz]` / `.syn`).

use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

pub struct StarDict {
  pub ifo: HashMap<String, String>,
  /// (headword, payload) in `.idx` order (sorted by headword, NOT book order;
  /// entries sharing a headword keep their relative source order).
  pub entries: Vec<(String, String)>,
  /// (alternate key, index into `entries`)
  pub syn: Vec<(String, usize)>,
}

fn cstr(buf: &[u8], i: &mut usize) -> Result<String> {
  let end = buf[*i..]
    .iter()
    .position(|&b| b == 0)
    .context("unterminated string")?;
  let s = String::from_utf8_lossy(&buf[*i..*i + end]).into_owned();
  *i += end + 1;
  Ok(s)
}

fn be_u32(buf: &[u8], i: &mut usize) -> Result<u32> {
  let b: [u8; 4] = buf.get(*i..*i + 4).context("truncated u32")?.try_into()?;
  *i += 4;
  Ok(u32::from_be_bytes(b))
}

/// `base` is the path without extension, e.g. `dict_data/MW/mw-sa/mw-sa`.
pub fn load(base: &Path) -> Result<StarDict> {
  let with = |ext: &str| {
    base.with_file_name(format!(
      "{}{ext}",
      base.file_name().unwrap().to_string_lossy()
    ))
  };

  let ifo_text = std::fs::read_to_string(with(".ifo")).with_context(|| format!("{base:?}.ifo"))?;
  let mut ifo = HashMap::new();
  for line in ifo_text.lines().skip(1) {
    if let Some((k, v)) = line.split_once('=') {
      ifo.insert(k.trim().to_string(), v.trim().to_string());
    }
  }
  if ifo.get("idxoffsetbits").map(|s| s.as_str()) == Some("64") {
    bail!("64-bit idx offsets are not supported");
  }

  let dz = with(".dict.dz");
  let data = if dz.exists() {
    let mut out = Vec::new();
    GzDecoder::new(std::fs::File::open(&dz)?).read_to_end(&mut out)?;
    out
  } else {
    std::fs::read(with(".dict"))?
  };

  let idx = std::fs::read(with(".idx"))?;
  let mut entries = Vec::new();
  let mut i = 0;
  while i < idx.len() {
    let word = cstr(&idx, &mut i)?;
    let off = be_u32(&idx, &mut i)? as usize;
    let size = be_u32(&idx, &mut i)? as usize;
    let payload = data.get(off..off + size).context("payload out of range")?;
    entries.push((word, String::from_utf8_lossy(payload).into_owned()));
  }

  let mut syn = Vec::new();
  let syn_path = with(".syn");
  if syn_path.exists() {
    let buf = std::fs::read(syn_path)?;
    let mut i = 0;
    while i < buf.len() {
      let word = cstr(&buf, &mut i)?;
      let n = be_u32(&buf, &mut i)? as usize;
      syn.push((word, n));
    }
  }
  Ok(StarDict { ifo, entries, syn })
}
