import type { ApiError, CreateOrderRequest, Order, Product } from '../shared/types';

export class ApiRequestError extends Error {
  constructor(
    readonly status: number,
    readonly body: ApiError,
  ) {
    super(body.error);
  }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  let res: Response;
  try {
    res = await fetch(path, {
      ...init,
      headers: { 'Content-Type': 'application/json', ...init?.headers },
    });
  } catch {
    throw new ApiRequestError(0, { error: '通信に失敗しました。ネットワーク接続をご確認ください' });
  }
  const body = await res.json().catch(() => ({ error: 'サーバーから不正な応答がありました' }));
  if (!res.ok) throw new ApiRequestError(res.status, body as ApiError);
  return body as T;
}

export const api = {
  listProducts: () => request<{ items: Product[] }>('/api/products').then((r) => r.items),
  getOrder: (id: string) => request<Order>(`/api/orders/${encodeURIComponent(id)}`),
  createOrder: (payload: CreateOrderRequest) =>
    request<Order>('/api/orders', { method: 'POST', body: JSON.stringify(payload) }),
};
