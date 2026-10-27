import type { ReactNode } from 'react';
import { formatYen, type Totals } from '../../shared/pricing';

export function OrderSummary({ totals, children }: { totals: Pick<Totals, 'subtotal' | 'shippingFee' | 'total'>; children?: ReactNode }) {
  return (
    <dl className="summary">
      <div className="summary__row">
        <dt>小計</dt>
        <dd data-testid="subtotal">{formatYen(totals.subtotal)}</dd>
      </div>
      <div className="summary__row">
        <dt>送料</dt>
        <dd data-testid="shipping-fee">{totals.shippingFee === 0 ? '無料' : formatYen(totals.shippingFee)}</dd>
      </div>
      <div className="summary__row summary__row--total">
        <dt>合計（税込）</dt>
        <dd data-testid="total">{formatYen(totals.total)}</dd>
      </div>
      {children}
    </dl>
  );
}
