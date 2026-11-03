# テストの結果

実行日時: 2026-11-03（UTC）。場所: リポジトリのルート。

## ビルド

| コマンド | 結果 |
| --- | --- |
| `cargo build --workspace` | 成功 |
| `bash scripts/build-web.sh` | 成功（0 errors、古い `derive(...)` の書き方の警告 31 件） |

## 単体テスト（各ユニットのコマンドを 1 回ずつ）

| コマンド | 結果 |
| --- | --- |
| `bash scripts/verify.sh`（fmt・clippy・`cargo test --workspace`・`moon -C web test --target js`・画面のビルド） | 成功 |
| `cargo test --workspace` | 88 件成功、失敗 0、スキップ 0 |
| `moon -C web test --target js` | 57 件成功、失敗 0 |
| U3: `cargo test -p purchasing owned_tickets_tests` | 7 件成功 |
| U3: `cargo test -p web_api --test my_tickets` | 7 件成功 |
| U3: `moon -C web test --target js my_tickets_test.mbt` | 11 件成功 |

U1・U2 のユニットのコマンドは `cargo test --workspace`・`moon -C web test --target js` に含まれる（重複して数えない）。カバレッジの計測は行っていない（下限なし）。

## 性能（95 パーセンタイル。目標 200 ms）

| 受け口 | 測った値 |
| --- | --- |
| GET /api/courses | 0.463 ms |
| GET /api/courses（探索） | 0.404 ms |
| 数万文字のキーワード | 10.763 ms |
| GET /api/courses/{courseId} | 0.197 ms |
| POST /api/purchases | 0.190 ms |
| GET /api/me | 0.096 ms |
| GET /api/tickets/{ticketId} | 0.108 ms |
| GET /api/tickets（U3） | 0.275 ms |
| 画面: キーワードの反映（ブラウザ） | 204 ms（目標 300 ms） |

## ブラウザでの確認（Playwright・Chromium、2 プロセスを起動）

成功: 一覧とヘッダー、キーワード・0 件・リセット・URL の復元と不正な項目の除去、スクリプトを含むキーワードが実行されない、日程の選択・購入確認・確定・完了の e チケット、ヘッダーの残高の更新、完了画面から購入済みチケット（いま購入したチケットを含む）、開閉（`aria-expanded`・`aria-controls`）、ヘッダーのリンクをキーボードで、Tab で主要な操作に届く、名前のないフォーム部品なし、存在しないコースの案内、キーボードだけでの購入（遷移 3 回）、選べない日程が矢印キーで選択されない、日程の状態の文字、初期データで 3 枚の従業員（伊藤 さくら）の並び、チケットのない従業員（鈴木 美咲）の案内、詳細から戻ったときの条件の保持、全画面を読み込み直さない、ライブリージョン、不正な探索条件の 400。

失敗: **Web API を止めた状態では、一覧も購入済みチケットも「読み込んでいます」のまま**で、失敗の表示と「再試行」が出ない（画面で `TypeError: Cannot read properties of undefined (reading '_1')`）。

途中で、確認のスクリプト側の誤り 2 件（開閉の直後に再描画を待たずに読んだ、初期データのチケットを持つ従業員を取り違えた）を直して確認し直した。どちらも画面の不具合ではない。

## ログ（Web API）

要求ごとの 1 行（method・path・status・elapsed_ms）、起動時の待ち受けのアドレス、購入の拒否の理由（reason=full・closed）、探索の 400 の項目名（fields=category,periodFrom）、詳細レベルの探索の件数（`RUST_LOG=debug`）を確認した。チケット番号・従業員名・入力したキーワードの値はログに出ていない。

## 不具合の詳細

Web API に接続できないとき（Web API が止まっている、ネットワークが切れている）、画面が JavaScript の例外 `TypeError: Cannot read properties of undefined (reading '_1')` で止まり、「…を読み込んでいます」のまま、失敗の表示と「再試行」が出ない。

