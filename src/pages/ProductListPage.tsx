import { useMemo } from 'react';
import { useSearchParams } from 'react-router-dom';
import { filterProducts, SORT_LABELS, type SortKey } from '../../shared/catalog';
import { CATEGORY_LABELS, type Category } from '../../shared/types';
import { EmptyState } from '../components/EmptyState';
import { ProductCard, ProductCardSkeleton } from '../components/ProductCard';
import { useProducts } from '../state/products';

const CATEGORIES = Object.keys(CATEGORY_LABELS) as Category[];
const SORT_KEYS = Object.keys(SORT_LABELS) as SortKey[];

function parseCategory(value: string | null): Category | 'all' {
  return CATEGORIES.includes(value as Category) ? (value as Category) : 'all';
}

function parseSort(value: string | null): SortKey {
  return SORT_KEYS.includes(value as SortKey) ? (value as SortKey) : 'recommended';
}

export function ProductListPage() {
  const { products, status, error, reload } = useProducts();
  const [params, setParams] = useSearchParams();
  const keyword = params.get('q') ?? '';
  const category = parseCategory(params.get('category'));
  const sort = parseSort(params.get('sort'));

  const visible = useMemo(
    () => filterProducts(products, { keyword, category, sort }),
    [products, keyword, category, sort],
  );

  const update = (key: string, value: string, defaultValue: string) => {
    setParams(
      (prev) => {
        const next = new URLSearchParams(prev);
        if (value === defaultValue) next.delete(key);
        else next.set(key, value);
        return next;
      },
      { replace: true },
    );
  };

  const resetFilters = () => setParams({}, { replace: true });
  const hasFilters = keyword !== '' || category !== 'all';

  return (
    <>
      <section className="hero">
        <h1 className="hero__title">毎日を、ちょっと心地よく。</h1>
        <p className="hero__lead">
          長く使える生活雑貨をセレクトしました。<strong>5,000 円以上のお買い上げで送料無料</strong>。
        </p>
      </section>

      <section aria-labelledby="catalog-heading" className="catalog">
        <h2 id="catalog-heading" className="visually-hidden">
          商品一覧
        </h2>

        <div className="toolbar">
          <div className="search">
            <span className="search__icon" aria-hidden="true">🔍</span>
            <input
              type="search"
              className="search__input"
              placeholder="商品を検索（例: タオル、マグ）"
              aria-label="商品を検索"
              value={keyword}
              onChange={(e) => update('q', e.target.value, '')}
            />
          </div>
          <label className="sort">
            <span className="visually-hidden">並べ替え</span>
            <select value={sort} onChange={(e) => update('sort', e.target.value, 'recommended')} aria-label="並べ替え">
              {SORT_KEYS.map((key) => (
                <option key={key} value={key}>
                  {SORT_LABELS[key]}
                </option>
              ))}
            </select>
          </label>
        </div>

        <div className="chips" role="group" aria-label="カテゴリで絞り込む">
          {(['all', ...CATEGORIES] as const).map((c) => (
            <button
              key={c}
              type="button"
              className="chip"
              aria-pressed={category === c}
              onClick={() => update('category', c, 'all')}
            >
              {c === 'all' ? 'すべて' : CATEGORY_LABELS[c]}
            </button>
          ))}
        </div>

        {status === 'error' ? (
          <EmptyState icon="📡" title="商品を読み込めませんでした">
            <p className="muted">{error}</p>
            <button type="button" className="btn btn--primary" onClick={() => void reload()}>
              再読み込み
            </button>
          </EmptyState>
        ) : status === 'loading' ? (
          <div className="grid" aria-busy="true" aria-label="商品を読み込み中">
            {Array.from({ length: 8 }, (_, i) => (
              <ProductCardSkeleton key={i} />
            ))}
          </div>
        ) : visible.length === 0 ? (
          <EmptyState icon="🔎" title="条件に合う商品が見つかりませんでした">
            <p className="muted">キーワードを変えるか、絞り込みを解除してお試しください。</p>
            <button type="button" className="btn btn--secondary" onClick={resetFilters}>
              条件をリセット
            </button>
          </EmptyState>
        ) : (
          <>
            <p className="result-count" aria-live="polite">
              {visible.length} 件の商品
              {hasFilters && (
                <button type="button" className="link-button" onClick={resetFilters}>
                  条件をリセット
                </button>
              )}
            </p>
            <div className="grid">
              {visible.map((p) => (
                <ProductCard key={p.id} product={p} />
              ))}
            </div>
          </>
        )}
      </section>
    </>
  );
}
