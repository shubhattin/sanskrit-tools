import { AudioWaveformIcon, CombineIcon, ShapesIcon, SproutIcon, ZapIcon } from 'lucide-react';
import { cn } from '@/lib/utils';

export const TOOLS = [
  {
    to: '/vidyut/dhatu',
    label: 'Dhātu',
    deva: 'धातुरूपाणि',
    icon: SproutIcon,
    blurb:
      'Conjugate any root of the Dhātupāṭha in every lakāra and prayoga, with causative, desiderative and upasargas, and form its participles, gerunds and infinitives.'
  },
  {
    to: '/vidyut/nama',
    label: 'Nāma',
    deva: 'शब्दरूपाणि',
    icon: ShapesIcon,
    blurb:
      'Decline a noun stem in any gender, form its feminine, its taddhita derivatives, and compounds with other stems.'
  },
  {
    to: '/vidyut/chandas',
    label: 'Chandas',
    deva: 'छन्दः',
    icon: AudioWaveformIcon,
    blurb:
      'Scan a verse into laghu and guru syllables and gaṇas, find its meter, and browse the meters of the Vṛttaratnākara.'
  },
  {
    to: '/vidyut/sandhi',
    label: 'Sandhi',
    deva: 'सन्धिः',
    icon: CombineIcon,
    blurb:
      'Join words with the rules of sandhi, see the ways a joined word can be split, and look up the rule table.'
  }
] as const;

export function VidyutMark({ className }: { className?: string }) {
  return (
    <span
      aria-hidden
      className={cn(
        'grid size-9 shrink-0 place-items-center rounded-xl bg-gradient-to-br from-amber-400 via-orange-500 to-rose-500 text-white shadow-md shadow-orange-500/30',
        className
      )}
    >
      <ZapIcon className="size-5" fill="currentColor" />
    </span>
  );
}

/** Thanks to Ambuda, whose open-source vidyut powers these tools. */
export function AmbudaCredit({ className }: { className?: string }) {
  return (
    <p className={cn('text-xs leading-relaxed text-muted-foreground', className)}>
      These tools run on{' '}
      <a
        href="https://github.com/ambuda-org/vidyut"
        target="_blank"
        rel="noreferrer"
        className="font-medium text-foreground underline underline-offset-2"
      >
        vidyut
      </a>
      , the open-source Sanskrit toolkit from the{' '}
      <a
        href="https://ambuda.org/"
        target="_blank"
        rel="noreferrer"
        className="font-medium text-foreground underline underline-offset-2"
      >
        Ambuda
      </a>{' '}
      project. Our thanks to Ambuda for building it and sharing it freely.
    </p>
  );
}
