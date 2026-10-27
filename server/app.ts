import { Hono } from 'hono';
import type { CartLine, CheckoutForm } from '../shared/types';
import { normalizeCheckout, validateCheckout } from '../shared/validation';
import { OrderError, Store } from './store';

function isCartLines(value: unknown): value is CartLine[] {
  return (
    Array.isArray(value) &&
    value.every(
      (v) =>
        typeof v === 'object' &&
        v !== null &&
        typeof (v as CartLine).productId === 'string' &&
        typeof (v as CartLine).quantity === 'number',
    )
  );
}

export function createApp(store = new Store()) {
  const app = new Hono().basePath('/api');

  app.get('/products', (c) => c.json({ items: store.listProducts() }));

  app.get('/products/:id', (c) => {
    const product = store.getProduct(c.req.param('id'));
    return product ? c.json(product) : c.json({ error: '商品が見つかりません' }, 404);
  });

  app.get('/orders/:id', (c) => {
    const order = store.getOrder(c.req.param('id'));
    return order ? c.json(order) : c.json({ error: '注文が見つかりません' }, 404);
  });

  app.post('/orders', async (c) => {
    const body = await c.req.json().catch(() => null);
    if (!body || typeof body !== 'object' || !isCartLines(body.items)) {
      return c.json({ error: 'リクエストの形式が正しくありません' }, 400);
    }
    const customer = (typeof body.customer === 'object' && body.customer) || {};
    const fieldErrors = validateCheckout(customer);
    if (Object.keys(fieldErrors).length > 0) {
      return c.json({ error: '入力内容に誤りがあります', fieldErrors }, 400);
    }
    try {
      const order = store.createOrder(body.items, normalizeCheckout(customer as CheckoutForm));
      return c.json(order, 201);
    } catch (e) {
      if (e instanceof OrderError) return c.json({ error: e.message }, e.status);
      throw e;
    }
  });

  return app;
}
