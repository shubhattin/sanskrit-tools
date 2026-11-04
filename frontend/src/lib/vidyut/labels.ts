// Devanagari names for the grammatical categories the vidyut API uses as ids.
import type {
  LakaraId,
  PadaInfo,
  LingaId,
  PadaId,
  PrayogaId,
  PurushaId,
  SamasaKind,
  SanadiId,
  VacanaId,
  VibhaktiId
} from './api';

export const LAKARAS: { id: LakaraId; label: string; hint: string }[] = [
  { id: 'lat', label: 'लट्', hint: 'present' },
  { id: 'lit', label: 'लिट्', hint: 'perfect' },
  { id: 'lut', label: 'लुट्', hint: 'periphrastic future' },
  { id: 'lrt', label: 'लृट्', hint: 'simple future' },
  { id: 'lot', label: 'लोट्', hint: 'imperative' },
  { id: 'lan', label: 'लङ्', hint: 'imperfect' },
  { id: 'vidhi_lin', label: 'विधिलिङ्', hint: 'optative' },
  { id: 'ashir_lin', label: 'आशीर्लिङ्', hint: 'benedictive' },
  { id: 'lun', label: 'लुङ्', hint: 'aorist' },
  { id: 'lrn', label: 'लृङ्', hint: 'conditional' }
];
export const LAKARA_LABEL = Object.fromEntries(LAKARAS.map((l) => [l.id, l.label])) as Record<
  LakaraId,
  string
>;

export const PRAYOGAS: { id: PrayogaId; label: string; hint: string }[] = [
  { id: 'kartari', label: 'कर्तरि', hint: 'active' },
  { id: 'karmani', label: 'कर्मणि', hint: 'passive' },
  { id: 'bhave', label: 'भावे', hint: 'impersonal' }
];

export const PADA_LABEL: Record<PadaId, string> = {
  parasmaipada: 'परस्मैपदम्',
  atmanepada: 'आत्मनेपदम्'
};

export const PURUSHAS: { id: PurushaId; label: string }[] = [
  { id: 'prathama', label: 'प्रथमपुरुषः' },
  { id: 'madhyama', label: 'मध्यमपुरुषः' },
  { id: 'uttama', label: 'उत्तमपुरुषः' }
];

export const VACANAS: { id: VacanaId; label: string }[] = [
  { id: 'eka', label: 'एकवचनम्' },
  { id: 'dvi', label: 'द्विवचनम्' },
  { id: 'bahu', label: 'बहुवचनम्' }
];

export const VIBHAKTIS: { id: VibhaktiId; label: string }[] = [
  { id: 'prathama', label: 'प्रथमा' },
  { id: 'dvitiya', label: 'द्वितीया' },
  { id: 'trtiya', label: 'तृतीया' },
  { id: 'caturthi', label: 'चतुर्थी' },
  { id: 'panchami', label: 'पञ्चमी' },
  { id: 'sasthi', label: 'षष्ठी' },
  { id: 'saptami', label: 'सप्तमी' },
  { id: 'sambodhana', label: 'सम्बोधनम्' }
];

export const LINGAS: { id: LingaId; label: string; short: string }[] = [
  { id: 'pum', label: 'पुंलिङ्गम्', short: 'पुं' },
  { id: 'stri', label: 'स्त्रीलिङ्गम्', short: 'स्त्री' },
  { id: 'napumsaka', label: 'नपुंसकलिङ्गम्', short: 'नपुं' }
];
export const LINGA_LABEL = Object.fromEntries(LINGAS.map((l) => [l.id, l.label])) as Record<
  LingaId,
  string
>;

export const SANADIS: { id: SanadiId; label: string; hint: string }[] = [
  { id: 'nic', label: 'णिच्', hint: 'causative' },
  { id: 'san', label: 'सन्', hint: 'desiderative' },
  { id: 'yan', label: 'यङ्', hint: 'intensive' },
  { id: 'yan_luk', label: 'यङ्लुक्', hint: 'intensive' }
];
export const SANADI_LABEL = Object.fromEntries(SANADIS.map((s) => [s.id, s.label])) as Record<
  SanadiId,
  string
