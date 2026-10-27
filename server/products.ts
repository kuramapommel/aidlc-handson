import type { Product } from '../shared/types';

export const INITIAL_PRODUCTS: readonly Product[] = [
  { id: 'p-01', name: '波佐見焼のマグカップ', description: '手になじむ丸いフォルム。電子レンジ・食洗機に対応した毎日使えるマグです。', category: 'kitchen', price: 1980, rating: 4.7, reviewCount: 128, stock: 24, emoji: '☕', color: '#fde68a' },
  { id: 'p-02', name: 'ひのきのまな板', description: '抗菌性に優れた国産ひのき製。軽くて扱いやすく、包丁あたりもやさしい。', category: 'kitchen', price: 3300, rating: 4.5, reviewCount: 64, stock: 12, emoji: '🪵', color: '#fed7aa' },
  { id: 'p-03', name: 'ステンレス保温ボトル 350ml', description: '6 時間後も 70℃ 以上をキープ。バッグに収まるスリムサイズ。', category: 'kitchen', price: 2750, rating: 4.4, reviewCount: 210, stock: 3, emoji: '🧴', color: '#bae6fd' },
  { id: 'p-04', name: 'リネンのキッチンクロス', description: '吸水性と速乾性に優れたリネン 100%。使うほど柔らかくなります。', category: 'kitchen', price: 1210, rating: 4.2, reviewCount: 45, stock: 40, emoji: '🧺', color: '#e9d5ff' },
  { id: 'p-05', name: 'ウールのブランケット', description: 'ふんわり軽いメリノウール。ソファでもオフィスでも活躍します。', category: 'living', price: 6600, rating: 4.8, reviewCount: 89, stock: 8, emoji: '🧣', color: '#fecaca' },
  { id: 'p-06', name: 'アロマディフューザー', description: '超音波式で静かにミストを広げます。やさしい間接照明つき。', category: 'living', price: 4980, rating: 4.3, reviewCount: 152, stock: 15, emoji: '🌿', color: '#bbf7d0' },
  { id: 'p-07', name: '陶器のフラワーベース', description: '一輪挿しにちょうどいいサイズ。マットな質感がどんな部屋にもなじみます。', category: 'living', price: 2420, rating: 4.6, reviewCount: 37, stock: 0, emoji: '🌷', color: '#fbcfe8' },
  { id: 'p-08', name: 'オーガニックコットンのバスタオル', description: '厚手でふかふか。洗濯を重ねても吸水性が落ちにくいタオルです。', category: 'bath', price: 3520, rating: 4.7, reviewCount: 301, stock: 30, emoji: '🛁', color: '#c7d2fe' },
  { id: 'p-09', name: '竹の歯ブラシ 4 本セット', description: '持ち手は生分解性の竹。やわらかめの毛先で歯ぐきにやさしい。', category: 'bath', price: 880, rating: 4.1, reviewCount: 76, stock: 50, emoji: '🪥', color: '#d9f99d' },
  { id: 'p-10', name: '無添加せっけん', description: 'こだわりの釜焚き製法。しっとり洗い上がる、家族みんなで使えるせっけん。', category: 'bath', price: 660, rating: 4.5, reviewCount: 188, stock: 4, emoji: '🧼', color: '#a5f3fc' },
  { id: 'p-11', name: '方眼ノート A5', description: '書き心地のよい国産紙。罫線が薄く、図や表も書きやすい方眼タイプ。', category: 'stationery', price: 550, rating: 4.4, reviewCount: 95, stock: 60, emoji: '📓', color: '#e5e7eb' },
  { id: 'p-12', name: '真鍮のボールペン', description: 'ずっしりとした重みで安定した書き味。使うほどに味わいが増します。', category: 'stationery', price: 3850, rating: 4.9, reviewCount: 42, stock: 10, emoji: '🖊️', color: '#fef08a' },
];
