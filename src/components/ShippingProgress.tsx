import { FREE_SHIPPING_THRESHOLD, formatYen } from '../../shared/pricing';

export function ShippingProgress({ subtotal }: { subtotal: number }) {
  const remaining = Math.max(0, FREE_SHIPPING_THRESHOLD - subtotal);
  const ratio = Math.min(1, subtotal / FREE_SHIPPING_THRESHOLD);
  return (
    <div className={`shipping-progress${remaining === 0 ? ' is-free' : ''}`}>
      <p className="shipping-progress__text" data-testid="shipping-message">
        {remaining === 0 ? (
          <>🎉 送料無料でお届けします</>
        ) : (
          <>
            あと <strong>{formatYen(remaining)}</strong> のお買い上げで<strong>送料無料</strong>
          </>
        )}
      </p>
      <div
        className="shipping-progress__bar"
        role="progressbar"
        aria-label="送料無料までの進捗"
        aria-valuemin={0}
        aria-valuemax={FREE_SHIPPING_THRESHOLD}
        aria-valuenow={Math.min(subtotal, FREE_SHIPPING_THRESHOLD)}
      >
        <span style={{ width: `${ratio * 100}%` }} />
      </div>
    </div>
  );
}
