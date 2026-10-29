# 研修コース e チケット購入サイト（ローカル検証専用）

契約企業の従業員が、研修予算（ポイント）で研修コースの受講用 e チケットを購入するサイトです。カートは持たず、1 回の購入で 1 つの日程を購入します。

> [!WARNING]
> **ローカル検証専用です。公開された環境に置いてはなりません。**
> ログイン機能はなく、Cookie に入った従業員 ID をそのまま認証済みの従業員として扱います（ADR-008）。Cookie の値を書き換えれば他人として操作できます。Web API と静的ファイルサーバーは 127.0.0.1 だけで待ち受けます。

## 現在の状況（U1 購入の最小の流れ）

| 部分 | 状況 |
| --- | --- |
| Rust の業務の部品・データストア・Web API・静的ファイルサーバー | 実装済み。テストはすべて成功 |
| MoonBit の画面（`web/`） | **未実装（Step 13 で停止）**。`web/` に MoonBit のプロジェクトを作り、後継の TEA ライブラリ `moonbit-community/rabbita` の最新版 0.17.0 を追加した（ビルドは通る）。ただし rabbita の通信の部品（`@http`）は、fetch に `credentials` を渡さず、利用側がヘッダーを足す口もなく、2xx 以外の応答の本文を捨てる。そのため、別オリジンの Web API への Cookie 付きの通信、CSRF 対策の専用ヘッダー（`X-Requested-With`）、409 の拒否の理由の受け取りができない。ヘッドレスの Chromium で実際に確かめた（下の「画面の TEA ライブラリの確認結果」）。TEA を自作しない方針のため、ここで止めて判断を仰いでいる |

そのため `bash scripts/verify.sh` は、MoonBit の画面のテストの段階で失敗で終わります（成功と偽らないため）。画面ができるまでは、下の「デモの手順」のとおり `curl` で Web API を確かめられます。

### 画面の TEA ライブラリの確認結果（2026-10-29、rabbita 0.17.0）

| 確かめたこと | 結果 |
| --- | --- |
| rabbita の最新版の追加とビルド（`moon add moonbit-community/rabbita`、`moon build --target js`） | できる |
| `@http.get` / `@http.post` で、画面（`http://localhost:8080`）から Web API（`http://localhost:3000`）へ Cookie を送る | **できない**。fetch の `credentials` が既定（same-origin）のままで、`Set-Cookie` も保存されない |
| 書き込みの要求に `X-Requested-With: aidlc-handson-web` を付ける | **できない**。ヘッダーを指定する API がなく、Web API は 403 を返す |
| 409（購入の拒否）の本文から拒否の理由を読む | **できない**。2xx 以外は `HTTP 409 Conflict` という文字列だけになる |
| 対照: 同じページから素の `fetch`（`credentials: "include"`＋専用ヘッダー） | Cookie の発行・送信、従業員の切り替え、409 と理由の受け取りまですべて動く（Web API 側の CORS と CSRF 対策の設定は正しい） |

## 前提のツール

| ツール | バージョン | 導入 |
| --- | --- | --- |
| Rust（rustc・cargo・rustfmt・clippy） | 最新の stable（1.97.0 で確認） | <https://rustup.rs/> |
| MoonBit（moon） | 最新版（moon 0.1.20261028・moonc v0.10.18 で確認） | `curl -fsSL https://cli.moonbitlang.com/install/unix.sh \| bash`（`~/.moon/bin` に入ります） |
| Node.js | 22 系（v22.22.0 で確認） | **画面のテスト（`moon -C web test --target js`）の実行にだけ必要**。moon が JavaScript 向けのテストを `node` で動かすため。画面のビルドとブラウザでの実行には不要 |

依存の crate は `Cargo.lock` でバージョンを固定しています。画面の依存（rabbita 0.17.0）は `web/moon.mod` で固定しています。

> [!NOTE]
> Node.js が画面のテストに必要なことは、要件 NFR9.1（前提のツールは Rust と MoonBit だけ）と食い違います。どう扱うかは判断待ちです。

## リポジトリの構成

```text
Cargo.toml              Rust のワークスペース
crates/
  course_catalog/       業務: コースと日程、日程の状態、一覧の集計と並び（純粋関数）
  eligibility/          業務: 企業と従業員、利用資格（純粋関数）
  training_budget/      業務: 予算の取引と残高（純粋関数）
  purchasing/           業務: 購入の決定とチケット（純粋関数）
  in_memory_store/      アダプター: メモリ上のデータストアと初期データ
  web_api/              アダプター: Web API（実行ファイル。部品の組み立て）
  static_server/        画面用の静的ファイルサーバー（実行ファイル）
scripts/verify.sh       確認コマンド
web/                    MoonBit の画面のプロジェクト（rabbita を追加済み。画面は未実装。上の「現在の状況」を参照）
```

六角形アーキテクチャで、業務の 4 crate は HTTP・データ保存・画面に依存しません。依存の逆転はトレイトのオブジェクトではなく関数（クロージャ）の受け渡しで行い、現在時刻と乱数も関数として外から渡します。

## ビルドとテスト（リポジトリのルートで実行）

```bash
# ビルド
cargo build --workspace

# U1 の Rust のテスト（サーバーを起動せずに Web API もテストする）
cargo test -p course_catalog -p eligibility -p training_budget -p purchasing -p in_memory_store -p web_api -p static_server

# U1 の画面のテスト（画面の実装ができてから。Node.js が必要）
# この版の moon には `moon test --directory` がないため `-C` を使う
moon -C web test --target js

# 確認コマンド: fmt → clippy（警告はエラー）→ cargo test → moon test → 画面のビルド
bash scripts/verify.sh
```

