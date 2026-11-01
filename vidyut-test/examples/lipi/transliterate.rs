//! vidyut-lipi · `Lipika::transliterate(text, from, to)`
//! Convert Sanskrit text between scripts/romanizations. `Lipika` caches the
//! from→to mapping, so reuse one instance when converting many strings.
//!
//! Run: cargo run --example lipi_transliterate [-- "IAST text"]
use vidyut_lipi::{Lipika, Scheme};

fn main() {
  let input = vidyut_test::arg_or("dharmakṣetre kurukṣetre samavetā yuyutsavaḥ");
  let mut lipika = Lipika::new();

  println!("input (IAST): {input}\n");
  for to in [
    Scheme::Slp1,
    Scheme::Devanagari,
    Scheme::HarvardKyoto,
    Scheme::Itrans,
    Scheme::Iso15919,
    Scheme::Velthuis,
    Scheme::Wx,
    Scheme::Telugu,
    Scheme::Tamil,
    Scheme::Grantha,
    Scheme::Bengali,
    Scheme::Kannada,
    Scheme::Sharada,
    Scheme::Brahmi,
  ] {
    let out = lipika.transliterate(&input, Scheme::Iast, to);
    println!("{:<14} {out}", format!("{to:?}"));
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example lipi_transliterate
// input (IAST): dharmakṣetre kurukṣetre samavetā yuyutsavaḥ
//
// Slp1           Darmakzetre kurukzetre samavetA yuyutsavaH
// Devanagari     धर्मक्षेत्रे कुरुक्षेत्रे समवेता युयुत्सवः
// HarvardKyoto   dharmakSetre kurukSetre samavetA yuyutsavaH
// Itrans         dharmakShetre kurukShetre samavetA yuyutsavaH
// Iso15919       dharmakṣētrē kurukṣētrē samavētā yuyutsavaḥ
// Velthuis       dharmak.setre kuruk.setre samavetaa yuyutsava.h
// Wx             XarmakRewre kurukRewre samavewA yuyuwsavaH
// Telugu         ధర్మక్షేత్రే కురుక్షేత్రే సమవేతా యుయుత్సవః
// Tamil          த⁴ர்மக்ஷேத்ரே குருக்ஷேத்ரே ஸமவேதா யுயுத்ஸவ꞉
// Grantha        𑌧𑌰𑍍𑌮𑌕𑍍𑌷𑍇𑌤𑍍𑌰𑍇 𑌕𑍁𑌰𑍁𑌕𑍍𑌷𑍇𑌤𑍍𑌰𑍇 𑌸𑌮𑌵𑍇𑌤𑌾 𑌯𑍁𑌯𑍁𑌤𑍍𑌸𑌵𑌃
// Bengali        ধর্মক্ষেত্রে কুরুক্ষেত্রে সমবেতা যুয়ুত্সবঃ
// Kannada        ಧರ್ಮಕ್ಷೇತ್ರೇ ಕುರುಕ್ಷೇತ್ರೇ ಸಮವೇತಾ ಯುಯುತ್ಸವಃ
// Sharada        𑆣𑆫𑇀𑆩𑆑𑇀𑆰𑆼𑆠𑇀𑆫𑆼 𑆑𑆶𑆫𑆶𑆑𑇀𑆰𑆼𑆠𑇀𑆫𑆼 𑆱𑆩𑆮𑆼𑆠𑆳 𑆪𑆶𑆪𑆶𑆠𑇀𑆱𑆮𑆂
// Brahmi         𑀥𑀭𑁆𑀫𑀓𑁆𑀱𑁂𑀢𑁆𑀭𑁂 𑀓𑀼𑀭𑀼𑀓𑁆𑀱𑁂𑀢𑁆𑀭𑁂 𑀲𑀫𑀯𑁂𑀢𑀸 𑀬𑀼𑀬𑀼𑀢𑁆𑀲𑀯𑀂
