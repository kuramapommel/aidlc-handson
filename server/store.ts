import type { CartLine, CheckoutForm, Order, OrderLine, PaymentMethod, Product } from '../shared/types';
import { calcTotals } from '../shared/pricing';
import { INITIAL_PRODUCTS } from './products';

export const MAX_QUANTITY_PER_LINE = 99;

export class OrderError extends Error {
  constructor(
    message: string,
    readonly status: 400 | 409,
  ) {
    super(message);
  }
}

function generateOrderId(now: Date): string {
  const date = now.toISOString().slice(0, 10).replace(/-/g, '');
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
  let suffix = '';
  for (let i = 0; i < 6; i++) suffix += chars[Math.floor(Math.random() * chars.length)];
  return `HM-${date}-${suffix}`;
}

/** 在庫と注文をプロセス内メモリで管理するストア */
export class Store {
  private products: Map<string, Product>;
  private orders = new Map<string, Order>();

  constructor(seed: readonly Product[] = INITIAL_PRODUCTS) {
    this.products = new Map(seed.map((p) => [p.id, { ...p }]));
  }

  listProducts(): Product[] {
    return [...this.products.values()].map((p) => ({ ...p }));
  }

  getProduct(id: string): Product | undefined {
    const p = this.products.get(id);
    return p && { ...p };
  }

  getOrder(id: string): Order | undefined {
    return this.orders.get(id);
  }

  /** customer は検証・正規化済みであること */
  createOrder(items: CartLine[], customer: CheckoutForm, now = new Date()): Order {
    if (items.length === 0) throw new OrderError('カートに商品がありません', 400);

    // 同一商品の明細を合算する（BR-O4）
    const merged = new Map<string, number>();
    for (const item of items) {
      if (!Number.isInteger(item.quantity) || item.quantity < 1 || item.quantity > MAX_QUANTITY_PER_LINE) {
        throw new OrderError('数量が正しくありません', 400);
      }
      merged.set(item.productId, (merged.get(item.productId) ?? 0) + item.quantity);
    }

    // すべての明細を検証してから在庫を引き当てる（BR-O5, BR-O6）
    const lines: OrderLine[] = [];
    for (const [productId, quantity] of merged) {
      const product = this.products.get(productId);
      if (!product) throw new OrderError('取り扱いのない商品が含まれています', 400);
      if (product.stock < quantity) {
        throw new OrderError(`「${product.name}」の在庫が不足しています（残り ${product.stock} 点）`, 409);
      }
      lines.push({
        productId,
        name: product.name,
        unitPrice: product.price,
        quantity,
        lineTotal: product.price * quantity,
      });
    }

    for (const line of lines) this.products.get(line.productId)!.stock -= line.quantity;

    const { subtotal, shippingFee, total } = calcTotals(lines);
    const order: Order = {
      id: generateOrderId(now),
      lines,
      subtotal,
      shippingFee,
      total,
      customer: { ...customer, paymentMethod: customer.paymentMethod as PaymentMethod },
      createdAt: now.toISOString(),
    };
    this.orders.set(order.id, order);
    return order;
  }
}
