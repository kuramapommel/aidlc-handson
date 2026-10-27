import type { CSSProperties } from 'react';
import type { Product } from '../../shared/types';

export function ProductVisual({ product, size = 'md' }: { product: Product; size?: 'sm' | 'md' | 'lg' }) {
  return (
    <div className={`product-visual product-visual--${size}`} style={{ '--visual-color': product.color } as CSSProperties}>
      <span aria-hidden="true">{product.emoji}</span>
    </div>
  );
}
