# 画面の部品（U1 購入の最小の流れ）

U1 の Web 画面（WebUi）の部品構成・状態・メッセージ・Web API との連携を定める。画面は The Elm Architecture（TEA。ADR-007）で組み立てる。画面全体の状態を 1 つの不変なデータで持ち、「メッセージ＋状態 → 新しい状態＋コマンド」の更新関数と「状態 → 画面」の描画関数をどちらも純粋関数にする。Web API の呼び出し・URL の書き換え・時刻の取得はコマンドとして外に出す。

具体的なライブラリ（MoonBit 向けの TEA の実装、または自作の薄い実行環境）はコード生成で決める（要件 OQ1）。

## 1. 部品の構成

```text
App
├── Header                 操作中の従業員名・企業名・残高、従業員の切り替え（BR7.11）
│   └── EmployeeSwitcher   切り替え用の従業員一覧と選択（BR6.1、BR6.4）
├── EligibilityNotice      契約外の企業の案内（BR1.3）
└── Page（どれか 1 つ）
    ├── CourseListPage     コース一覧（WF1）
    │   ├── LoadState      読み込み中・失敗と「再試行」（BR7.1）
    │   ├── ResultCount    件数（BR3.4）
    │   └── CourseCard*    コース名・カテゴリ・概要・受講料・直近の開催日・空席のある日程の数
    ├── CourseDetailPage   コースの詳細と日程の選択（WF2）
    │   ├── LoadState
    │   ├── CourseSummary  コースの全項目
    │   ├── SessionList    日程の一覧（選択の部品のまとまり）
    │   │   └── SessionOption*  開催日時・会場または形式・定員・残席数・状態（BR2.5）
    │   └── ProceedButton  「購入確認へ進む」と案内（BR7.2）
    ├── PurchaseConfirmPage 購入確認（WF3）
    │   ├── PurchaseSummary  コース名・開催日時・会場または形式・受講料
    │   ├── BudgetPreview    現在の残高・購入後の残高 または 不足額（BR4.3、BR4.4）
    │   ├── ConfirmButton    確定（無効化・処理中。BR7.3、BR1.4）
    │   └── PurchaseOutcome  拒否・通信失敗のメッセージと次の操作（BR7.4、BR7.6）
    ├── PurchaseCompletePage 完了と e チケット（WF3・WF5）
    │   └── TicketView       e チケットの項目（BR5.9）
    └── NotFoundPage       コース・チケットが見つからない（BR7.9、BR5.10）
```

`*` は繰り返し表示する部品。

## 2. 画面の状態（Model）

画面全体の状態を 1 つの不変なデータで持つ。読み込みの状態は「未取得・読み込み中・成功・失敗」の 4 つを持つ型（以下「Remote」）で表し、読み込み中・0 件・失敗を区別する（BR7.1）。

| 項目 | 型（概念） | 中身 |
| --- | --- | --- |
| route | Route | 現在の画面（下の 3 節） |
| me | Remote<EmployeeContext> | 操作中の従業員（表示名・企業名・契約状態・利用資格・残高） |
| switcher | 開閉の状態＋Remote<SwitchableEmployee の一覧>＋切り替え中かどうか＋切り替え失敗のメッセージ | 従業員の切り替え |
| courseList | Remote<コースの行の一覧> | コース一覧（U2 が探索条件を加えられるよう、条件を持つ項目を後から足せる形にする） |
| courseDetail | Remote<コースと日程の一覧＞ または 「見つからない」 | 表示中のコースの詳細 |
| selectedSessionId | 日程 ID またはなし | コースの詳細で選んだ日程 |
| purchase | Ready／Blocked／Submitting／Succeeded（e チケット）／Rejected（理由）／NetworkFailed | 購入確認の中の状態（`functional-spec.md` 5.3） |
| ticket | Remote<e チケット> または 「見つからない」 | 完了画面を開き直したときのチケット |

購入確認の Ready と Blocked は保存せず、`me` と `courseDetail` と `selectedSessionId` から描画のたびに計算してもよい（同じ値を 2 か所に持たないため）。

