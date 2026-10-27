import { useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { CATEGORY_LABELS } from '../../shared/types';
import { FREE_SHIPPING_THRESHOLD, formatYen } from '../../shared/pricing';
import { EmptyState } from '../components/EmptyState';
import { ProductVisual } from '../components/ProductVisual';
import { QuantityStepper } from '../components/QuantityStepper';
import { Rating } from '../components/Rating';
import { StockBadge } from '../components/StockBadge';
import { maxQuantityFor, useCart } from '../state/cart';
import { useProducts } from '../state/products';
import { useToast } from '../state/toast';

export function ProductDetailPage() {
  const { id = '' } = useParams();
  const { byId, status } = useProducts();
  const cart = useCart();
  const toast = useToast();
  const [quantity, setQuantity] = useState(1);
  const product = byId(id);

  if (status === 'loading') {
    return (
      <div className="detail" aria-busy="true">
        <div className="skeleton skeleton--visual-lg" />
        <div>
          <div className="skeleton skeleton--line" style={{ width: '30%' }} />
          <div className="skeleton skeleton--title" />
          <div className="skeleton skeleton--line" style={{ width: '90%' }} />
        </div>
      </div>
    );
  }

  if (!product) {
    return (
      <EmptyState icon="🧭" title="商品が見つかりません">
        <p className="muted">お探しの商品は販売を終了したか、URL が間違っている可能性があります。</p>
        <Link to="/" className="btn btn--primary">
          商品一覧へ戻る
        </Link>
      </EmptyState>
    );
  }

  const inCart = cart.quantityOf(product.id);
  const addable = Math.max(0, maxQuantityFor(product) - inCart);
  const selected = Math.min(quantity, Math.max(1, addable));

  const handleAdd = () => {
    const added = cart.add(product, selected);
    if (added > 0) {
      toast.show(`「${product.name}」を ${added} 点カートに追加しました`);
      setQuantity(1);
    } else {
      toast.show('在庫の上限までカートに入っています', 'info');
    }
  };

  return (
    <>
      <nav aria-label="パンくずリスト" className="breadcrumb">
        <Link to="/">商品一覧</Link>
        <span aria-hidden="true">›</span>
        <Link to={`/?category=${product.category}`}>{CATEGORY_LABELS[product.category]}</Link>
        <span aria-hidden="true">›</span>
        <span aria-current="page">{product.name}</span>
      </nav>

      <article className="detail">
        <ProductVisual product={product} size="lg" />
        <div className="detail__info">
          <span className="card__category">{CATEGORY_LABELS[product.category]}</span>
          <h1 className="detail__title">{product.name}</h1>
          <Rating value={product.rating} count={product.reviewCount} />
          <p className="detail__price">
            {formatYen(product.price)} <span className="muted small">税込</span>
          </p>
          <p className="detail__desc">{product.description}</p>
          <StockBadge stock={product.stock} />

          <div className="detail__buy">
            {addable > 0 && (
              <QuantityStepper label="購入数量" value={selected} max={addable} onChange={setQuantity} />
            )}
            <button
              type="button"
              className="btn btn--primary btn--lg"
              disabled={addable === 0}
              onClick={handleAdd}
            >
              {product.stock === 0 ? '在庫切れ' : addable === 0 ? '在庫の上限までカートに入っています' : 'カートに追加'}
            </button>
          </div>
          {inCart > 0 && (
            <p className="detail__incart">
              カートに {inCart} 点入っています。<Link to="/cart">カートを見る →</Link>
            </p>
          )}
          <ul className="perks">
            <li>🚚 {formatYen(FREE_SHIPPING_THRESHOLD)} 以上で送料無料</li>
            <li>📦 ご注文から 2〜4 日でお届け</li>
            <li>↩️ 到着後 14 日以内なら返品 OK</li>
          </ul>
        </div>
      </article>
    </>
  );
}
