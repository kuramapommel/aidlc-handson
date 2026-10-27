import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import type { CartLine, Product } from '../../shared/types';

export const CART_STORAGE_KEY = 'hibi-market:cart:v1';
export const MAX_QUANTITY = 99;

function loadCart(): CartLine[] {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(CART_STORAGE_KEY) ?? '[]');
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (l): l is CartLine =>
        typeof l?.productId === 'string' && Number.isInteger(l?.quantity) && l.quantity > 0,
    );
  } catch {
    return [];
  }
}

export function maxQuantityFor(product: Product): number {
  return Math.min(product.stock, MAX_QUANTITY);
}

interface CartValue {
  lines: CartLine[];
  /** カート内の合計点数 */
  count: number;
  quantityOf: (productId: string) => number;
  /** 実際に追加できた数量を返す（在庫上限で丸める） */
  add: (product: Product, quantity: number) => number;
  setQuantity: (product: Product, quantity: number) => void;
  remove: (productId: string) => void;
  clear: () => void;
}

const CartContext = createContext<CartValue | null>(null);

export function CartProvider({ children }: { children: ReactNode }) {
  const [lines, setLines] = useState<CartLine[]>(loadCart);

  useEffect(() => {
    try {
      localStorage.setItem(CART_STORAGE_KEY, JSON.stringify(lines));
    } catch {
      // プライベートモード等で保存できない場合はメモリ上のカートのみで動作する
    }
  }, [lines]);

  // 別タブでの変更を反映する
  useEffect(() => {
    const onStorage = (e: StorageEvent) => {
      if (e.key === CART_STORAGE_KEY) setLines(loadCart());
    };
    window.addEventListener('storage', onStorage);
    return () => window.removeEventListener('storage', onStorage);
  }, []);

  const quantityOf = useCallback(
    (productId: string) => lines.find((l) => l.productId === productId)?.quantity ?? 0,
    [lines],
  );

  const add = useCallback(
    (product: Product, quantity: number) => {
      const current = quantityOf(product.id);
      const next = Math.min(current + quantity, maxQuantityFor(product));
      const added = Math.max(0, next - current);
      if (added > 0) {
        setLines((prev) =>
          current > 0
            ? prev.map((l) => (l.productId === product.id ? { ...l, quantity: next } : l))
            : [...prev, { productId: product.id, quantity: next }],
        );
      }
      return added;
    },
    [quantityOf],
  );

  const setQuantity = useCallback((product: Product, quantity: number) => {
    const q = Math.max(1, Math.min(Math.floor(quantity), maxQuantityFor(product)));
    setLines((prev) => prev.map((l) => (l.productId === product.id ? { ...l, quantity: q } : l)));
  }, []);

  const remove = useCallback((productId: string) => {
    setLines((prev) => prev.filter((l) => l.productId !== productId));
  }, []);

  const clear = useCallback(() => setLines([]), []);

  const value = useMemo<CartValue>(
    () => ({
      lines,
      count: lines.reduce((sum, l) => sum + l.quantity, 0),
      quantityOf,
      add,
      setQuantity,
      remove,
      clear,
    }),
    [lines, quantityOf, add, setQuantity, remove, clear],
  );

  return <CartContext.Provider value={value}>{children}</CartContext.Provider>;
}

export function useCart(): CartValue {
  const ctx = useContext(CartContext);
  if (!ctx) throw new Error('useCart は CartProvider の内側で使用してください');
  return ctx;
}
