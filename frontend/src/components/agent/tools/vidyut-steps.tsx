import type { MouseEvent, ReactNode } from 'react';
import { Link, useNavigate } from '@tanstack/react-router';
import {
  AudioWaveformIcon,
  CombineIcon,
  GitBranchIcon,
  ShapesIcon,
  SproutIcon,
  ZapIcon,
  type LucideIcon
} from 'lucide-react';
import type { VidyutToolPart } from '../message-parts';
import { ToolShell, toolStepStatus } from './tool-shell';

const LAKARA: Record<string, string> = {
  lat: 'लट्',
  lit: 'लिट्',
  lut: 'लुट्',
  lrt: 'लृट्',
  lot: 'लोट्',
  lan: 'लङ्',
  vidhi_lin: 'विधिलिङ्',
  ashir_lin: 'आशीर्लिङ्',
  lun: 'लुङ्',
  lrn: 'लृङ्'
};
const LINGA: Record<string, string> = { pum: 'पुं', stri: 'स्त्री', napumsaka: 'नपुं' };

const META: Record<
  VidyutToolPart['type'],
  { icon: LucideIcon; running: string; done: string; error: string }
> = {
  'tool-vidyut_conjugate': {
    icon: SproutIcon,
    running: 'Conjugating',
    done: 'Conjugated',
    error: 'Conjugation failed'
  },
  'tool-vidyut_decline': {
    icon: ShapesIcon,
    running: 'Declining',
    done: 'Declined',
    error: 'Declension failed'
  },
  'tool-vidyut_krdantas': {
    icon: SproutIcon,
    running: 'Forming kṛdantas',
    done: 'Formed kṛdantas',
    error: 'Kṛdanta generation failed'
  },
  'tool-vidyut_derivation': {
    icon: GitBranchIcon,
    running: 'Deriving',
    done: 'Derived',
    error: 'Derivation failed'
  },
  'tool-vidyut_sandhi': {
    icon: CombineIcon,
    running: 'Applying sandhi',
    done: 'Applied sandhi',
    error: 'Sandhi failed'
  },
  'tool-vidyut_meter': {
    icon: AudioWaveformIcon,
    running: 'Scanning meter',
    done: 'Scanned meter',
    error: 'Meter scan failed'
  }
};

/** A site path from a tool's `page` field, navigated inside the app. */
function PageLink({ href, children }: { href: string; children: ReactNode }) {
  const navigate = useNavigate();
  const onClick = (e: MouseEvent<HTMLAnchorElement>) => {
    if (e.metaKey || e.ctrlKey || e.shiftKey || e.button !== 0) return;
    e.preventDefault();
    void navigate({ href });
  };
  return (
    <a
      href={href}
      onClick={onClick}
      className="inline-flex items-center gap-1 self-start text-xs font-medium text-primary hover:underline"
    >
      <ZapIcon className="size-3" aria-hidden /> {children}
    </a>
  );
}

function Rows({ rows }: { rows: string[] }) {
  return (
    <ul className="flex flex-col gap-0.5 font-devanagari text-xs leading-relaxed">
      {rows.map((r) => (
        <li key={r}>{r}</li>
      ))}
    </ul>
  );
}

/** What the call was about, from its (possibly still streaming) input. */
function describe(part: VidyutToolPart): string {
  const i = (part.input ?? {}) as Record<string, unknown>;
  const str = (v: unknown) => (typeof v === 'string' ? v : '');
  const list = (v: unknown, sep: string) =>
    Array.isArray(v) ? v.filter((x) => typeof x === 'string').join(sep) : '';
  switch (part.type) {
    case 'tool-vidyut_conjugate':
      return [list(i.upasargas, '+'), str(i.root), LAKARA[str(i.lakara)], str(i.prayoga)]
        .filter(Boolean)
        .join(' · ');
    case 'tool-vidyut_decline':
      return [str(i.stem), LINGA[str(i.linga)]].filter(Boolean).join(' · ');
    case 'tool-vidyut_krdantas':
      return str(i.root);
    case 'tool-vidyut_derivation':
      return str(i.form) || str(i.root) || str(i.stem);
    case 'tool-vidyut_sandhi':
      return list(i.join, ' + ') || str(i.split);
    case 'tool-vidyut_meter':
      return str(i.verse).split('\n')[0] ?? '';
  }
}

