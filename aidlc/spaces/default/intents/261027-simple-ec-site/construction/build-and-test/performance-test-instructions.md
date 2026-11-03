# 性能テストの手順

性能の目標（要件定義書 NFR1、各ユニットの `nfr-requirements/performance-requirements.md`）は、Web API の自動テストで測る。負荷試験の仕組みは使わない（ローカル検証のみ、初期データの規模）。

## 実行

```bash
cargo test -p web_api --test performance -- --nocapture   # 一覧・詳細・購入・操作中の従業員・チケット詳細（各 100 回）
cargo test -p web_api --test search -- --nocapture        # 探索（条件を替えた 100 回）と数万文字のキーワード
cargo test -p web_api --test my_tickets -- --nocapture    # 購入済みチケットの一覧（100 回）
```

各テストは 95 パーセンタイルが 200 ms 以内であることを確かめ、`--nocapture` で測った値を表示する。

## 画面の反映の時間

キーワードの反映（入力から 300 ms 以内、U2 NFR1.2）はブラウザで測る（`integration-test-instructions.md` のブラウザでの確認）。

## 目標

| 対象 | 目標 |
| --- | --- |
| Web API の各受け口 | 95% の要求が 200 ms 以内 |
| 数万文字のキーワード | 200 ms 以内 |
| 画面の条件の反映 | 300 ms 以内 |
