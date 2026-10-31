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
| mw-heritage | MW/mw-heritage_du_sanskrit_san-eng (HTML) | reordered into MW book order; `bword://` links resolved via `.syn` (~75%); source lacks a few headwords (e.g. अग्नि) |
| ganapatha | Panini/gaNapAThaH/mUlam/gaNapATha.csv | 241 gaṇas, linked to their sūtra; members are search keys |
| apte-en, mw-en, borooah-en | dict_data/cologne/{ae,mwe,bor}.txt | English→Sanskrit (Cologne csl-orig, pinned commit + sha256: `scripts/fetch-cologne.sh`); SLP1 → Devanagari |
| `paradigms` + `paradigm_forms` | Heritage Declension A1–A5 (`heritage_decl`), B (`heritage_kridanta`, stem@root), Grammar (`heritage_conj`), JNU tiṅanta (`jnu_tinanta`) | 381,578 tables, 6,272,161 form→table rows; noun/kṛdanta tables store only forms + one of 5 shared layouts |

## Not imported (duplicates of imported data)
- Heritage `san-eng` files (same tables as `san-san`, English labels), `vachaspatyam`/`kalpadruma` HK
  editions, `mw-itrans-rom`, `MW-bi`/`Apte-bi` (same text, ITRANS keys + noisy English reverse keys),
  `apte-prAtipadikam-sa` (used only for stem keys), `purANa-index` (non-Devanagari), `rv-padapATha-hk`
  (empty TSV; same as -dev), `kalpadruma-prathamA` (subset with prathamā headwords).
- `mUlam/` raw sources (balamanorama.txt, *-dump.txt, rv-base TITUS html/text).

## Known data caveats
- Sūtra numbers follow the bundle's numbering (3,962 sūtras; e.g. 1.1 has 74, 8.4 has 67) and can be
  off by one against the common ashtadhyayi.com numbering (3,981). All five commentaries share it.
- Lost glyphs (U+FFFD) in Nyāsa / Tattvabodhinī / Bālamanoramā are shown as ◌, not guessed.
- The Heritage MW edition lacks some headwords present in MW (e.g. अग्नि).

## Size (sans_dict, after `vacuum full`)
~1.97 GB: paradigm_forms 804 MB, entries 597 MB, paradigms 416 MB, entry_keys 144 MB.
