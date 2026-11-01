# vidyut-test

Runnable examples for every crate in [ambuda-org/vidyut](https://github.com/ambuda-org/vidyut). Each file covers one function or feature: a short header comment, the code, and the real output pasted at the bottom.

```sh
./fetch-data.sh                                   # one-time: ~108 MB of vidyut data into ./data (git-ignored)
cargo run --example prakriya_tinanta              # default sample input
cargo run --example prakriya_tinanta -- "kṛ 8"    # your own input (IAST; roots take an optional gaṇa)
./fill-outputs.sh [example_name ...]              # re-run examples and refresh the output comments
```

Input and output use IAST. The vidyut APIs themselves use SLP1, so the examples convert with
`slp()`, `iast()` and `deva()` from `src/lib.rs`, which wrap vidyut-lipi.

| dir | crate | examples |
|---|---|---|
| `lipi/` | vidyut-lipi: transliteration | transliterate, mapping, detect, schemes |
| `sandhi/` | vidyut-sandhi: phonetic sandhi rules | generate_rules, split_at, split_all, join, custom_rules |
| `cheda/` | vidyut-cheda: segmenter and tagger | segment, lemma_probability |
| `kosha/` | vidyut-kosha: word-form lookup (167M forms in an FST) | lookup, contains_key, dhatus, pratipadikas, lookup_to_prakriya |
| `prakriya/` | vidyut-prakriya: Pāṇinian word generator | tinanta, subanta, krdanta, taddhitanta, samasa, sanadi, upasarga, namadhatu, stryanta, vakya, dhatupatha, explain_steps, options, ganapatha |
| `chandas/` | vidyut-chandas: meter identification | classify, classify_all, scan, meters |
| `akshara/` | vidyut-akshara: sound-set bitset | sound_set |

All crates are pinned to a single git revision (see `Cargo.toml`). Three of them (cheda, sandhi
and akshara) are not published on crates.io, and the types only line up when every crate comes
from the same source.
