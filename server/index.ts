import { serve } from '@hono/node-server';
import { serveStatic } from '@hono/node-server/serve-static';
import { Hono } from 'hono';
import { createApp } from './app';

const port = Number(process.env.PORT ?? 8787);
const app = new Hono();

app.route('/', createApp());

if (process.env.NODE_ENV === 'production') {
  // ビルド済みの SPA を配信し、未知のパスは index.html にフォールバックする
  app.use('/*', serveStatic({ root: './dist' }));
  app.get('*', serveStatic({ path: './dist/index.html' }));
}

serve({ fetch: app.fetch, port }, (info) => {
  console.log(`🛍️  ひびマーケット API: http://localhost:${info.port}`);
});
