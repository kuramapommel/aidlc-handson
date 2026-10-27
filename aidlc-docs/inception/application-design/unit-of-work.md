# ユニット定義

## ユニット一覧

| ユニット | 内容 | 対応ストーリー |
| --- | --- | --- |
| `ec-app` | フロントエンド（React）＋ API（Hono）＋共有ロジックを含む単一アプリケーション | US-01〜US-09 |

## 分割しなかった理由

- 全体の規模が小さく、フロントエンドと API は `shared/` のロジックを共有している
- ローカル検証のみで独立デプロイの必要がない
- ワークショップとして 1 リポジトリ・1 コマンドで起動できることを優先した

## ストーリーとコンポーネントの対応

| ストーリー | 主なコンポーネント |
| --- | --- |
| US-01 商品一覧 | `ProductListPage`, `ProductCard`, `Skeleton`, `GET /api/products` |
| US-02 検索 | `ProductListPage`, `shared/catalog.ts` |
| US-03 絞り込み・並べ替え | `ProductListPage`, `shared/catalog.ts` |
| US-04 商品詳細 | `ProductDetailPage`, `GET /api/products/:id` |
| US-05 カート追加 | `CartContext`, `QuantityStepper`, `ToastContext` |
| US-06 カート編集 | `CartPage`, `CartContext` |
| US-07 送料と合計 | `ShippingProgress`, `OrderSummary`, `shared/pricing.ts` |
| US-08 入力 | `CheckoutPage`, `shared/validation.ts` |
| US-09 注文確定 | `CheckoutPage`, `OrderCompletePage`, `POST /api/orders`, `server/store.ts` |
