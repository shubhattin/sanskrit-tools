//! vidyut-lipi · `Scheme::iter()` / `scheme.iso_15924_code()`
//! List every supported scheme (Brahmic scripts + romanizations) with its ISO 15924
//! code, and render one word in all of them.
//!
//! Run: cargo run --example lipi_schemes [-- "IAST word"]
use vidyut_lipi::{Lipika, Scheme};

fn main() {
  let word = vidyut_test::arg_or("vidyut");
  let mut lipika = Lipika::new();
  let all: Vec<_> = Scheme::iter().collect();
  println!("{} schemes; '{word}' in each:\n", all.len());
  for s in all {
    let out = lipika.transliterate(&word, Scheme::Iast, *s);
    println!("{:<22} {:<5} {out}", format!("{s:?}"), s.iso_15924_code());
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example lipi_schemes
// 50 schemes; 'vidyut' in each:
//
// Assamese               Beng  ৱিদ্যুৎ
// Balinese               Bali  ᬯᬶᬤ᭄ᬬᬸᬢ᭄
// BarahaSouth            Latn  vidyut
// Bengali                Beng  বিদ্যুৎ
// Bhaiksuki              Bhks  𑰪𑰰𑰟𑰿𑰧𑰲𑰝𑰿
// Brahmi                 Brah  𑀯𑀺𑀤𑁆𑀬𑀼𑀢𑁆
// Burmese                Mymr  ဝိဒျုတ်
// Cham                   Cham  ꨥꨪꨕꨳꨭꩅ
// Devanagari             Deva  विद्युत्
// Dogra                  Dogr  𑠦𑠭𑠛𑠹𑠣𑠯𑠙𑠹
// Grantha                Gran  𑌵𑌿𑌦𑍍𑌯𑍁𑌤𑍍
// Gujarati               Gujr  વિદ્યુત્
// GunjalaGondi           Gong  𑵭𑶋𑵸𑶗𑵬𑶍𑵳
// Gurmukhi               Guru  ਵਿਦ੍ਯੁਤ੍
// HarvardKyoto           Latn  vidyut
// Iast                   Latn  vidyut
// Iso15919               Latn  vidyut
// Itrans                 Latn  vidyut
// Javanese               Java  ꦮꦶꦢꦾꦸꦠ꧀
// Kaithi                 Kthi  𑂫𑂱𑂠𑂹𑂨𑂳𑂞𑂹
// Kannada                Knda  ವಿದ್ಯುತ್
// Kharoshthi             Khar  𐨬𐨁𐨡𐨿𐨩𐨂𐨟𐨿
// Khmer                  Khmr  វិទ្យុត៑
// Khudawadi              Sind  𑋛𑋡𑋏𑋪𑋘𑋣𑋍𑋪
// Limbu                  Limb  ᤘᤡᤍᤩᤢᤋ᤻
// Malayalam              Mlym  വിദ്യുത്
// MasaramGondi           Gonm  𑴨𑴲𑴝𑵅𑴥𑴴𑴛𑵅
// MeeteiMayek            Mtei  ꯋꯤꯗ꯭ꯌꯨꯠ
// Modi                   Modi  𑘪𑘱𑘟𑘿𑘧𑘳𑘝𑘿
// Mon                    Mymr  ဝိဒျုတ်
// Nandinagari            Nand  𑧊𑧒𑦿𑧠𑧇𑧔𑦽𑧠
// Newa                   Newa  𑐰𑐶𑐡𑑂𑐫𑐸𑐟𑑂
// Odia                   Orya  ଵିଦ୍ଯୁତ୍
// OlChiki                Olck  ᱣᱤᱫᱭᱩᱛ
// Saurashtra             Saur  ꢮꢶꢣ꣄ꢫꢸꢡ꣄
// Sharada                Shrd  𑆮𑆴𑆢𑇀𑆪𑆶𑆠𑇀
// Siddham                Sidd  𑖪𑖰𑖟𑖿𑖧𑖲𑖝𑖿
// Sinhala                Sinh  විද්යුත්
// Slp1                   Latn  vidyut
// Soyombo                Soyo  𑩾𑩑𑩭𑪘𑩻𑩒𑩫𑪘
// TaiTham                Lana  ᩅᩥᨴ᩠ᨿᩩᨲ᩺
// Takri                  Takr  𑚦𑚮𑚛𑚶𑚣𑚰𑚙𑚶
// Tamil                  Taml  வித்³யுத்
// Telugu                 Telu  విద్యుత్
// Thai                   Thai  วิทฺยุตฺ
// Tibetan                Tibt  བིདྱུཏ
// Tirhuta                Tirh  𑒫𑒱𑒠𑓂𑒨𑒳𑒞𑓂
// Velthuis               Latn  vidyut
// Wx                     Latn  vixyuw
// ZanabazarSquare        Zanb  𑨭𑨁𑨛𑩇𑨪𑨂𑨙𑨴
