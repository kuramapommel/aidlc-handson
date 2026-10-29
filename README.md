# 研修コース e チケット購入サイト（ローカル検証専用）

契約企業の従業員が、研修予算（ポイント）で研修コースの受講用 e チケットを購入するサイトです。カートは持たず、1 回の購入で 1 つの日程を購入します。

> [!WARNING]
> **ローカル検証専用です。公開された環境に置いてはなりません。**
> ログイン機能はなく、Cookie に入った従業員 ID をそのまま認証済みの従業員として扱います（ADR-008）。Cookie の値を書き換えれば他人として操作できます。Web API と静的ファイルサーバーは 127.0.0.1 だけで待ち受けます。

## 現在の状況（U1 購入の最小の流れ）

| 部分 | 状況 |
| --- | --- |
| Rust の業務の部品・データストア・Web API・静的ファイルサーバー（`crates/`） | 実装済み。テストはすべて成功 |
| MoonBit の画面（`web/`） | 実装済み。コース一覧 → コースの詳細と日程の選択 → 購入確認 → 完了と e チケット、従業員の切り替え。更新関数のテストはすべて成功。説明は [web/README.md](web/README.md) |

画面の TEA の実行環境・描画・URL の扱いには `moonbit-community/rabbita` を、Web API との通信（Cookie 付きの fetch・専用ヘッダー・2xx 以外の本文の受け取り）には `mizchi/js_web` を使っています。rabbita 0.17.0 の通信の部品ではこれらができなかったためです（経緯は [web/README.md](web/README.md#通信に-mizchijs_web-を使う理由)）。

## 前提のツール

| ツール | バージョン | 導入 |
| --- | --- | --- |
| Rust（rustc・cargo・rustfmt・clippy） | 最新の stable（1.97.0 で確認） | <https://rustup.rs/> |
| MoonBit（moon） | 最新版（moon 0.1.20261028・moonc v0.10.18 で確認） | `curl -fsSL https://cli.moonbitlang.com/install/unix.sh \| bash`（`~/.moon/bin` に入ります） |
| Node.js | 22 系（v22.22.0 で確認） | **画面のテスト（`moon -C web test --target js`）の実行にだけ必要**。moon が JavaScript 向けのテストを `node` で動かすため（2026-10-29 に前提として承認）。画面のビルドとブラウザでの実行には不要 |
| ブラウザ | 最新版の Chrome・Edge・Firefox・Safari | 画面を開くため |

依存の crate は `Cargo.lock` で、画面の依存（rabbita 0.17.0・mizchi/js_web 0.13.1・mizchi/js_core 0.13.0）は `web/moon.mod` でバージョンを固定しています。

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
web/                    アダプター: MoonBit の画面（TEA。説明は web/README.md）
scripts/verify.sh       確認コマンド（fmt・clippy・テスト・画面のテストとビルド）
scripts/build-web.sh    画面のビルド（web/dist に配信するファイルをそろえる）
.githooks/              コミット前の確認（下の「コミット前の確認」）
.github/workflows/      GitHub Copilot のコーディングエージェント向けの環境の準備
AGENTS.md               AI のエージェント向けの案内（monorepo の構成と各部分の決まり）
```

六角形アーキテクチャで、業務の 4 crate は HTTP・データ保存・画面に依存しません。依存の逆転はトレイトのオブジェクトではなく関数（クロージャ）の受け渡しで行い、現在時刻と乱数も関数として外から渡します。画面は The Elm Architecture（TEA）で、更新関数と描画関数を純粋関数にし、通信と URL の書き換えはコマンドとして外に出します。

## ビルドとテスト（リポジトリのルートで実行）

```bash
# Rust のビルド
cargo build --workspace

# 画面のビルド（web/dist に main.js・index.html・style.css・config.js をそろえる）
bash scripts/build-web.sh

# U1 の Rust のテスト（サーバーを起動せずに Web API もテストする）
cargo test -p course_catalog -p eligibility -p training_budget -p purchasing -p in_memory_store -p web_api -p static_server

# U1 の画面のテスト（更新関数のテスト。Node.js が必要）
# この版の moon には `moon test --directory` がないため `-C` を使う
moon -C web test --target js

# 確認コマンド: fmt → clippy（警告はエラー）→ cargo test → moon test → 画面のビルド
bash scripts/verify.sh
```

応答時間のテスト（`crates/web_api/tests/performance.rs`）は、一覧・詳細・購入・操作中の従業員・チケット詳細にそれぞれ 100 回ずつ要求を送り、95 パーセンタイルが 200 ms 以内であることを確かめます。`cargo test -p web_api --test performance -- --nocapture` で測った値が表示されます。

## 起動（2 つのプロセス）

```bash
# 0. 画面をビルドしておく（初回と、web/ を変えたとき）
bash scripts/build-web.sh

# 1. Web API（既定 http://localhost:3000）
cargo run -p web_api

# 2. 画面用の静的ファイルサーバー（既定 http://localhost:8080。別の端末で）
cargo run -p static_server
```

ブラウザで <http://localhost:8080/> を開きます。静的ファイルサーバーは、配信するディレクトリに `index.html` がないと理由を表示して終了します（先に `bash scripts/build-web.sh` を実行してください）。画面内の URL（`/courses/C001` など）を直接開いても、`index.html` が返って画面が復元されます。

### 環境変数

| 変数 | 対象 | 既定値 | 意味 |
| --- | --- | --- | --- |
| `API_PORT` | Web API | `3000` | 待ち受けのポート |
| `UI_ORIGIN` | Web API | `http://localhost:8080` | 資格情報（Cookie）付きの通信と書き込みの要求を許可する画面のオリジン |
| `STATIC_PORT` | 静的ファイルサーバー | `8080` | 待ち受けのポート |
| `STATIC_DIR` | 静的ファイルサーバー | `web/dist` | 配信するディレクトリ |
| `RUST_LOG` | Web API・静的ファイルサーバー | `info` | ログの詳しさ（開発中は `debug`） |
| `API_BASE` | 画面のビルド（`scripts/build-web.sh`） | `http://localhost:3000` | 画面が呼ぶ Web API のオリジン（`web/dist/config.js` に書き込む） |
| `WEB_DIST` | 画面のビルド（`scripts/build-web.sh`） | `web/dist` | 画面のファイルをそろえるディレクトリ |

ポートを変えるときは、`UI_ORIGIN`（Web API が許可する画面のオリジン）と `API_BASE`（画面が呼ぶ Web API）もあわせて変えます。秘密情報（パスワード・API キーなど）は使いません。

## デモの手順（画面で）

Web API と静的ファイルサーバーを起動し、<http://localhost:8080/> を開きます。

1. **高橋 遥で購入する**: ヘッダーに「操作中の従業員: 高橋 遥（株式会社サンプル商事）」「残高: 60,000 pt」が出ます。「チームマネジメント入門」のカードを開き、「残席 3 席・状態: 購入可」の日程（S0001。日時は起動した日から 1 週間後）を選んで「購入確認へ進む」。購入確認で購入後の残高 35,000 pt を確かめて「購入を確定する」。完了画面に e チケット（チケット番号は `XXXX-XXXX`）が出て、ヘッダーの残高が 35,000 pt になります。同じコースの詳細を開き直すと、その日程は「状態: 購入済み」で選べません。
2. **選べない日程**: 同じコースの「満席」「受付終了」の日程は、理由が文字で表示され、選べません（キーボードの矢印キーでも選択されません）。
3. **残高不足**: ヘッダーの「従業員を切り替える」で鈴木 美咲（20,000 pt）に切り替えると、コース一覧へ戻ります。受講料 25,000 pt のコースの購入確認では「不足額 5,000 pt」が出て、確定ボタンは押せません。
4. **契約外の企業**: 田中 一郎（旧契約産業株式会社・契約終了）に切り替えると、一覧・詳細・購入確認の上部に「ご所属の企業は現在このサービスをご利用いただけません」と出て、確定ボタンは押せません。閲覧は続けられます。
5. **存在しないコース**: <http://localhost:8080/courses/C999> を開くと「コースが見つかりません」と「コース一覧へ戻る」が出ます。

主要な流れ（一覧 → 詳細と日程の選択 → 購入確認 → 完了）は、全画面を読み込み直さずに画面内で移動します。Tab・Shift+Tab・Enter・Space・矢印キーだけでも操作できます。

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

### curl での確かめ方

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

## コミット前の確認

`.githooks/pre-commit` は、コミットの前に `cargo fmt --all --check` と `moon -C web check` を行います。Git はフックの置き場所の設定をリポジトリに含めないため、各自の手元で 1 回だけ、リポジトリのルートで次を実行して有効にしてください。

```bash
git config core.hooksPath .githooks
```

clippy・テスト・画面のビルドまで含めたすべての確認は `bash scripts/verify.sh` で行います。詳しくは [.githooks/README.md](.githooks/README.md) を参照してください。
