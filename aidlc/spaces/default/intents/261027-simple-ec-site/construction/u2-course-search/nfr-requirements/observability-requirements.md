# 可観測性の要件（U2 コース探索）

要件定義書（`../../../inception/requirements-analysis/requirements.md`）の NFR10（可観測性（最小））を、U2 の探索に当てはめる。U1 の要件（`../../u1-purchase-skeleton/nfr-requirements/observability-requirements.md`。要求ごとの 1 行のログなど）はそのまま引き継ぐ。この工程の決定のまとめは `nfr-requirements-questions.md` で確認済み（2026-10-30）。

## 要件

| ID | 要件 | 確かめ方 |
| --- | --- | --- |
| NFR10.1 | 探索の要求も、U1 と同じく要求ごとに 1 行のログ（メソッド・パス・状態コード・処理時間）を出す。パスにはクエリを含めない（キーワードをログに残さない） | Web API を起動して目視 |
| NFR10.2 | 探索の 400 は、不正な条件の項目名（category・period など）だけをログに出す。値そのもの（キーワードや日付の文字列）は出さない | 目視とコードレビュー |
| NFR10.3 | 詳細レベルのログでは、探索の結果の件数と処理時間を出せる（既定の情報レベルでは出さない） | 起動時の設定を変えて目視 |

## 参照した資料

U2 の NFR 要件（この文書と、同じフォルダーの性能・セキュリティ・拡張性・信頼性の要件、技術の選定）は、次の資料をもとにした。

- 要件定義書: `../../../inception/requirements-analysis/requirements.md`（NFR1〜NFR10、C5）
- U2 の機能仕様: `../functional-design/functional-spec.md`（利用の流れと状態の遷移）
- U2 の業務ルール: `../functional-design/rules.md`（BR8〜BR11）
- U1 の NFR 要件: `../../u1-purchase-skeleton/nfr-requirements/`