- 再現: Web API を止めて <http://localhost:8080/> または `/my-tickets` を開く。
- 場所: `web/http.mbt` の `request_cmd`。通信の失敗（`Err(error)`）を `"\{error}"` で文字列にするところで、接続の拒否で投げられた JavaScript の例外を MoonBit の `Error` として表示しようとして落ちる（スタック: `Error::to_string` ← `request_cmd` の `@rabbita.attempt`）。
- 影響: コース一覧（U1・U2）、コースの詳細・購入確認（U1）、ヘッダーの従業員の情報（U1）、購入済みチケット（U3）など、通信するすべての画面。Web API が 5xx を返す場合（接続はできる場合）は影響しない。
- 発生した時点: U3 より前の版（U1・U2 のコード）でも同じく起きる。U3 で入った不具合ではない。今回入れた moon（0.1.20261102）でビルドした結果で確認しており、U1・U2 を作った時点の moon では起きていなかった可能性もある（未確認）。
- 自動テストで見つからなかった理由: 画面のテストは更新関数に失敗のメッセージを直接与えており、`request_cmd` の失敗の文字列化を通らない。

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| U1-NFR10.1 | `u1-purchase-skeleton/nfr-requirements/observability-requirements.md` | Web API は、要求ごとに 1 行のログを出す。項目はメソッド・パス・状態コード・処理時間（ミリ秒）とする | 要求ごとに 1 行（method・path・status・elapsed_ms） | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR10.2 | `u1-purchase-skeleton/nfr-requirements/observability-requirements.md` | 購入の拒否は、従業員 ID・日程 ID・拒否の理由（BR5.1 の 7 つのどれか）をログに出す（BR5.8）。承認した購入は、チケット ID・従業員 ID・日程 ID をログに出… | 拒否で employee_id・session_id・reason=full/closed を出力 | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR10.3 | `u1-purchase-skeleton/nfr-requirements/observability-requirements.md` | ログに個人の連絡先は出さない（今回は扱わない）。従業員の表示名もログに出さず、従業員 ID だけを出す | ログに従業員名・チケット番号が出ない（0 件） | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR10.4 | `u1-purchase-skeleton/nfr-requirements/observability-requirements.md` | ログは人が読める 1 行の形式で出す。ログの詳しさは環境変数で変えられる（既定は情報レベル。開発中は詳細レベルにできる） | RUST_LOG=debug で詳細レベル、既定は情報レベル | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR10.5 | `u1-purchase-skeleton/nfr-requirements/observability-requirements.md` | 予期しない失敗は、エラーの内容をログに出す（応答には出さない。security-requirements.md NFR3.10） | 予期しない失敗の error ログ（app.rs・handlers.rs）と、応答は一般的なメッセージ（自動テスト） | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR10.6 | `u1-purchase-skeleton/nfr-requirements/observability-requirements.md` | 起動時に、Web API と画面用の静的ファイルサーバーが待ち受けるアドレス（例: `http://localhost:3000`、`http://localhost:8080`）… | 起動時に http://localhost:3000・http://localhost:8080 を出力 | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR10.7 | `u1-purchase-skeleton/nfr-requirements/observability-requirements.md` | Web API は、動作確認用に `GET /api/health` を持ち、動いていれば 200 を返す。確認コマンドや手動の確認で使う | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR1.1 | `u1-purchase-skeleton/nfr-requirements/performance-requirements.md` | コース一覧の受け口（`GET /api/courses`） 95% の要求が 200 ms 以内に応答する | 一覧 p95 0.463 ms・詳細 0.197 ms | cargo test -p web_api --test performance/search/my_tickets -- --nocapture | build-and-test | Met |
| U1-NFR1.2 | `u1-purchase-skeleton/nfr-requirements/performance-requirements.md` | コースの詳細の受け口（`GET /api/courses/{courseId}`） 95% の要求が 200 ms 以内に応答する | 操作中の従業員 0.096 ms・チケット詳細 0.108 ms | cargo test -p web_api --test performance/search/my_tickets -- --nocapture | build-and-test | Met |
| U1-NFR1.3 | `u1-purchase-skeleton/nfr-requirements/performance-requirements.md` | 購入の受け口（`POST /api/purchases`）と、操作中の従業員・チケット詳細の受け口 95% の要求が 200 ms 以内に応答する（要件定義書の NFR1 は一覧・… | 購入 p95 0.190 ms | cargo test -p web_api --test performance/search/my_tickets -- --nocapture | build-and-test | Met |
| U1-NFR1.4 | `u1-purchase-skeleton/nfr-requirements/performance-requirements.md` | 画面の移動（一覧 → 詳細 → 購入確認 → 完了） 全画面を読み込み直さず、画面内の移動で表示する。Web API の応答を受けてから 300 ms 以内に表示へ反映する | 一覧→詳細→確認→完了を操作して表示を確認（確定から完了表示まで 109 ms） | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR1.5 | `u1-purchase-skeleton/nfr-requirements/performance-requirements.md` | 処理の組み立て 一覧の集計・日程の状態の判定は、1 回の要求の中でチケットの数を日程ごとに 1 回だけ数える（同じ数え直しを繰り返さない） | コードレビュー | コードの検索とレビュー／bash scripts/verify.sh | build-and-test | Met |
| U1-NFR4.1 | `u1-purchase-skeleton/nfr-requirements/performance-requirements.md` | 「コース一覧 → 日程の選択（コースの詳細）→ 購入確認 → 完了」を、ページの遷移 3 回で完了できる | キーボードだけの購入でページの遷移 3 回 | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR4.2 | `u1-purchase-skeleton/nfr-requirements/performance-requirements.md` | 読み込み中・読み込み失敗の表示を区別する（U1 の一覧は 0 件にならないが、文言を分けておく。BR7.1） | Web API に接続できないとき、画面が JavaScript の例外（Error の文字列化）で止まり「読み込んでいます」のまま。失敗の表示と「再試行」が出ない | e2e: Web API を止めて画面を開く（pageerror TypeError reading _1） | build-and-test | Not Met |
| U1-NFR4.3 | `u1-purchase-skeleton/nfr-requirements/performance-requirements.md` | 確定の処理中であることが表示で分かり、確定ボタンは無効になる（BR7.3） | 画面のテストで確認（ブラウザでは処理が速く表示を観測できず） | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR4.4 | `u1-purchase-skeleton/nfr-requirements/performance-requirements.md` | 購入できなかったときに、理由と次の操作（別の日程を選ぶ・一覧へ戻る・もう一度試す）が必ず表示される（BR7.4、BR7.6） | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR5.1 | `u1-purchase-skeleton/nfr-requirements/performance-requirements.md` | 主要な流れ（一覧 → 詳細と日程の選択 → 購入確認 → 確定 → 完了）を、キーボードだけで操作できる | キーボードだけで一覧→詳細→日程→確認→確定 | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR5.2 | `u1-purchase-skeleton/nfr-requirements/performance-requirements.md` | ボタン・日程の選択の部品・従業員の切り替えの部品に、名前（ラベル）がある | 名前のないフォーム部品 0 件 | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR5.3 | `u1-purchase-skeleton/nfr-requirements/performance-requirements.md` | 日程の状態（受付終了・満席・購入済み）と契約外の案内は、色だけでなく文字で示し、読み上げ用のテキストにも含める（BR2.5、BR1.3） | 日程の状態を文字（満席・受付終了・購入可）で表示 | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR5.4 | `u1-purchase-skeleton/nfr-requirements/performance-requirements.md` | 選べない日程は、キーボードで移動しても選択の状態にならない（AC2.3.4） | 選べない日程 2 件が矢印キーで選択されない | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR2.1 | `u1-purchase-skeleton/nfr-requirements/reliability-requirements.md` | 予算の残高と残席数は負の値にならない。購入は「チケット 1 枚と消費の取引 1 件をどちらも反映する」か「何も反映しない」かのどちらかで、一部だけを反映した状態を残さない（BR5.… | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR2.2 | `u1-purchase-skeleton/nfr-requirements/reliability-requirements.md` | 残席 1 の日程に同時に 2 件の購入が届いても、チケットは 1 枚だけ発行され、もう 1 件は満席として拒否される（BR5.4、AC3.3.4） | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR2.4 | `u1-purchase-skeleton/nfr-requirements/reliability-requirements.md` | 同じ従業員の同じ日程への購入が重ねて届いても、チケットは 1 枚だけ発行される（BR5.5、AC3.4.1、AC3.4.2） | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR2.5 | `u1-purchase-skeleton/nfr-requirements/reliability-requirements.md` | Web API の処理中に予期しない失敗が起きても、データストアの状態は変わらず、プロセスは次の要求を受け付け続ける（1 件の失敗でサーバー全体を止めない） | 自動テストとコードレビュー | コードの検索とレビュー／bash scripts/verify.sh | build-and-test | Met |
| U1-NFR2.6 | `u1-purchase-skeleton/nfr-requirements/reliability-requirements.md` | 画面は、Web API と通信できないときも操作不能にならない。一覧・詳細では失敗の表示と「再試行」、購入では「もう一度試す」と確かめ方の案内を出す（BR7.1、BR7.6） | Web API に接続できないとき、画面が JavaScript の例外（Error の文字列化）で止まり「読み込んでいます」のまま。失敗の表示と「再試行」が出ない | e2e: Web API を止めて画面を開く（pageerror TypeError reading _1） | build-and-test | Not Met |
| U1-NFR6.1 | `u1-purchase-skeleton/nfr-requirements/reliability-requirements.md` | 業務の部品は純粋関数で書く。現在時刻と乱数は引数（関数）として受け取り、業務の部品の中で時計や乱数を直接呼ばない | 業務の crate に現在時刻・乱数の直接の呼び出しなし（検索） | コードの検索とレビュー／bash scripts/verify.sh | build-and-test | Met |
| U1-NFR6.2 | `u1-purchase-skeleton/nfr-requirements/reliability-requirements.md` | データは不変として扱う。状態の更新は「現在の状態 → 新しい状態」を返す関数で表し、可変な内部状態を持つオブジェクトやクラス階層による設計をしない。可変な共有状態は、データストアの… | コードレビュー（状態の更新は新しい値を返す形） | コードの検索とレビュー／bash scripts/verify.sh | build-and-test | Met |
| U1-NFR6.3 | `u1-purchase-skeleton/nfr-requirements/reliability-requirements.md` | 部品の組み立ては、関数の部分適用（クロージャ）で行う。依存の逆転にインターフェース（トレイトのオブジェクト）を使わず、関数を受け渡す（project.md の決定） | 業務の crate に dyn（トレイトのオブジェクト）なし | コードの検索とレビュー／bash scripts/verify.sh | build-and-test | Met |
| U1-NFR6.4 | `u1-purchase-skeleton/nfr-requirements/reliability-requirements.md` | 画面は TEA（ADR-007）で組み、更新関数と描画関数は純粋関数にする。Web API の呼び出し・URL の書き換えはコマンドとして外に出す | TEA の更新・描画関数を画面のテストで直接呼べる | コードの検索とレビュー／bash scripts/verify.sh | build-and-test | Met |
| U1-NFR8.1 | `u1-purchase-skeleton/nfr-requirements/reliability-requirements.md` | Rust のテストは、リポジトリのルートからの `cargo test` 1 回で、手作業の準備なしにすべて実行できる（Web API のテストもサーバーを別に起動せずに動く） | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR8.2 | `u1-purchase-skeleton/nfr-requirements/reliability-requirements.md` | MoonBit のテストは、リポジトリのルートからの `moon -C web test --target js` で、手作業の準備なしに実行できる（Node.js が必要。NFR… | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR8.3 | `u1-purchase-skeleton/nfr-requirements/reliability-requirements.md` | 業務の部品のテストは、成功の場合に加えて、少なくとも 2 つの失敗・境目の場合を持つ（例: 購入の決定なら、残高ちょうど・残高不足・満席・受付終了・購入済み・契約外・存在しない従業… | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR8.4 | `u1-purchase-skeleton/nfr-requirements/reliability-requirements.md` | 時刻に依存するテスト（受付終了、初期データの日程）は、固定した時刻を渡して結果が毎回同じになるようにする | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR9.1 | `u1-purchase-skeleton/nfr-requirements/reliability-requirements.md` | ビルドと起動の前提のツールは Rust（最新 stable）と MoonBit（最新版）だけとする。画面用の静的ファイルサーバーは Rust のワークスペースの実行ファイルで用意す… | 前提は Rust と MoonBit（moon 0.1.20261102 を公式の手順で導入）のみ | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR9.2 | `u1-purchase-skeleton/nfr-requirements/reliability-requirements.md` | リポジトリのルートからのコマンドで、ビルド・テスト・2 プロセスの起動ができる。手順は README に書く | README の手順で 2 プロセスを起動し画面を操作 | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR9.3 | `u1-purchase-skeleton/nfr-requirements/reliability-requirements.md` | Construction の確認コマンド（`bash scripts/verify.sh`）は、Rust のテスト・MoonBit のテスト・画面のビルドを順に実行し、どれかが失敗… | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR7.1 | `u1-purchase-skeleton/nfr-requirements/scalability-requirements.md` | 業務の部品（CourseCatalog・Eligibility・TrainingBudget・Purchasing）は、データの読み書きを「関数の形」として定め、データストアの実装… | 業務の 4 crate の依存は chrono・unicode-normalization と業務の crate だけ | コードの検索とレビュー／bash scripts/verify.sh | build-and-test | Met |
| U1-NFR7.2 | `u1-purchase-skeleton/nfr-requirements/scalability-requirements.md` | データストアの実装を差し替えられることを、テストで示す。業務の部品のテストでは、メモリ上のデータストアではなく、テスト用の関数（固定のデータを返すクロージャ）を渡して動かす | 固定の値を返す関数で業務の部品をテスト | コードの検索とレビュー／bash scripts/verify.sh | build-and-test | Met |
| U1-NFR7.3 | `u1-purchase-skeleton/nfr-requirements/scalability-requirements.md` | 将来 PostgreSQL に移すときに必要な保証（購入の検証と記録を 1 つのトランザクションで行い、日程の行のロックまたは「従業員と日程の組」の一意制約で重複と定員超過を防ぐ）… | PostgreSQL への移行の注意をコメントに記載（store.rs・search.rs・owned_tickets.rs） | コードの検索とレビュー／bash scripts/verify.sh | build-and-test | Met |
| U1-NFR2.3 | `u1-purchase-skeleton/nfr-requirements/scalability-requirements.md` | 同時に届いた購入の要求は、データストアの更新を 1 か所で 1 件ずつ順に適用して処理する（BR5.4）。書き込みの並行性は低くなるが、想定する規模では問題にならない | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR7.4 | `u1-purchase-skeleton/nfr-requirements/scalability-requirements.md` | コース一覧の受け口と画面の状態は、U2 が探索条件（キーワード・カテゴリ・空席ありのみ・期間・並べ替え）を後から追加できる形にする（ユニット定義の注意） | U2・U3 で既存の項目を変えずに足した（U1・U2 のテストは成功のまま） | コードの検索とレビュー／bash scripts/verify.sh | build-and-test | Met |
| U1-NFR3.1 | `u1-purchase-skeleton/nfr-requirements/security-requirements.md` | Web API は、すべての入力を検証する。ID の形式（コース ID は C と 3 桁の数字、日程 ID は S と 4 桁の数字、従業員 ID は E と 3 桁の数字、チケ… | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR3.2 | `u1-purchase-skeleton/nfr-requirements/security-requirements.md` | 購入の可否は Web API が最終的に判断し、画面の表示（ボタンの無効化など）に頼らない。最終チェックの順と理由は BR5.1 のとおり BR1.1、BR5.1 | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR3.3 | `u1-purchase-skeleton/nfr-requirements/security-requirements.md` | 受講料はコースのデータから決め、要求の本文に金額があっても読み取らない BR5.2 | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR3.4 | `u1-purchase-skeleton/nfr-requirements/security-requirements.md` | 操作中の従業員 ID の Cookie は、画面のスクリプトから読めない設定（HttpOnly）、同じサイトの中だけで送る設定（SameSite=Lax）、パスは `/` とする。… | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR3.5 | `u1-purchase-skeleton/nfr-requirements/security-requirements.md` | 書き込みの要求（購入・従業員の切り替え）は、送信元（Origin ヘッダー）が許可した画面のオリジンであり、かつ画面だけが付ける専用のヘッダー（例: `X-Requested-Wi… | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR3.6 | `u1-purchase-skeleton/nfr-requirements/security-requirements.md` | オリジンをまたぐ通信（CORS）は、画面のオリジン（既定は `http://localhost:8080`。設定で変えられる）だけに、資格情報（Cookie）付きで許可する。ワイル… | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR3.7 | `u1-purchase-skeleton/nfr-requirements/security-requirements.md` | チケット 1 枚の詳細は、そのチケットを購入した従業員にだけ返す。ほかの従業員のチケットには、存在しない場合と同じ 404 を返す（存在するかどうかも漏らさない） BR5.10 | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U1-NFR3.8 | `u1-purchase-skeleton/nfr-requirements/security-requirements.md` | Cookie の従業員 ID を認証済みとして扱うのはローカル検証専用であることを、README と、Cookie を読み取るコードのコメントに明記する ADR-008 | README と session_cookie.rs にローカル検証専用と明記 | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR3.9 | `u1-purchase-skeleton/nfr-requirements/security-requirements.md` | 秘密情報（パスワード・API キーなど）をコードに埋め込まない。今回は秘密情報を使わない。設定値（ポート、画面のオリジン）は環境変数で変えられるようにし、既定値をコードに持つ — | 秘密情報の埋め込みの検索で 0 件 | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR3.10 | `u1-purchase-skeleton/nfr-requirements/security-requirements.md` | 予期しない失敗では 5xx と一般的なメッセージだけを返し、内部の情報（スタックトレース・内部の状態）を応答に含めない。詳細はログにだけ出す — | 自動テスト（一般的な 500） | コードの検索とレビュー／bash scripts/verify.sh | build-and-test | Met |
| U1-NFR3.11 | `u1-purchase-skeleton/nfr-requirements/security-requirements.md` | 画面は、Web API から受け取った文字列（コース名・説明など）をそのまま HTML として解釈させず、文字として表示する（スクリプトの埋め込みを防ぐ） — | 生の HTML を入れる箇所の検索で 0 件。スクリプトを含むキーワードが実行されない | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U1-NFR3.12 | `u1-purchase-skeleton/nfr-requirements/security-requirements.md` | 依存する crate とパッケージは、最新の stable 版を使い、取得したバージョンを固定する（Rust は `Cargo.lock`、MoonBit はパッケージの固定の仕組… | Cargo.lock と web/moon.mod でバージョンを固定 | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U2-NFR10.1 | `u2-course-search/nfr-requirements/observability-requirements.md` | 探索の要求も、U1 と同じく要求ごとに 1 行のログ（メソッド・パス・状態コード・処理時間）を出す。パスにはクエリを含めない（キーワードをログに残さない） | 探索の要求も 1 行。パスにクエリを含めない | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U2-NFR10.2 | `u2-course-search/nfr-requirements/observability-requirements.md` | 探索の 400 は、不正な条件の項目名（category・period など）だけをログに出す。値そのもの（キーワードや日付の文字列）は出さない | 400 は fields=category,periodFrom だけ。値（bogus など）はログに 0 件 | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U2-NFR10.3 | `u2-course-search/nfr-requirements/observability-requirements.md` | 詳細レベルのログでは、探索の結果の件数と処理時間を出せる（既定の情報レベルでは出さない） | 詳細レベルで count と elapsed_us を出力 | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U2-NFR1.1 | `u2-course-search/nfr-requirements/performance-requirements.md` | 探索条件付きのコース一覧（`GET /api/courses`。キーワード・カテゴリ・空席ありのみ・期間・並べ替えの組み合わせ） 95% の要求が 200 ms 以内に応答する | 探索 p95 0.404 ms | cargo test -p web_api --test performance/search/my_tickets -- --nocapture | build-and-test | Met |
| U2-NFR1.2 | `u2-course-search/nfr-requirements/performance-requirements.md` | 条件を変えてから一覧に反映するまで（画面） キーワードは入力が止まってから、ほかの条件は変えてから 300 ms 以内に表示へ反映する（キーワードの 150 ms の待ちを含む。B… | キーワードの反映 204 ms（ブラウザで計測） | e2e（Playwright） | build-and-test | Met |
| U2-NFR1.3 | `u2-course-search/nfr-requirements/performance-requirements.md` | 画面の再描画 条件を変えても全画面を読み込み直さず、探索条件の入力欄の値・入力中の文字の位置を保つ | 条件の変更で全画面を読み込み直さず入力の値が保たれる | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U2-NFR4.1 | `u2-course-search/nfr-requirements/performance-requirements.md` | 読み込み中・0 件・読み込み失敗の 3 つを、文言の違う別の表示にする（BR11.6、BR11.7） | Web API に接続できないとき、画面が JavaScript の例外（Error の文字列化）で止まり「読み込んでいます」のまま。失敗の表示と「再試行」が出ない | e2e: Web API を止めて画面を開く（pageerror TypeError reading _1） | build-and-test | Not Met |
| U2-NFR4.2 | `u2-course-search/nfr-requirements/performance-requirements.md` | 0 件では「条件をリセット」、失敗では「再試行」を必ず出す | Web API に接続できないとき、画面が JavaScript の例外（Error の文字列化）で止まり「読み込んでいます」のまま。失敗の表示と「再試行」が出ない | e2e: Web API を止めて画面を開く（pageerror TypeError reading _1） | build-and-test | Not Met |
| U2-NFR4.3 | `u2-course-search/nfr-requirements/performance-requirements.md` | 期間の前後が逆のとき、入力欄の近くに案内を出し、一覧はそれまでのまま（BR11.5） | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U2-NFR4.4 | `u2-course-search/nfr-requirements/performance-requirements.md` | 詳細から一覧へ戻ったとき、条件と結果が保たれる（BR11.8） | 詳細から戻ると条件と件数（4 件）が保たれる | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U2-NFR5.1 | `u2-course-search/nfr-requirements/performance-requirements.md` | 探索条件のすべての入力（キーワード・カテゴリ・空席ありのみ・開始日・終了日・並べ替え）と「条件をリセット」「再試行」を、キーボードだけで操作できる | Tab で探索の入力・並べ替えに届く | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U2-NFR5.2 | `u2-course-search/nfr-requirements/performance-requirements.md` | すべての入力にラベルがある | 名前のないフォーム部品 0 件 | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U2-NFR5.3 | `u2-course-search/nfr-requirements/performance-requirements.md` | 件数の変化と、0 件・失敗・期間の案内を、読み上げ可能な文字で伝える（ライブリージョンと、入力欄に結び付けた説明） | 件数と案内にライブリージョン（course-search-results） | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U2-NFR2.1 | `u2-course-search/nfr-requirements/reliability-requirements.md` | 探索はデータを書き換えない。探索の受け口を何回呼んでも、残席・残高・チケットは変わらない | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U2-NFR2.2 | `u2-course-search/nfr-requirements/reliability-requirements.md` | 入力中に連続して送った探索の応答が順不同で届いても、表示は最後に送った条件の結果になる（BR11.2） | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U2-NFR2.3 | `u2-course-search/nfr-requirements/reliability-requirements.md` | 探索の失敗（通信の失敗・5xx・400）で画面が操作不能にならない。失敗の表示と「再試行」、または一覧を保った案内を出す（BR11.7、BR11.9） | Web API に接続できないとき、画面が JavaScript の例外（Error の文字列化）で止まり「読み込んでいます」のまま。失敗の表示と「再試行」が出ない | e2e: Web API を止めて画面を開く（pageerror TypeError reading _1） | build-and-test | Not Met |
| U2-NFR6.1 | `u2-course-search/nfr-requirements/reliability-requirements.md` | 条件の検証・キーワードの正規化・絞り込み・並べ替えは純粋関数で書く。現在時刻は引数で受け取る | 探索の関数は現在時刻を引数で受け取る純粋関数 | コードレビュー／自動テスト | build-and-test | Met |
| U2-NFR6.2 | `u2-course-search/nfr-requirements/reliability-requirements.md` | キーワードの入力の待ち（150 ms）は、画面の更新関数が「待ちのコマンド」を返し、待ち終わりのメッセージの番号で最新かを判断する形にする。更新関数の中で時計やタイマーを直接使わな… | 待ちは更新関数が返すコマンド（画面のテストで確認） | bash scripts/verify.sh（画面 57 件） | build-and-test | Met |
| U2-NFR6.3 | `u2-course-search/nfr-requirements/reliability-requirements.md` | 並べ替えの比較は、どの並べ方でも最後にコース ID で比べ、毎回同じ順になる（安定した全順序） | 並べ替えのテストで同値はコース ID 順 | bash scripts/verify.sh | build-and-test | Met |
| U2-NFR8.1 | `u2-course-search/nfr-requirements/reliability-requirements.md` | Rust の探索のテストは `cargo test` で、画面の探索のテストは `moon -C web test --target js` で、手作業の準備なしに実行できる（U1… | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U2-NFR8.2 | `u2-course-search/nfr-requirements/reliability-requirements.md` | 受け入れ基準（AC1.2.1〜AC1.9.4）の振る舞いを、Web API のテスト（絞り込み・並べ替え・件数・400）と画面の更新関数のテスト（待ち・古い応答・URL の復元・0… | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U2-NFR8.3 | `u2-course-search/nfr-requirements/reliability-requirements.md` | 時刻に依存する探索のテスト（今後の日程、期間）は固定の時刻で行う | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U2-NFR9.1 | `u2-course-search/nfr-requirements/reliability-requirements.md` | U2 で前提のツールを増やさない。Construction の確認コマンド（`bash scripts/verify.sh`）で U2 のテストも実行される | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U2-NFR7.1 | `u2-course-search/nfr-requirements/scalability-requirements.md` | 絞り込みと並べ替えは CourseCatalog の純粋関数として書き、データストアに依存しない（ADR-001・ADR-006）。将来 PostgreSQL の検索に置き換えると… | 探索は CourseCatalog の純粋関数（依存は chrono・unicode-normalization） | コードレビュー | build-and-test | Met |
| U2-NFR7.2 | `u2-course-search/nfr-requirements/scalability-requirements.md` | 絞り込みと並べ替えのテストは、データストアを使わず、固定のコース・日程・発行済み数を返す関数を渡して動かす | 固定の値を返す関数で業務の部品をテスト | コードの検索とレビュー／bash scripts/verify.sh | build-and-test | Met |
| U2-NFR7.3 | `u2-course-search/nfr-requirements/scalability-requirements.md` | 将来 PostgreSQL に移すときの注意（キーワードの正規化を検索用の列に持たせる、よみがなに索引を張る、期間は開催開始日の索引で絞る）を、探索の関数のコメントに書いておく | PostgreSQL への移行の注意を search.rs に記載 | コードの検索 | build-and-test | Met |
| U2-NFR7.4 | `u2-course-search/nfr-requirements/scalability-requirements.md` | 画面の探索条件の状態と URL の変換は、U3 以降で条件を足しても既存の項目を変えずに足せる形にする | U3 で探索の項目を変えずに足した（U2 のテストは成功のまま） | bash scripts/verify.sh | build-and-test | Met |
| U2-NFR3.1 | `u2-course-search/nfr-requirements/security-requirements.md` | 形の決まった条件（カテゴリ・空席ありのみ・開始日・終了日・期間の前後・並べ替え）は、Web API がすべて検証し、不正なら 400 と不正な条件の一覧を返す。結果は返さない BR… | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U2-NFR3.2 | `u2-course-search/nfr-requirements/security-requirements.md` | キーワードは自由入力として拒否しない。ただし探索に使うのは先頭 200 文字までとし、それより長い部分は捨てる。キーワードの長さで処理の時間が大きく伸びないようにする BR8.1 | 数万文字のキーワード 10.8 ms | cargo test -p web_api --test performance/search/my_tickets -- --nocapture | build-and-test | Met |
| U2-NFR3.3 | `u2-course-search/nfr-requirements/security-requirements.md` | キーワードは文字列の一致だけで照合し、正規表現やほかの命令として解釈しない BR9.1 | str::contains の部分一致だけで照合（正規表現の依存なし） | コードレビュー（crates/course_catalog/src/search.rs） | build-and-test | Met |
| U2-NFR3.4 | `u2-course-search/nfr-requirements/security-requirements.md` | 画面は、キーワード・コース名・概要など Web API から受け取った文字列も、利用者が入力した文字列も、文字として表示し、HTML として解釈させない（U1 の NFR3.11 … | <img onerror> を含むキーワードが実行されない | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U2-NFR3.5 | `u2-course-search/nfr-requirements/security-requirements.md` | URL のクエリの値は、画面が項目ごとに検証してから使う。不正な値は無視し、そのまま Web API に送らない BR11.4 | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U2-NFR3.6 | `u2-course-search/nfr-requirements/security-requirements.md` | 400 の応答に含めるのは、不正な条件の項目名と短い理由だけにし、受け取った値そのものや内部の情報は返さない BR8.6 | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U3-NFR10.1 | `u3-my-tickets/nfr-requirements/observability-requirements.md` | 一覧の要求も、U1 と同じく要求ごとに 1 行のログ（メソッド・パス・状態コード・処理時間）を出す | 一覧の要求も 1 行 | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U3-NFR10.2 | `u3-my-tickets/nfr-requirements/observability-requirements.md` | チケット番号・従業員名・コース名など一覧の内容はログに出さない（従業員 ID も U1 と同じ扱いにとどめる） | ログにチケット番号・従業員名が出ない | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U3-NFR10.3 | `u3-my-tickets/nfr-requirements/observability-requirements.md` | チケットの日程やコースが見つからないデータの不整合で 5xx にしたときは、どのチケット ID で起きたかをログに残す | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U3-NFR1.1 | `u3-my-tickets/nfr-requirements/performance-requirements.md` | 購入済みチケットの一覧（`GET /api/tickets`） 初期データの規模で、95% の要求が 200 ms 以内に応答する（NFR1 はコース一覧・詳細の目標だが、U3 の… | 一覧 p95 0.275 ms | cargo test -p web_api --test performance/search/my_tickets -- --nocapture | build-and-test | Met |
| U3-NFR1.2 | `u3-my-tickets/nfr-requirements/performance-requirements.md` | 一覧のチケットを開く（BR14.4） 通信せず、操作した直後の再描画で詳細を表示する（BR13.2） | 開閉で通信しない（画面のテスト）。ブラウザで開閉を確認 | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U3-NFR4.1 | `u3-my-tickets/nfr-requirements/performance-requirements.md` | 一覧の読み込み中・0 件・読み込み失敗の 3 つを、文言の違う別の表示にする（BR14.2・BR14.3） | Web API に接続できないとき、画面が JavaScript の例外（Error の文字列化）で止まり「読み込んでいます」のまま。失敗の表示と「再試行」が出ない | e2e: Web API を止めて画面を開く（pageerror TypeError reading _1） | build-and-test | Not Met |
| U3-NFR4.2 | `u3-my-tickets/nfr-requirements/performance-requirements.md` | 0 件では「コース一覧へ」、失敗では「再試行」を必ず出す | Web API に接続できないとき、画面が JavaScript の例外（Error の文字列化）で止まり「読み込んでいます」のまま。失敗の表示と「再試行」が出ない | e2e: Web API を止めて画面を開く（pageerror TypeError reading _1） | build-and-test | Not Met |
| U3-NFR4.3 | `u3-my-tickets/nfr-requirements/performance-requirements.md` | ヘッダーの「購入済みチケット」と、完了画面の「購入済みチケットを見る」から、1 回の操作で一覧へ移れる（BR14.6・BR14.7） | ヘッダー・完了画面のリンクから 1 回で一覧へ | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U3-NFR5.1 | `u3-my-tickets/nfr-requirements/performance-requirements.md` | ヘッダーのリンク・完了画面のリンク・開閉のボタン・「再試行」・「コース一覧へ」を、キーボードだけで操作できる | ヘッダーのリンク・完了画面のリンク・開閉ボタン・「コース一覧へ」をキーボードで操作（「再試行」は接続失敗の不具合で確認できず） | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U3-NFR5.2 | `u3-my-tickets/nfr-requirements/performance-requirements.md` | 開閉のボタンに名前（「詳細を見る」「詳細を閉じる」）を付け、開いているかどうかを支援技術に伝える（BR14.4） | aria-expanded が false→true、aria-controls の先が存在 | e2e（Playwright）・Web API のログ | build-and-test | Met |
| U3-NFR5.3 | `u3-my-tickets/nfr-requirements/performance-requirements.md` | 「受講済み」を色だけでなく文字で示す（BR14.5） | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U3-NFR2.1 | `u3-my-tickets/nfr-requirements/reliability-requirements.md` | 一覧はデータを書き換えない。一覧の受け口を何回呼んでも、残席・残高・チケットは変わらない | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U3-NFR2.2 | `u3-my-tickets/nfr-requirements/reliability-requirements.md` | 購入の直後に一覧を開くと、いま購入したチケットが含まれる（一覧を開くたびに読み込み直す。BR14.1） | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U3-NFR2.3 | `u3-my-tickets/nfr-requirements/reliability-requirements.md` | 一覧の読み込みに失敗しても画面が操作不能にならず、失敗の表示と「再試行」を出す。一覧の画面でないときに届いた応答は表示に使わない | Web API に接続できないとき、画面が JavaScript の例外（Error の文字列化）で止まり「読み込んでいます」のまま。失敗の表示と「再試行」が出ない | e2e: Web API を止めて画面を開く（pageerror TypeError reading _1） | build-and-test | Not Met |
| U3-NFR6.1 | `u3-my-tickets/nfr-requirements/reliability-requirements.md` | 一覧の絞り込み・受講済みの判定・並べは純粋関数で書く。現在時刻は引数で受け取る | 一覧の関数は現在時刻を引数で受け取る純粋関数 | コードレビュー／自動テスト | build-and-test | Met |
| U3-NFR6.2 | `u3-my-tickets/nfr-requirements/reliability-requirements.md` | 開閉の状態は画面の Model の不変な値（開いているチケット ID の集合）で持ち、更新関数が新しい値を返す | open_ticket_ids を不変の配列で持ち toggle_ticket_id が新しい配列を返す（画面のテスト） | bash scripts/verify.sh | build-and-test | Met |
| U3-NFR6.3 | `u3-my-tickets/nfr-requirements/reliability-requirements.md` | 並べの比較は、開催日時・購入日時・チケット番号の順で比べ、毎回同じ順になる（安定した全順序。BR12.2） | 同点の並びのテストが成功 | cargo test -p purchasing owned_tickets_tests | build-and-test | Met |
| U3-NFR8.1 | `u3-my-tickets/nfr-requirements/reliability-requirements.md` | Rust の一覧のテストは `cargo test` で、画面のテストは `moon -C web test --target js` で、手作業の準備なしに実行できる（U1・U2… | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U3-NFR8.2 | `u3-my-tickets/nfr-requirements/reliability-requirements.md` | 受け入れ基準（AC4.2.1〜AC4.2.5）の振る舞いを、Purchasing と Web API のテスト（並び・受講済み・他人のチケットを含めない・0 件）と画面のテスト（読… | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U3-NFR8.3 | `u3-my-tickets/nfr-requirements/reliability-requirements.md` | 受講済みの判定と並びのテストは固定の時刻で行い、開始日時ちょうど・同じ開催日時・同じ購入日時の場合を含める | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U3-NFR9.1 | `u3-my-tickets/nfr-requirements/reliability-requirements.md` | U3 で前提のツールを増やさない。Construction の確認コマンド（`bash scripts/verify.sh`）で U3 のテストも実行される | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U3-NFR7.1 | `u3-my-tickets/nfr-requirements/scalability-requirements.md` | 一覧の絞り込み（操作中の従業員）・受講済みの判定・並べは Purchasing の純粋関数として書き、データストアに依存しない。日程・コース・従業員は引く関数として、現在時刻は値と… | Purchasing の純粋関数。データストアに依存しない | コードレビュー | build-and-test | Met |
| U3-NFR7.2 | `u3-my-tickets/nfr-requirements/scalability-requirements.md` | 将来 PostgreSQL に移すときは、従業員 ID でチケットを引く検索（従業員 ID の索引）に置き換えられる形にし、その注意を関数のコメントに書いておく | PostgreSQL への移行の注意を owned_tickets.rs に記載 | コードの検索 | build-and-test | Met |
| U3-NFR7.3 | `u3-my-tickets/nfr-requirements/scalability-requirements.md` | 一覧のテストは、データストアを使わず、固定のチケット・日程・コース・従業員を返す関数を渡して動かす | PostgreSQL への移行の注意をコメントに記載（store.rs・search.rs・owned_tickets.rs） | コードの検索とレビュー／bash scripts/verify.sh | build-and-test | Met |
| U3-NFR7.4 | `u3-my-tickets/nfr-requirements/scalability-requirements.md` | 画面の Route と状態は、U1・U2 の項目を変えずに足す | U1・U2 の項目を変えずに Route と状態を足した（U1・U2 のテストは成功のまま） | bash scripts/verify.sh | build-and-test | Met |
| U3-NFR3.1 | `u3-my-tickets/nfr-requirements/security-requirements.md` | 一覧は Cookie の操作中の従業員のチケットだけを返す。クエリや本文で従業員を指定する手段は設けず、付けられても無視する BR12.1、BR13.1 | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U3-NFR3.2 | `u3-my-tickets/nfr-requirements/security-requirements.md` | Cookie の従業員 ID が存在しない従業員を指すときは、空の一覧を返し、ほかの従業員のチケットや内部の情報を返さない BR12.1 | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
| U3-NFR3.3 | `u3-my-tickets/nfr-requirements/security-requirements.md` | 一覧は読み出しの要求なので、U1 の CSRF 対策（BR6.3）の対象外とする。書き込みの受け口は U3 で増やさない — | /api/tickets は読み出し用のルーター。書き込みの受け口は 2 つのまま | コードレビュー（crates/web_api/src/app.rs） | build-and-test | Met |
| U3-NFR3.4 | `u3-my-tickets/nfr-requirements/security-requirements.md` | 画面は、コース名・会場・従業員名など Web API から受け取った文字列を文字として表示し、HTML として解釈させない（U1 の NFR3.11 を一覧にも当てはめる） — | 生の HTML を入れる箇所の検索で 0 件 | コードの検索 | build-and-test | Met |
| U3-NFR3.5 | `u3-my-tickets/nfr-requirements/security-requirements.md` | 従業員を切り替えたら、前の従業員のチケットを画面に残さない BR14.8 | 自動テストが成功 | bash scripts/verify.sh（Rust 88 件・画面 57 件 成功） | build-and-test | Met |