応答時間のテスト（`crates/web_api/tests/performance.rs`）は、一覧・詳細・購入・操作中の従業員・チケット詳細にそれぞれ 100 回ずつ要求を送り、95 パーセンタイルが 200 ms 以内であることを確かめます。`cargo test -p web_api --test performance -- --nocapture` で測った値が表示されます。

## 起動（2 つのプロセス）

```bash
# 1. Web API（既定 http://localhost:3000）
cargo run -p web_api

# 2. 画面用の静的ファイルサーバー（既定 http://localhost:8080。web/dist/index.html が必要）
cargo run -p static_server
```

静的ファイルサーバーは、配信するディレクトリに `index.html` がないと理由を表示して終了します（画面は未実装のため、現時点では起動できません）。

### 環境変数

| 変数 | 対象 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `API_PORT` | Web API | `3000` | 待ち受けのポート |
| `UI_ORIGIN` | Web API | `http://localhost:8080` | 資格情報（Cookie）付きの通信と書き込みの要求を許可する画面のオリジン |
| `STATIC_PORT` | 静的ファイルサーバー | `8080` | 待ち受けのポート |
| `STATIC_DIR` | 静的ファイルサーバー | `web/dist` | 配信するディレクトリ |
| `RUST_LOG` | 両方 | `info` | ログの詳しさ（開発中は `debug`） |

秘密情報（パスワード・API キーなど）は使いません。

## Web API

| 操作 | 方法とパス | 備考 |
| --- | --- | --- |
| 動作確認 | `GET /api/health` | 動いていれば 200 |
| コース一覧 | `GET /api/courses` | 直近の開催日の早い順、同じ日はコース ID 順、予定なしは最後 |
| コースの詳細 | `GET /api/courses/{courseId}` | 形式不正は 400、存在しなければ 404 |
| 操作中の従業員 | `GET /api/me` | Cookie がなければ先頭の従業員（高橋 遥）の Cookie を発行 |
| 切り替え用の従業員一覧 | `GET /api/employees` | |
| 従業員の切り替え | `POST /api/session/employee` | 本文 `{"employeeId":"E002"}`。CSRF 対策のヘッダーが必要 |
| 購入 | `POST /api/purchases` | 本文 `{"sessionId":"S0001"}`（金額は読み取りません）。成功は 201、拒否は 409 と理由 |
| チケット 1 枚の詳細 | `GET /api/tickets/{ticketId}` | 本人のものでなければ 404 |

書き込みの要求（`POST`）は、`Origin` が画面のオリジンで、かつ `X-Requested-With: aidlc-handson-web` が付いているときだけ受け付けます。満たさなければ 403 です。拒否の理由は `employee-not-found`・`company-not-active`・`session-not-found`・`closed`・`already-purchased`・`full`・`insufficient-balance` のどれか 1 つです。

## デモの手順（画面ができるまでは curl で）

Web API を起動した別の端末で実行します。`cookies.txt` に Cookie を保存して、ブラウザと同じように従業員を引き継ぎます。

```bash
H=(-H 'Origin: http://localhost:8080' -H 'X-Requested-With: aidlc-handson-web' -H 'Content-Type: application/json')

# 1. 高橋 遥として開く（Cookie が発行される。残高 60,000 pt）
curl -s -c cookies.txt -b cookies.txt http://localhost:3000/api/me

# 2. コース一覧と、コース C001 の詳細（S0001 は残席 3・受講料 25,000 pt）
curl -s -b cookies.txt http://localhost:3000/api/courses
curl -s -b cookies.txt http://localhost:3000/api/courses/C001

# 3. S0001 を購入する（e チケットと購入後の残高 35,000 pt が返る）
curl -s -b cookies.txt "${H[@]}" -d '{"sessionId":"S0001"}' http://localhost:3000/api/purchases

# 4. 残高不足の従業員（鈴木 美咲・20,000 pt）に切り替えて購入する → 409 insufficient-balance
curl -s -c cookies.txt -b cookies.txt "${H[@]}" -d '{"employeeId":"E003"}' http://localhost:3000/api/session/employee
curl -s -b cookies.txt "${H[@]}" -d '{"sessionId":"S0001"}' http://localhost:3000/api/purchases

# 5. 契約が終了した企業の従業員（田中 一郎）に切り替えて購入する → 409 company-not-active
curl -s -c cookies.txt -b cookies.txt "${H[@]}" -d '{"employeeId":"E004"}' http://localhost:3000/api/session/employee
curl -s -b cookies.txt "${H[@]}" -d '{"sessionId":"S0001"}' http://localhost:3000/api/purchases
```

## 初期データ

Web API を起動するたびに、起動した日（日本時間）を基準に作り直します。再起動すると購入した内容は消えます。

- 企業: 株式会社サンプル商事・みらい技研株式会社（契約中）、旧契約産業株式会社（契約終了）
- 従業員: 高橋 遥（E001・60,000 pt。Cookie がないときの操作中の従業員）、佐藤 健太（E002・残高ちょうど 25,000 pt）、鈴木 美咲（E003・20,000 pt）、田中 一郎（E004・契約終了の企業）ほか
- コース: 4 カテゴリ・13 コース（オンラインと会場の両方）
- 日程: 満席（S0002・S0011）、受付終了の過去の日程（S0003 など）、残席 1（S0005）、残席 3（S0001）、今後の開催予定がないコース（C012・C013）を含む
