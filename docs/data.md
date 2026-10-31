# Source data (`dict_data/`)

Arsha Drishti bundle of StarDict dictionaries (made for GoldenDict / iOS "Dictionary Universal");
content mostly from the sanskrit-coders *stardict-sanskrit* project (Cologne CDSL, Sanskrit
Heritage, JNU, Simon Wiles' BHSD).

## Format
- `.ifo` metadata (`sametypesequence` m = text, h = HTML); `.idx` headword → (offset, size), **sorted**;
  `.dict[.dz]` payloads (written in sorted order, so book order is lost); `.syn` alternate keys.
- `.tsv` (`headword<TAB>body`, literal `\n`) is the build source: **one line per sense**, book order;
  contiguous lines with the same headword form one entry. Preferred whenever present.
- Ignored: `_ IOS … zips` (same files + GoldenDict caches), `__MACOSX`, `Icon_`, `Grammar/{kashika,nyasa,
  tattvabodhini,balamanorama,laghu-kaumudi,dhatupatha}` (byte-identical to `Panini/`), `-bi` English
  reverse keys (noisy tokenization; replaced by Postgres FTS), HK/ITRANS variant files (we compute keys).

## Imported (slug → file)
| slug | source | notes |
|---|---|---|
| mw | MW/mw-sa/mw-sa.tsv | Cologne "AS" digit encoding converted to IAST |
| apte | Apte/apte-sa + apte-prAtipadikam-sa (line-aligned → stem keys) | |
| shabda-sagara | Other/shabda-sAgara | `hw¦` prefix stripped |
| bhsd | Other/stardict-bhsd (HTML, IAST headwords) | ordered by Cologne record number |
| vacaspatyam, kalpadruma | Sanskrit-Sanskrit/*-sa | Cologne `{@ @}`, `[Page…]` |
| amarakosha | amara-onto | synonym sets + relations as cross-links |
| akhyatachandrika, ranade-vedic-rituals, computer-shrikanta | | rAnADe is sa→hi; computer terms en→sa |
| dhatupatha (+ `dhatus` table) | Panini/dhAtupATha-sa | 41/2282 lines have irregular layout (kept raw in `extra`) |
| madhaviya-dhatuvrtti, kshiratarangini, dhatupradipa, krdanta-rupa-mala | | linked to a dhātu only when the root is unambiguous |
| kashika, nyasa, laghu-kaumudi, balamanorama, tattvabodhini (+ `sutras`) | keyed by sūtra number | ॒…॑ misused as quotes → ‘…’; U+FFFD (lost glyphs) shown as ◌, not guessed |
| rv-padapatha, rv-padasvara, bloomfield, purana-index | concordances | |
| `word_forms` | Grammar/jnu-tiNanta | conjugated form → root + paradigm |

## Not yet imported (phase 2)
- MW Heritage edition (HTML, 193k entries).
- Heritage declension (A1–A5, B) and conjugation tables → `word_forms` (nominal form lookup, e.g. रामेण → राम).
- Gaṇapāṭha (`Panini/gaNapAThaH/mUlam/gaNapATha.csv`).