function Body({ part }: { part: VidyutToolPart }) {
  if (part.state !== 'output-available') return null;
  switch (part.type) {
    case 'tool-vidyut_conjugate': {
      const o = part.output;
      return (
        <>
          {o.tables.slice(0, 3).map((t) => (
            <div key={`${t.pada}-${t.lakara}`}>
              <p className="font-devanagari text-xs font-semibold">{LAKARA[t.lakara]}</p>
              <Rows rows={t.rows} />
            </div>
          ))}
          {o.tables.length > 3 ? (
            <p className="text-[11px] text-muted-foreground">+ {o.tables.length - 3} more tables</p>
          ) : null}
          <PageLink href={o.page}>All forms with derivations</PageLink>
        </>
      );
    }
    case 'tool-vidyut_decline':
      return (
        <>
          <Rows rows={part.output.rows} />
          <PageLink href={part.output.page}>Open in Nāma tool</PageLink>
        </>
      );
    case 'tool-vidyut_krdantas':
      return (
        <>
          {part.output.groups.map((g) => (
            <div key={g.group}>
              <p className="text-xs font-semibold">{g.group}</p>
              <Rows rows={g.items} />
            </div>
          ))}
          <PageLink href={part.output.page}>Open in Dhātu tool</PageLink>
        </>
      );
    case 'tool-vidyut_derivation':
      return (
        <ol className="flex flex-col gap-0.5 text-xs">
          {part.output.steps.map((s, i) => (
            <li key={i} className="flex gap-2">
              {s.sutra_id ? (
                <Link
                  to="/sutra/$id"
                  params={{ id: s.sutra_id }}
                  className="w-14 shrink-0 font-mono text-primary hover:underline"
                >
                  {s.rule}
                </Link>
              ) : (
                <span className="w-14 shrink-0 font-mono text-muted-foreground">{s.rule}</span>
              )}
              <span className="font-devanagari">{s.result}</span>
            </li>
          ))}
        </ol>
      );
    case 'tool-vidyut_sandhi': {
      const o = part.output;
      return (
        <>
          {'joined' in o ? (
            <p className="font-devanagari text-sm font-semibold">{o.joined.join(' / ')}</p>
          ) : (
            o.chunks.map((c) => (
              <div key={c.chunk}>
                <p className="font-devanagari text-xs font-semibold">{c.chunk}</p>
                <Rows rows={c.splits} />
              </div>
            ))
          )}
          <PageLink href={o.page}>Open in Sandhi tool</PageLink>
        </>
      );
    }
    case 'tool-vidyut_meter': {
      const o = part.output;
      return (
        <>
          <p className="font-devanagari text-sm font-semibold">{o.meter?.name ?? '—'}</p>
          <Rows rows={o.lines.map((l) => `${l.pattern} · ${l.ganas}`)} />
          <PageLink href={o.page}>Open in Chandas tool</PageLink>
        </>
      );
    }
  }
}

export function isVidyutPart(part: { type: string }): part is VidyutToolPart {
  return part.type in META;
}

/** Trace row for the vidyut_* grammar tools. */
export function VidyutStep({ part, streaming }: { part: VidyutToolPart; streaming: boolean }) {
  const status = toolStepStatus(part.state, streaming);
  const meta = META[part.type];
  return (
    <ToolShell
      icon={meta.icon}
      runningTitle={meta.running}
      doneTitle={meta.done}
      errorTitle={meta.error}
      description={describe(part)}
      status={status}
      errorText={status === 'error' && part.state === 'output-error' ? part.errorText : undefined}
    >
      {status === 'done' ? (
        <div className="flex min-w-0 flex-col gap-2">
          <Body part={part} />
        </div>
      ) : null}
    </ToolShell>
  );
}
