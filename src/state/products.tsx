import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import type { Product } from '../../shared/types';
import { api } from '../api';

type Status = 'loading' | 'ready' | 'error';

interface ProductsValue {
  products: Product[];
  status: Status;
  error?: string;
  byId: (id: string) => Product | undefined;
  reload: () => Promise<void>;
}

const ProductsContext = createContext<ProductsValue | null>(null);

export function ProductsProvider({ children }: { children: ReactNode }) {
  const [products, setProducts] = useState<Product[]>([]);
  const [status, setStatus] = useState<Status>('loading');
  const [error, setError] = useState<string>();

  const reload = useCallback(async () => {
    try {
      setProducts(await api.listProducts());
      setStatus('ready');
      setError(undefined);
    } catch (e) {
      setError(e instanceof Error ? e.message : '商品の読み込みに失敗しました');
      setStatus((s) => (s === 'ready' ? s : 'error'));
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  const value = useMemo<ProductsValue>(() => {
    const map = new Map(products.map((p) => [p.id, p]));
    return { products, status, error, byId: (id) => map.get(id), reload };
  }, [products, status, error, reload]);

  return <ProductsContext.Provider value={value}>{children}</ProductsContext.Provider>;
}

export function useProducts(): ProductsValue {
  const ctx = useContext(ProductsContext);
  if (!ctx) throw new Error('useProducts は ProductsProvider の内側で使用してください');
  return ctx;
}
