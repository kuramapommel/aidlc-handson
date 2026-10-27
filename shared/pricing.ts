export const FREE_SHIPPING_THRESHOLD = 5000;
export const SHIPPING_FEE = 500;

export interface PricedLine {
  unitPrice: number;
  quantity: number;
}

export interface Totals {
  subtotal: number;
  shippingFee: number;
  total: number;
  /** 送料無料までの残額（達成済みなら 0） */
  remainingForFreeShipping: number;
}

export function calcTotals(lines: PricedLine[]): Totals {
  const subtotal = lines.reduce((sum, l) => sum + l.unitPrice * l.quantity, 0);
  const shippingFee = subtotal === 0 || subtotal >= FREE_SHIPPING_THRESHOLD ? 0 : SHIPPING_FEE;
  return {
    subtotal,
    shippingFee,
    total: subtotal + shippingFee,
    remainingForFreeShipping: Math.max(0, FREE_SHIPPING_THRESHOLD - subtotal),
  };
}

const yen = new Intl.NumberFormat('ja-JP', { style: 'currency', currency: 'JPY' });

export function formatYen(value: number): string {
  return yen.format(value);
}
