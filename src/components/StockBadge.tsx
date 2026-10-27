export const LOW_STOCK_THRESHOLD = 5;

export function StockBadge({ stock }: { stock: number }) {
  if (stock === 0) return <span className="badge badge--soldout">在庫切れ</span>;
  if (stock <= LOW_STOCK_THRESHOLD) return <span className="badge badge--low">残りわずか（{stock} 点）</span>;
  return <span className="badge badge--ok">在庫あり</span>;
}