## 3. 画面と URL（Route）

全画面を読み込み直さず、画面内で移動して URL だけを書き換える。URL を直接開いたり再読み込みしたりしたときは、URL から Route を復元して必要なデータを取得する。

| Route | URL（例） | 開いたときに取得するもの |
| --- | --- | --- |
| CourseList | `/` | コース一覧 |
| CourseDetail | `/courses/C001` | コースの詳細 |
| PurchaseConfirm | `/courses/C001/sessions/S0001/confirm` | コースの詳細（選んだ日程を含む）。日程が購入可でなくなっていれば詳細へ戻して理由を見せる |
| PurchaseComplete | `/tickets/{ticketId}` | 購入直後は購入の応答の e チケットを使い、開き直したときはチケット 1 枚の詳細を取得 |
| NotFound | 上のどれかで 404 のとき | — |

操作中の従業員 ID は URL に含めない（Cookie で受け渡す。BR6.1）。

## 4. メッセージ（Msg）と更新

| メッセージ | 状態の変化 | コマンド |
| --- | --- | --- |
| AppStarted(URL) | route を URL から復元。me と画面のデータを読み込み中にする | 操作中の従業員の取得、画面のデータの取得 |
| UrlChanged(URL) / LinkClicked(Route) | route を変える | URL の書き換え、画面のデータの取得 |
| MeLoaded(結果) | me を成功・失敗にする | — |
| CourseListLoaded(結果) | courseList を成功・失敗にする | — |
| RetryCourseList | courseList を読み込み中に戻す | コース一覧の取得 |
| CourseDetailLoaded(結果) | courseDetail を成功・失敗・見つからないにする。selectedSessionId がもう購入可でなければ外す | — |
| SessionSelected(日程 ID) | 購入可の日程のときだけ selectedSessionId を変える（BR2.5） | — |
| ProceedToConfirm | route を PurchaseConfirm にする（選んだ日程があるときだけ。BR7.2） | URL の書き換え |
| BackToDetail（日程を選び直す） | route を CourseDetail にする。purchase を初期に戻す（BR7.8） | URL の書き換え |
| ConfirmClicked | Ready のときだけ purchase を Submitting にする（Submitting 中の押下は無視。BR7.3） | 購入の要求（日程 ID だけを送る。BR5.2） |
| PurchaseSucceeded(e チケット, 購入後の残高) | purchase を Succeeded、me の残高を購入後の値に更新（BR7.7）、route を PurchaseComplete | URL の書き換え |
| PurchaseRejected(理由) | purchase を Rejected(理由) にする。残高は変えない（BR7.4） | — |
| PurchaseNetworkFailed | purchase を NetworkFailed にする（BR7.6） | — |
| RetryPurchase（もう一度試す） | purchase を Submitting にする | 購入の要求 |
| ChooseAnotherSession（別の日程を選ぶ） | route を CourseDetail、courseDetail を読み込み中、selectedSessionId を外す（BR7.5） | コースの詳細の取得 |
| SwitcherOpened | switcher を開き、従業員一覧を読み込み中にする | 切り替え用の従業員一覧の取得 |
| EmployeeChosen(従業員 ID) | switcher を切り替え中にする | 従業員の切り替えの要求 |
| EmployeeSwitched(結果) | 成功: 選んだ日程・購入確認・購入の結果・チケットを捨て、route を CourseList、me を新しい従業員にし、courseList を読み込み中にする（BR6.4）。失敗: 失敗のメッセージを出し、me は変えない | 成功時: URL の書き換え、コース一覧の取得 |

古い応答の扱い: 画面の移動や切り替えの後に、前の画面の要求の応答が届いたら捨てる（応答に要求時の route と従業員を対応づけて照合する）。U2 で探索の「最後の要求の結果だけを反映する」仕組みを足すときも同じ考え方を使う。

## 5. 描画のルール