>;

export const UPASARGAS = [
  'प्र',
  'परा',
  'अप',
  'सम्',
  'अनु',
  'अव',
  'निस्',
  'निर्',
  'दुस्',
  'दुर्',
  'वि',
  'आङ्',
  'नि',
  'अधि',
  'अपि',
  'अति',
  'सु',
  'उद्',
  'अभि',
  'प्रति',
  'परि',
  'उप'
] as const;

export const SAMASAS: { id: SamasaKind; label: string; hint: string }[] = [
  { id: 'tatpurusha', label: 'तत्पुरुषः', hint: 'determinative' },
  { id: 'karmadharaya', label: 'कर्मधारयः', hint: 'descriptive' },
  { id: 'dvigu', label: 'द्विगुः', hint: 'numeral' },
  { id: 'bahuvrihi', label: 'बहुव्रीहिः', hint: 'possessive' },
  { id: 'dvandva', label: 'द्वन्द्वः', hint: 'copulative' },
  { id: 'samahara_dvandva', label: 'समाहारद्वन्द्वः', hint: 'collective' },
  { id: 'avyayibhava', label: 'अव्ययीभावः', hint: 'adverbial' }
];

export const GANAS = [
  'भ्वादिः',
  'अदादिः',
  'जुहोत्यादिः',
  'दिवादिः',
  'स्वादिः',
  'तुदादिः',
  'रुधादिः',
  'तनादिः',
  'क्र्यादिः',
  'चुरादिः'
] as const;

const DIGITS = '०१२३४५६७८९';
/** 12 → १२ */
export function devaNum(n: number | string): string {
  return String(n).replace(/[0-9]/g, (d) => DIGITS[Number(d)]!);
}

/** Site Dhātupāṭha number → vidyut code: `1.10` → `01.0010`. Keeps URLs stable (TanStack Router
 * would read `?code=1.10` as the number 1.1). */
export function padCode(number: string): string {
  const [g, n] = number.split('.');
  return `${(g ?? '').padStart(2, '0')}.${(n ?? '').padStart(4, '0')}`;
}

const VACANA_SHORT: Record<VacanaId, string> = { eka: 'एक॰', dvi: 'द्वि॰', bahu: 'बहु॰' };
const PURUSHA_SHORT: Record<PurushaId, string> = {
  prathama: 'प्रथम॰',
  madhyama: 'मध्यम॰',
  uttama: 'उत्तम॰'
};
const VIBHAKTI_LABEL = Object.fromEntries(VIBHAKTIS.map((v) => [v.id, v.label])) as Record<
  VibhaktiId,
  string
>;
const LINGA_SHORT = Object.fromEntries(LINGAS.map((l) => [l.id, l.short])) as Record<
  LingaId,
  string
>;

/** A word's stem with its kṛt origin: `उद्भवन (उद्भू + ल्युट्)`. */
export function padaLemma(a: PadaInfo): string {
  return a.dhatu && a.krt ? `${a.lemma} (${a.dhatu} + ${a.krt})` : a.lemma;
}

/** A form's grammar in brief: `नपुं प्रथमा एक॰`, `लट् प्रथम॰ एक॰`, `अव्ययम्`. */
export function padaTags(a: PadaInfo): string {
  if (a.kind === 'avyaya') return 'अव्ययम्';
  if (a.kind === 'tinanta')
    return [
      a.lakara ? LAKARA_LABEL[a.lakara] : null,
      a.purusha ? PURUSHA_SHORT[a.purusha] : null,
      a.vacana ? VACANA_SHORT[a.vacana] : null,
      a.prayoga && a.prayoga !== 'kartari' ? PRAYOGAS.find((p) => p.id === a.prayoga)?.label : null
    ]
      .filter(Boolean)
      .join(' ');
  return [
    a.linga ? LINGA_SHORT[a.linga] : null,
    a.vibhakti ? VIBHAKTI_LABEL[a.vibhakti] : null,
    a.vacana ? VACANA_SHORT[a.vacana] : null
  ]
    .filter(Boolean)
    .join(' ');
}
