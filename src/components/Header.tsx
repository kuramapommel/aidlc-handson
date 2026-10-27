import { Link, NavLink } from 'react-router-dom';
import { useCart } from '../state/cart';

export function Header() {
  const { count } = useCart();
  return (
    <header className="header">
      <div className="container header__inner">
        <Link to="/" className="logo" aria-label="ひびマーケット トップへ">
          <span className="logo__mark" aria-hidden="true">🛍️</span>
          <span className="logo__text">ひびマーケット</span>
        </Link>
        <nav aria-label="メイン">
          <NavLink to="/cart" className="cart-link" aria-label={`カート（${count} 点）`}>
            <span aria-hidden="true" className="cart-link__icon">🛒</span>
            <span className="cart-link__label" aria-hidden="true">カート</span>
            {count > 0 && (
              <span className="cart-link__badge" data-testid="cart-count" aria-hidden="true" key={count}>
                {count}
              </span>
            )}
          </NavLink>
        </nav>
      </div>
    </header>
  );
}
