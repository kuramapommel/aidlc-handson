# 可観測性の要件（U3 購入済みチケット）

要件定義書（`../../../inception/requirements-analysis/requirements.md`）の NFR10（可観測性（最小））を、U3 の購入済みチケットに当てはめる。U1 の要件（`../../u1-purchase-skeleton/nfr-requirements/observability-requirements.md`。要求ごとの 1 行のログなど）はそのまま引き継ぐ。この工程の決定のまとめは `nfr-requirements-questions.md` で確認済み（2026-11-03）。

## 要件

| ID | 要件 | 確かめ方 |
| --- | --- | --- |
| NFR10.1 | 一覧の要求も、U1 と同じく要求ごとに 1 行のログ（メソッド・パス・状態コード・処理時間）を出す | Web API を起動して目視 |
| NFR10.2 | チケット番号・従業員名・コース名など一覧の内容はログに出さない（従業員 ID も U1 と同じ扱いにとどめる） | 目視とコードレビュー |
| NFR10.3 | チケットの日程やコースが見つからないデータの不整合で 5xx にしたときは、どのチケット ID で起きたかをログに残す | Web API の自動テスト、コードレビュー |

## 参照した資料

U3 の NFR 要件（この文書と、同じフォルダーの性能・セキュリティ・拡張性・信頼性の要件、技術の選定）は、次の資料をもとにした。

- 要件定義書: `../../../inception/requirements-analysis/requirements.md`（NFR1〜NFR10、C5）
- U3 の機能仕様: `../functional-design/functional-spec.md`（利用の流れと状態の遷移）
- U3 の業務ルール: `../functional-design/rules.md`（BR12〜BR14）
- U1・U2 の NFR 要件: `../../u1-purchase-skeleton/nfr-requirements/`、`../../u2-course-search/nfr-requirements/`
