# 画面（MoonBit）: 研修コース e チケット

研修コース e チケット購入サイトの Web 画面です。リポジトリ全体の説明・起動の手順・デモの手順は、ルートの [README.md](../README.md) を参照してください。

> [!WARNING]
> ローカル検証専用です（ルートの README の注意を参照）。

## 作り

画面は The Elm Architecture（TEA）で組み立てています。画面全体の状態を 1 つの不変な値（`Model`）で持ち、

- 更新関数 `update : (Model, Msg) -> (Model, Array[Effect])` と
- 描画関数 `view : (Model, Emit[Msg]) -> Html`

をどちらも純粋関数にしています。Web API の呼び出しと URL の書き換えは、更新関数が `Effect` の値として返すだけで、`effect_to_cmd` が rabbita のコマンドに変えて実行します。TEA の実行環境は自作せず、rabbita の `@rabbita.elmish` を使います。

| ファイル | 中身 |
| --- | --- |
| `model.mbt` | 状態（`Model`・`Route`・`Remote` など）・メッセージ（`Msg`）・作用（`Effect`）の型 |
| `update.mbt` | 更新関数。購入確認で確定できるか（`confirm_readiness`）の計算、古い応答を捨てる照合 |
| `search.mbt` | コース一覧の探索（U2）の純粋関数: 探索条件の入力の形（`SearchInput`）、URL のクエリとの相互の変換（既定の値は書かない・項目ごとに検証して復元）、期間の前後の確認、探索の応答（400 を含む）の読み取り |
| `update_search.mbt` | 更新関数のうち探索の部分（キーワードの 150 ms の待ち、要求の番号による古い応答の破棄、リセット・再試行・URL からの復元） |
| `view_search.mbt` | 探索条件の入力・0 件と「条件をリセット」・400 の案内の描画 |
| `route.mbt` | URL のパスと画面（`Route`）の相互の変換 |
| `view.mbt`・`view_header.mbt`・`view_pages.mbt` | 描画関数（画面全体・ヘッダーと従業員の切り替え・各画面） |
| `format.mbt` | 表示の形（日本時間の日時、3 桁区切りのポイント、表示名） |
| `decode.mbt` | Web API の応答（状態コードと本文）を画面の型にする純粋関数 |
| `http.mbt` | Web API との通信（`mizchi/js_web` の fetch）を rabbita のコマンドにする |
| `web.mbt` | 組み立て（作用 → コマンド、URL の購読、`app`） |
| `cmd/main/` | ブラウザで画面を起動する薄い入り口（`config.js` から Web API のオリジンを読む） |
| `static/` | `index.html` と `style.css`（`scripts/build-web.sh` が `dist/` にそろえる） |
| `web_test.mbt` | 公開の振る舞い（更新関数）のテスト |
| `search_test.mbt` | 探索（U2）の更新関数と URL の変換のテスト（`moon -C web test --target js search_test.mbt` で U2 だけを実行できる） |
| `web_wbtest.mbt` | 内部の補助関数（表示の形・応答の読み取り・ヘッダー）のテスト |

画面と URL:

| 画面 | URL |
| --- | --- |
| コース一覧 | `/`（探索条件はクエリ。例: `/?q=AI&category=technology&available=true&from=2026-11-10&to=2026-11-15&sort=fee-asc`） |
| コースの詳細と日程の選択 | `/courses/{courseId}` |
| 購入確認 | `/courses/{courseId}/sessions/{sessionId}/confirm` |
| 完了と e チケット | `/tickets/{ticketId}` |

操作中の従業員 ID は URL に含めず、Web API が発行する Cookie で受け渡します。

コース一覧の探索（U2）: キーワードは入力が止まってから 150 ms 待って（rabbita の `@cmd.delay`）、ほかの条件は変えたらすぐに Web API へ送ります。絞り込みと並べ替えは Web API が行い、画面は送る前の形の確認（期間の前後）と URL の扱いだけを持ちます。条件を変えるたびに履歴を増やさずに URL を書き換え（`@nav.replace_url`）、URL を開いたときは項目ごとに確かめて不正な項目だけを無視します。条件なしの一覧は U1 と同じ要求（`FetchCourses`）で取得し、条件付きの一覧は要求の番号付きの `SearchCourses` で取得します。

## ビルドとテスト（リポジトリのルートで実行）

```bash
# 型の確認
moon -C web check --target js

# 更新関数と補助関数のテスト（Node.js が必要）
moon -C web test --target js

# ビルドして web/dist にそろえる（main.js・index.html・style.css・config.js）
bash scripts/build-web.sh

# 整形とインターフェースの更新（変更のあとに）
moon -C web fmt
moon -C web info --target js
```

`moon test --directory web` はこの版の moon にないため、`-C web` を使います。

## 使っているライブラリ

| ライブラリ | 版 | 使い方 |
| --- | --- | --- |
| [`moonbit-community/rabbita`](https://mooncakes.io/docs/moonbit-community/rabbita) | 0.17.0 | TEA の実行環境（`@rabbita.elmish`）・仮想 DOM での描画（`@html`）・URL の購読と書き換え（`@sub.on_url_changed`・`@sub.on_url_request`・`@nav.push_url`）・非同期の処理のコマンド化（`@rabbita.attempt`） |
| [`mizchi/js_web`](https://mooncakes.io/docs/mizchi/js_web) | 0.13.1 | 通信だけ（`@fetch.fetch`。`credentials: "include"` とヘッダーを指定し、2xx 以外の本文も読む） |
| [`mizchi/js_core`](https://mooncakes.io/docs/mizchi/js_core) | 0.13.0 | 要求の本文を fetch に渡すための値の変換（`@js.any`）。`mizchi/js_web` の公開の関数がこの型を使う |

### 通信に mizchi/js_web を使う理由

rabbita 0.17.0 の通信の部品（`@http`。`moonbitlang/async` の fetch を使う）は、次の 3 つに対応していませんでした（2026-10-29 にヘッドレスの Chromium で確認）。

1. 別オリジンの Web API への Cookie の送信（fetch の `credentials` を指定できない）
2. CSRF 対策の専用ヘッダー（`X-Requested-With: aidlc-handson-web`）の付与
3. 2xx 以外の応答（409 の拒否の理由など）の本文の受け取り

そのため、TEA の実行環境・描画・URL の扱いは rabbita のまま使い、通信のコマンドだけを `mizchi/js_web` の fetch で作って、rabbita の公開の拡張口（`@rabbita.attempt`）からコマンドとして使っています。すべての要求は `credentials: "include"` で送り、書き込み（POST）には専用ヘッダーを付けます。応答は状態コードと本文の文字列のまま受け取り、`decode.mbt` の純粋関数で読み取ります。

## 画面の決まり

- 状態は色だけでなく文字で示します（「状態: 満席」など）。ボタンと日程の選択の部品には名前（ラベル）を付け、キーボードだけで操作できるようにします。
- 操作できる部品には `data-testid` を付けます（例: `course-card-link-C001`、`session-option-radio-S0001`、`purchase-confirm-button`）。
- Web API から受け取った文字列は、HTML として解釈させず文字として描画します（rabbita の `@html.text` などのテキストノード）。
