import type { ReactNode } from 'react';
import type { WordSpec } from '@/lib/vidyut/api';
import { FormButton } from './derivation';
import { cn } from '@/lib/utils';

/**
 * A paradigm grid (rows × eka / dvi / bahu). Each form opens its derivation: `specFor` builds the
 * request for a cell, `contextFor` its caption in the derivation sheet.
 */
export function FormsTable({
  caption,
  rows,
  cols,
  cells,
  specFor,
  contextFor,
  className,
  dense
}: {
  caption?: ReactNode;
  rows: string[];
  cols: string[];
  cells: string[][][];
  specFor: (row: number, col: number) => WordSpec;
  contextFor: (row: number, col: number) => string;
  className?: string;
  dense?: boolean;
}) {
  return (
    <figure className={cn('flex min-w-0 flex-col gap-1.5', className)}>
      {caption ? <figcaption className="text-sm font-semibold">{caption}</figcaption> : null}
      <div className="overflow-x-auto rounded-xl border bg-card">
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="bg-muted/50">
              <th scope="col" className="w-0 px-2 py-1.5" />
              {cols.map((c) => (
                <th
                  key={c}
                  scope="col"
                  className="px-2 py-1.5 text-left font-devanagari text-xs font-semibold whitespace-nowrap text-muted-foreground"
                >
                  {c}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {cells.map((row, ri) => (
              <tr key={ri} className="border-t">
                <th
                  scope="row"
                  className={cn(
                    'px-2 text-left font-devanagari text-xs font-medium whitespace-nowrap text-muted-foreground',
                    dense ? 'py-1' : 'py-1.5'
                  )}
                >
                  {rows[ri]}
                </th>
                {row.map((forms, ci) => (
                  <td key={ci} className={cn('px-2 align-top', dense ? 'py-1' : 'py-1.5')}>
                    {forms.length === 0 ? (
                      <span className="text-muted-foreground/60">—</span>
                    ) : (
                      <span className="flex flex-col items-start gap-0.5">
                        {forms.map((f) => (
                          <FormButton
                            key={f}
                            form={f}
                            spec={specFor(ri, ci)}
                            context={contextFor(ri, ci)}
                            className="text-[15px] leading-snug"
                          />
                        ))}
                      </span>
                    )}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </figure>
  );
}
