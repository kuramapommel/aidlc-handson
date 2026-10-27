import { Link } from 'react-router-dom';
import { calcTotals, formatYen } from '../../shared/pricing';
import type { Product } from '../../shared/types';
import { EmptyState } from '../components/EmptyState';
import { OrderSummary } from '../components/OrderSummary';
import { ProductVisual } from '../components/ProductVisual';
import { QuantityStepper } from '../components/QuantityStepper';
import { ShippingProgress } from '../components/ShippingProgress';
import { maxQuantityFor, useCart } from '../state/cart';
import { useProducts } from '../state/products';
import { useToast } from '../state/toast';

export interface ResolvedLine {
  product: Product;
  quantity: number;
}

/** カート明細を商品マスタと突き合わせる（マスタにない商品は除外: BR-C5） */
export function useResolvedCart() {
  const { lines } = useCart();
  const { byId } = useProducts();
  const resolved: ResolvedLine[] = lines.flatMap((l) => {
    const product = byId(l.productId);
    return product ? [{ product, quantity: l.quantity }] : [];
  });
  const totals = calcTotals(resolved.map((r) => ({ unitPrice: r.product.price, quantity: r.quantity })));
  return { resolved, totals };
}

export function CartPage() {
  const cart = useCart();
  const toast = useToast();
  const { status } = useProducts();
  const { resolved, totals } = useResolvedCart();

  if (status === 'loading') {
    return <div className="skeleton skeleton--block" aria-busy="true" aria-label="カートを読み込み中" />;
  }

  if (resolved.length === 0) {
    return (
      <EmptyState icon="🛒" title="カートは空です">
        <p className="muted">気になる商品を見つけて、カートに追加しましょう。</p>
        <Link to="/" className="btn btn--primary">
          お買い物を続ける
        </Link>
      </EmptyState>
    );
  }

  const overStock = resolved.some((r) => r.quantity > r.product.stock);

  return (
    <>
      <h1 className="page-title">カート</h1>
      <div className="two-col">
        <ul className="cart-list" aria-label="カート内の商品">
          {resolved.map(({ product, quantity }) => (
            <li key={product.id} className="cart-item" data-testid="cart-item">
              <Link to={`/products/${product.id}`} tabIndex={-1} aria-hidden="true">
                <ProductVisual product={product} size="sm" />
              </Link>
              <div className="cart-item__info">
                <Link to={`/products/${product.id}`} className="cart-item__name">
                  {product.name}
                </Link>
                <span className="muted small">{formatYen(product.price)} / 点</span>
                {quantity > product.stock && (
                  <span className="field-error">在庫が不足しています（残り {product.stock} 点）</span>
                )}
              </div>
              <QuantityStepper
                label={`「${product.name}」の数量`}
                value={quantity}
                max={Math.max(quantity, maxQuantityFor(product))}
                onChange={(q) => cart.setQuantity(product, q)}
              />
              <p className="cart-item__total">{formatYen(product.price * quantity)}</p>
              <button
                type="button"
                className="icon-button"
                aria-label={`「${product.name}」をカートから削除`}
                onClick={() => {
                  cart.remove(product.id);
                  toast.show(`「${product.name}」をカートから削除しました`, 'info');
                }}
              >
                🗑️
              </button>
            </li>
          ))}
        </ul>

        <aside className="panel" aria-label="ご注文金額">
          <ShippingProgress subtotal={totals.subtotal} />
          <OrderSummary totals={totals} />
          {overStock ? (
            <p className="field-error">在庫が不足している商品の数量を減らしてください。</p>
          ) : null}
          <Link
            to="/checkout"
            className={`btn btn--primary btn--lg btn--block${overStock ? ' is-disabled' : ''}`}
            aria-disabled={overStock}
            onClick={(e) => overStock && e.preventDefault()}
          >
            購入手続きへ進む
          </Link>
          <Link to="/" className="btn btn--ghost btn--block">
            お買い物を続ける
          </Link>
        </aside>
      </div>
    </>
  );
}
