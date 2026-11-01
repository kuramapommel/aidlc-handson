# 画面の部品（U2 コース探索）

U2 で U1 の画面（`../../u1-purchase-skeleton/functional-design/frontend-components.md`）に足す部品・状態・メッセージを定める。TEA（ADR-007）で、更新関数と描画関数は純粋関数とし、Web API の呼び出し・URL の書き換え・時間待ちはコマンドとして外に出す。TEA の実行環境・描画・URL の扱いは rabbita、通信は mizchi/js_web を使う（U1 と同じ）。

## 1. 部品の構成（CourseListPage に足すもの）

```text
CourseListPage
├── SearchForm            探索条件の入力（BR11.10）
│   ├── KeywordInput      キーワード（自由入力。上限なし）
│   ├── CategorySelect    カテゴリ（すべて・4 カテゴリ）
│   ├── AvailableOnlyToggle 空席ありのみ（チェックボックス）
│   ├── PeriodInputs      開始日・終了日（日付の入力欄）と、前後が逆のときの案内（BR11.5）
│   └── SortSelect        並べ替え（4 つ）
├── ResultCount           件数（U1 のもの。探索の結果で更新）
├── LoadState             読み込み中・失敗と「再試行」（U1 のもの。BR11.7）
├── EmptyState            0 件と「条件をリセット」（BR11.6）
├── ConditionError        万一の 400 の案内（BR11.9）
└── CourseCard*           U1 のもの（変更なし）
```

## 2. 状態（Model に足すもの）

| 項目 | 型（概念） | 中身 |
| --- | --- | --- |
| searchInput | SearchInput | 入力欄の今の値（キーワードの文字列・カテゴリ・空席ありのみ・開始日・終了日・並べ替え） |
| periodError | 文字列またはなし | 期間の前後が逆のときの案内（BR11.5） |
| keywordDebounce | 待ちの番号 | キーワードの入力ごとに進める番号。待ち終わりのメッセージの番号と同じときだけ送る（BR11.1） |
| searchRequestSeq | 要求の番号 | 探索を送るたびに進める番号（BR11.2） |
| courseList | U1 の Remote に「0 件（Empty）」の区別を足す | 結果あり・0 件・読み込み中・失敗 |
| conditionError | 不正な条件の一覧またはなし | 400 のときの案内（BR11.9） |

探索に送る条件（SearchCondition）は保存せず、searchInput から描画や送信のたびに計算する（同じ値を 2 か所に持たないため）。

## 3. 画面と URL

| Route | URL（例） | 復元と取得 |
| --- | --- | --- |
| CourseList（U1 のものに条件を足す） | `/?q=AI%20入門&category=technology&available=true&from=2026-11-10&to=2026-11-15&sort=fee-asc` | URL の各項目を項目ごとに確かめ（BR11.4）、正しい項目だけを searchInput に入れ、不正な項目を URL から取り除き、探索を送る |

条件の変更では、履歴を増やさずに URL を書き換える（BR11.3）。詳細への移動は U1 と同じく履歴を増やすので、戻る操作で条件付きの一覧に戻る（BR11.8）。詳細の「コース一覧へ戻る」は、詳細へ移る前の一覧の URL（条件付き）へ移る。

## 4. メッセージと更新（足すもの）

| メッセージ | 状態の変化 | コマンド |
| --- | --- | --- |
| KeywordChanged(文字列) | searchInput のキーワードを変え、keywordDebounce を進める | 150 ms 後に KeywordSettled(番号) を届ける |
| KeywordSettled(番号) | 番号が最新でなければ何もしない。最新なら searchRequestSeq を進め、courseList を読み込み中に | 探索の要求、URL の書き換え |
| CategoryChanged・AvailableOnlyChanged・SortChanged | searchInput を変え、searchRequestSeq を進め、読み込み中に | 探索の要求、URL の書き換え |
| PeriodChanged(開始日, 終了日) | searchInput を変える。前後が逆なら periodError を設定して送らない（一覧はそのまま）。正しければ periodError を消し、番号を進め、読み込み中に | 正しいときだけ探索の要求、URL の書き換え |
| SearchLoaded(番号, 結果) | 番号が searchRequestSeq と違えば捨てる（BR11.2）。結果が 1 件以上なら結果あり、0 件なら 0 件、通信失敗・5xx なら失敗、400 なら一覧をそのままにして conditionError を設定 | — |
| ResetConditions | searchInput を既定に戻し、periodError と conditionError を消し、番号を進め、読み込み中に | 探索の要求、URL のクエリを消す |
| RetrySearch | 番号を進め、読み込み中に | 今の条件で探索の要求 |
| UrlChanged（一覧の URL） | URL から条件を復元し（BR11.4）、番号を進め、読み込み中に | 不正な項目を取り除いた URL への書き換え（必要なときだけ）、探索の要求 |

従業員の切り替え（U1 の BR6.4）でコース一覧へ戻るときは、探索条件も既定に戻す（前の従業員の操作を残さない）。

## 5. 描画のルール（足すもの）

| 部品 | ルール |
| --- | --- |
| KeywordInput | ラベル「キーワード」。入力の上限は設けない。入力中の値は利用者の打ったとおりに表示する（正規化した値は表示しない） |
| CategorySelect | ラベル「カテゴリ」。選択肢は「すべて」・マネジメント・技術・ビジネススキル・コンプライアンス |
| AvailableOnlyToggle | ラベル「空席ありのみ」のチェックボックス |
| PeriodInputs | ラベル「開始日」「終了日」の日付の入力欄。前後が逆のとき、入力欄のすぐ下に案内を文字で出し、入力欄に説明を結び付けて読み上げられるようにする |
| SortSelect | ラベル「並べ替え」。開催日が近い順（既定）・受講料の安い順・受講料の高い順・コース名順 |
| ResultCount | 「13 件」の形。読み込み中は前の件数を残さず、読み込み中と分かる表示にする |
| EmptyState | 「条件に合うコースがありません」と「条件をリセット」ボタン |
| LoadState（失敗） | 「コースを読み込めませんでした」と「再試行」ボタン（0 件の表示と文言を分ける） |
| ConditionError | 「条件を確かめてください」と不正な条件の説明 |

操作できる部品には `data-testid` を付ける。結果の件数と一覧の変化は、支援技術に知らせる領域（ライブリージョン）で伝える（NFR5）。

## 6. 入力のチェック

| 入力 | 画面でのチェック | 最終的な判断 |
| --- | --- | --- |
| キーワード | しない（自由入力。どんな文字列も正しい） | Web API が正規化して先頭 200 文字で探す（BR8.1） |
| カテゴリ・並べ替え | 選択肢から選ぶだけ | Web API の BR8.2・BR8.5 |
| 空席ありのみ | チェックボックス | Web API の BR8.3 |
| 期間 | 日付の入力欄（空か正しい日付）と前後の確認（BR11.5） | Web API の BR8.4 |
| URL の各項目 | 項目ごとに BR8.2〜BR8.5 と同じ規則で確かめ、不正な項目は無視（BR11.4） | Web API の BR8.6 |

## 7. Web API との連携（変えるもの）

| コマンド | 受け口 | 送るもの | 備考 |
| --- | --- | --- | --- |
| 探索の要求 | `GET /api/courses` | `q`・`category`・`available`・`from`・`to`・`sort`（既定の値と空の値は送らない） | 応答に要求の番号を結び付けて SearchLoaded にする。400 の本文（不正な条件の一覧）も読む |
