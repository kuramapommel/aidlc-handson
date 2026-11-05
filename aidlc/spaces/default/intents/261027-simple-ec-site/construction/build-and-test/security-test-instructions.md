# セキュリティのテストの手順

セキュリティの要件（要件定義書 NFR3、各ユニットの `nfr-requirements/security-requirements.md`）は、Web API の自動テストと、コードの検索・レビュー、ブラウザでの確認で確かめる。SAST・DAST の仕組みは使わない（ローカル検証のみ）。

## 自動テスト

```bash
cargo test -p web_api   # 入力の検証と 4xx、購入の拒否、金額の改ざん、Cookie の属性、CSRF 対策（Origin・専用ヘッダー）、CORS、他人のチケット、不正な探索条件の 400
```

## コードの検索・レビュー

| 観点 | 確かめ方 |
| --- | --- |
| 秘密情報の埋め込み | `password`・`secret`・`api_key`・`token` の代入を検索して 0 件 |
| 画面の文字列の扱い | 生の HTML を入れる書き方（innerHTML など）を検索して 0 件 |
| 業務の部品の依存 | 業務の 4 crate の依存に HTTP・データ保存がない |
| 書き込みの受け口 | CSRF 対策の下にある書き込みの受け口は従業員の切り替えと購入だけ |
| ローカル検証専用の明記 | README と `session_cookie.rs` |
| 依存の版の固定 | `Cargo.lock`・`web/moon.mod` |

## ブラウザでの確認

スクリプトを含むキーワード（`<img src=x onerror=alert(1)>`）を入力しても実行されないことを確かめる。

## 前提

Cookie の従業員 ID を認証済みとして扱うのはローカル検証専用の前提（ADR-008）。公開された環境には置かない。
