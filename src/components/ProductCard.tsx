import { Link } from 'react-router-dom';
import type { Product } from '../../shared/types';
import { CATEGORY_LABELS } from '../../shared/types';
import { formatYen } from '../../shared/pricing';
import { maxQuantityFor, useCart } from '../state/cart';
import { useToast } from '../state/toast';
import { ProductVisual } from './ProductVisual';
import { Rating } from './Rating';
import { StockBadge } from './StockBadge';

export function ProductCard({ product }: { product: Product }) {
  const cart = useCart();
  const toast = useToast();
  const inCart = cart.quantityOf(product.id);
  const reachedLimit = inCart >= maxQuantityFor(product);

  const handleAdd = () => {
    const added = cart.add(product, 1);
    if (added > 0) toast.show(`「${product.name}」をカートに追加しました`);
    else toast.show('在庫の上限までカートに入っています', 'info');
  };

  return (
    <article className="card" data-testid="product-card">
      <Link to={`/products/${product.id}`} className="card__link">
        <ProductVisual product={product} />
        <div className="card__body">
          <span className="card__category">{CATEGORY_LABELS[product.category]}</span>
          <h3 className="card__title">{product.name}</h3>
          <Rating value={product.rating} count={product.reviewCount} />
        </div>
      </Link>
      <div className="card__footer">
        <div>
          <p className="price">{formatYen(product.price)}</p>
          <StockBadge stock={product.stock} />
        </div>
        <button
          type="button"
          className="btn btn--primary btn--sm"
          onClick={handleAdd}
          disabled={product.stock === 0 || reachedLimit}
          aria-label={`「${product.name}」をカートに追加`}
        >
          {product.stock === 0 ? '在庫切れ' : inCart > 0 ? `追加（${inCart}）` : 'カートに追加'}
        </button>
      </div>
    </article>
  );
}

export function ProductCardSkeleton() {
  return (
    <div className="card card--skeleton" aria-hidden="true">
      <div className="skeleton skeleton--visual" />
      <div className="card__body">
        <div className="skeleton skeleton--line" style={{ width: '40%' }} />
        <div className="skeleton skeleton--line" style={{ width: '85%' }} />
        <div className="skeleton skeleton--line" style={{ width: '55%' }} />
      </div>
    </div>
  );
}
