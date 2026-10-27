import type { CSSProperties } from 'react';
export function Rating({ value, count }: { value: number; count: number }) {
  return (
    <span className="rating" aria-label={`5 点満点中 ${value.toFixed(1)} 点（${count} 件のレビュー）`}>
      <span className="rating__stars" aria-hidden="true" style={{ '--rating': value } as CSSProperties} />
      <span aria-hidden="true" className="rating__text">
        {value.toFixed(1)} <span className="muted">({count})</span>
      </span>
    </span>
  );
}
