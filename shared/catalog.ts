import type { Category, Product } from './types';

export type SortKey = 'recommended' | 'price-asc' | 'price-desc' | 'rating';

export const SORT_LABELS: Record<SortKey, string> = {
  recommended: 'おすすめ順',
  'price-asc': '価格の安い順',
  'price-desc': '価格の高い順',
  rating: '評価の高い順',
};

export interface CatalogQuery {
  keyword?: string;
  category?: Category | 'all';
  sort?: SortKey;
}

function normalize(text: string): string {
  return text.normalize('NFKC').toLowerCase();
}

export function filterProducts(products: Product[], query: CatalogQuery): Product[] {
  const keywords = normalize(query.keyword ?? '')
    .split(/\s+/)
    .filter(Boolean);
  const category = query.category ?? 'all';

  const filtered = products.filter((p) => {
    if (category !== 'all' && p.category !== category) return false;
    const haystack = normalize(`${p.name} ${p.description}`);
    return keywords.every((k) => haystack.includes(k));
  });

  switch (query.sort ?? 'recommended') {
    case 'price-asc':
      return [...filtered].sort((a, b) => a.price - b.price);
    case 'price-desc':
      return [...filtered].sort((a, b) => b.price - a.price);
    case 'rating':
      return [...filtered].sort((a, b) => b.rating - a.rating || b.reviewCount - a.reviewCount);
    default:
      return filtered;
  }
}
