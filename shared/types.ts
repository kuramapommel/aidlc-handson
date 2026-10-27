export type Category = 'kitchen' | 'living' | 'bath' | 'stationery';

export const CATEGORY_LABELS: Record<Category, string> = {
  kitchen: 'キッチン',
  living: 'リビング',
  bath: 'バス・洗面',
  stationery: '文房具',
};

export interface Product {
  id: string;
  name: string;
  description: string;
  category: Category;
  /** 税込価格（円） */
  price: number;
  rating: number;
  reviewCount: number;
  stock: number;
  emoji: string;
  /** 商品ビジュアルの背景色 */
  color: string;
}

export interface CartLine {
  productId: string;
  quantity: number;
}

export type PaymentMethod = 'card' | 'cod';

export const PAYMENT_METHOD_LABELS: Record<PaymentMethod, string> = {
  card: 'クレジットカード（デモ）',
  cod: '代金引換',
};

export interface CheckoutForm {
  name: string;
  email: string;
  postalCode: string;
  address: string;
  phone: string;
  paymentMethod: PaymentMethod | '';
}

export type CheckoutField = keyof CheckoutForm;
export type FieldErrors = Partial<Record<CheckoutField, string>>;

export interface OrderLine {
  productId: string;
  name: string;
  unitPrice: number;
  quantity: number;
  lineTotal: number;
}

export interface Order {
  id: string;
  lines: OrderLine[];
  subtotal: number;
  shippingFee: number;
  total: number;
  customer: Omit<CheckoutForm, 'paymentMethod'> & { paymentMethod: PaymentMethod };
  createdAt: string;
}

export interface CreateOrderRequest {
  items: CartLine[];
  customer: CheckoutForm;
}

export interface ApiError {
  error: string;
  fieldErrors?: FieldErrors;
}