| 部品 | ルール |
| --- | --- |
| Header | どの画面でも表示名・企業名・残高を表示する（BR7.11）。残高は「60,000 pt」の形（BR7.10）。me が読み込み中なら読み込み中の表示 |
| EmployeeSwitcher | 名前の付いたボタンで開き、一覧は従業員名・企業名・契約状態（「契約終了」も文字で）を表示する。キーボードで開閉・選択できる（NFR5） |
| EligibilityNotice | 利用資格が eligible でないとき、一覧・詳細・購入確認の上部に案内を表示する（BR1.3） |
| LoadState | 読み込み中・失敗を別の文言にする。失敗には「再試行」ボタン（BR7.1） |
| CourseCard | 今後の日程がないコースは直近の開催日に「今後の開催予定なし」と表示する（BR3.1）。カード全体をコースの詳細へのリンクにする |
| SessionOption | 選択の部品（単一選択）として描画し、受付終了・満席・購入済みは選択できない状態にして、理由を文字で表示し読み上げ用のテキストにも含める（BR2.5）。残席数は数字で表示し、「残りわずか」などの特別な表示はしない（Q7） |
| ProceedButton | 日程を選んでいなければ無効にし、「日程を選んでください」と案内する（BR7.2） |
| BudgetPreview | 支払えるなら現在の残高・受講料・購入後の残高、支払えないなら不足額を表示する（BR4.3、BR4.4） |
| ConfirmButton | Blocked・Submitting では無効。Submitting では「購入を処理しています」と表示する（BR7.3） |
| PurchaseOutcome | 拒否の理由ごとの文言と次の操作（BR7.4）。通信の失敗は「もう一度試す」と確かめ方の案内（BR7.6） |
| TicketView | チケット番号・従業員名・コース名・開催日時・会場または形式・購入日時・支払ったポイント（BR5.9）。「コース一覧へ戻る」を置く |

状態は色だけで伝えず、必ず文字を添える（NFR5）。ボタンと選択の部品には名前（ラベル）を付ける。

## 6. 入力のチェック

U1 の画面で利用者が入力するのは、日程の選択と従業員の選択だけで、自由入力の欄はない。

| 入力 | 画面でのチェック | 最終的な判断 |
| --- | --- | --- |
| 日程の選択 | 購入可の日程だけ選べる（BR2.5） | Web API の最終チェック（BR5.1） |
| 購入の確定 | 支払えて利用資格があるときだけ確定できる（BR4.2、BR1.4） | Web API の最終チェック（BR5.1） |
| 従業員の選択 | 一覧にある従業員だけを選べる | Web API が一覧にいることを確かめる（BR6.1） |

画面のチェックは利用者の体験のためで、購入の可否と金額の最終判断は Web API が行う（NFR3、ADR-007）。

## 7. Web API との連携

| コマンド | 受け口 | 送るもの | 備考 |
| --- | --- | --- | --- |
| 操作中の従業員の取得 | `GET /api/me` | 資格情報（Cookie）付き | Cookie がなければ Web API が先頭の従業員の Cookie を発行する（BR6.2） |
| コース一覧の取得 | `GET /api/courses` | — | U2 で探索条件をクエリに加える |
| コースの詳細の取得 | `GET /api/courses/{courseId}` | Cookie 付き | 購入済みの表示に操作中の従業員を使う |
| 切り替え用の従業員一覧の取得 | `GET /api/employees` | — | |
| 従業員の切り替え | `POST /api/session/employee` | 従業員 ID、Cookie 付き、CSRF 対策の専用ヘッダー | 成功で Cookie が変わる（BR6.1、BR6.3） |
| 購入 | `POST /api/purchases` | 日程 ID だけ、Cookie 付き、CSRF 対策の専用ヘッダー | 金額は送らない（BR5.2）。409 は拒否、通信の失敗は NetworkFailed |
| チケット 1 枚の詳細 | `GET /api/tickets/{ticketId}` | Cookie 付き | 404 は NotFound |

資格情報付きの通信で Cookie が送られるよう、Web API 側は画面のオリジンだけを資格情報付きで許可する（BR6.3）。
