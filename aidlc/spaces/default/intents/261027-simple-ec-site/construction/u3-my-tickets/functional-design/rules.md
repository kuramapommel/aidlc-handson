# 業務ルール（U3 購入済みチケット）

U3（`u3-my-tickets`）の業務ルールを定める。下の YAML がルールの正本で、`functional-spec.md` のルールの要約はここから作った派生の表である。U1（BR1〜BR7）・U2（BR8〜BR11）と番号が重ならないよう、U3 は BR12 から始める。

グループ:

- BR12 購入済みチケットの一覧（Purchasing）
- BR13 一覧の受け口（WebApi）
- BR14 一覧の画面と導線（WebUi）

`source` には要件（FR・NFR）と、機能設計の確認事項（`functional-design-questions.md` の Q 番号。「前提」は確認事項の「前提（質問しないこと）」）を書く。

```yaml
rules:
  # ---------- BR12 購入済みチケットの一覧 ----------
  - id: BR12.1
    statement: 購入済みチケットの一覧には、操作中の従業員が購入したチケットだけを含める
    category: authorization
    applies_to: [OwnedTicketList, Ticket]
    trigger: 購入済みチケットの一覧の要求
    logic: >
      IF チケットの employeeId = 操作中の従業員 ID THEN 一覧に含める。それ以外のチケットは含めない。
      操作中の従業員 ID が存在しない従業員を指すときは、チケットはないものとして 0 件の一覧にする
    violation: 他人のチケットの内容を返さない（U1 の BR5.10 と同じ考え方）
    source: [FR6.3, NFR3]
  - id: BR12.2
    statement: 一覧は、これからのチケットを開催日時の近い順に並べ、そのあとに受講済みのチケットを開催日時の新しい順に並べる
    category: calculation
    applies_to: [OwnedTicketList]
    trigger: 購入済みチケットの一覧の作成
    logic: >
      これから（upcoming）のチケットを先に、受講済み（attended）のチケットを後に置く。
      これからのチケットどうしは startsAt の早い順、受講済みのチケットどうしは startsAt の遅い順。
      startsAt が同じときは、どちらのグループでも purchasedAt の早い順、それでも同じなら ticketNumber の昇順にする
    violation: 該当なし（並びの計算）
    source: [FR6.3, "Q1", "前提"]
  - id: BR12.3
    statement: 開催日時（開始）が Web API の現在時刻以前のチケットを受講済みとする
    category: calculation
    applies_to: [OwnedTicket]
    trigger: 購入済みチケットの一覧の作成
    logic: >
      IF Session.startsAt ≤ 現在時刻 THEN attendance = attended ELSE attendance = upcoming。
      現在時刻は Web API の時計で、関数として外から受け取る（U1 の日程の受付終了と同じ判定の時点）
    violation: 該当なし（導出の計算）
    source: [FR6.3, "Q1"]
  - id: BR12.4
    statement: 一覧の各行にはコース名・開催日時・会場または形式・チケット番号を表示し、残りの項目は開いたときに表示する
    category: policy
    applies_to: [OwnedTicket, WebUi]
    trigger: 購入済みチケットの一覧の表示
    logic: >
      行には courseTitle・開催日時（startsAt〜endsAt）・会場または形式・ticketNumber を表示する。
      開いたときに、これに従業員名・購入日時・支払ったポイントを加え、完了画面と同じ 7 項目（U1 の BR5.9）をそろえる
    violation: 該当なし（表示のルール）
    source: [FR6.1, FR6.3, "Q3"]

  # ---------- BR13 一覧の受け口 ----------
  - id: BR13.1
    statement: 購入済みチケットの一覧は、操作中の従業員について Web API が 1 回の要求で返す
    category: policy
    applies_to: [WebApi, OwnedTicketList]
    trigger: 購入済みチケットの一覧の取得（読み取りの要求）
    logic: >
      操作中の従業員 ID は U1 と同じく Cookie から受け取る（URL や本文で従業員を指定させない）。
      BR12.1 で絞り、BR12.3 で受講済みを決め、BR12.2 で並べた一覧を返す。
      要求ごとのログは U1 と同じ項目（メソッド・パス・状態コード・処理時間）を出し、チケットの内容はログに出さない
    violation: 該当なし（0 件でも成功として返す）
    source: [FR6.3, NFR3, NFR10]
  - id: BR13.2
    statement: 一覧の応答には、開いたときに表示する項目まで含め、開くたびに通信しない
    category: policy
    applies_to: [WebApi, OwnedTicket]
    trigger: 購入済みチケットの一覧の取得
    logic: 各 OwnedTicket に AC4.1.1 の 7 項目と attendance を含めて返す。画面はチケットを開くときに Web API を呼ばない
    violation: 該当なし
    source: [FR6.3, "Q2", "Q3"]

  # ---------- BR14 一覧の画面と導線 ----------
  - id: BR14.1
    statement: 購入済みチケットの一覧は専用の画面（URL）で表示し、開くたびに最新の一覧を読み込む
    category: policy
    applies_to: [WebUi]
    trigger: 購入済みチケットの一覧の画面を開く
    logic: >
      画面の URL は `/my-tickets`（操作中の従業員 ID は URL に含めない。U1 の BR6.1）。
      この画面を開くたびに Web API から一覧を読み込み直し、以前に読み込んだ一覧を使い回さない
    violation: 該当なし
    source: [FR6.3, "前提"]
  - id: BR14.2
    statement: 一覧の読み込み中・0 件・読み込み失敗は別の表示にし、失敗したときは理由と「再試行」を示す
    category: policy
    applies_to: [WebUi]
    trigger: 一覧の読み込みの状態が変わる
    logic: >
      読み込み中は「購入済みチケットを読み込んでいます」と表示し、0 件の表示と区別する。
      失敗したときは理由と「再試行」を表示し、「再試行」で同じ一覧を読み込み直す（U1 の BR7.1 と同じ扱い）
    violation: 該当なし
    source: [NFR4, "前提"]
  - id: BR14.3
    statement: 購入済みのチケットがないときは、チケットがないことと、コース一覧へ移動する操作を示す
    category: policy
    applies_to: [WebUi]
    trigger: 読み込んだ一覧が 0 件
    logic: 「購入済みのチケットはありません」と表示し、「コース一覧へ」の操作（コース一覧の画面への移動）を置く
    violation: 該当なし
    source: [FR6.3]
  - id: BR14.4
    statement: 一覧のチケットはその場で開いて詳細を表示し、画面は移らない
    category: policy
    applies_to: [WebUi, OwnedTicket]
    trigger: 一覧のチケットの「詳細を見る」を操作する
    logic: >
      各チケットに開閉のボタンを置く。最初はすべて閉じている。開くと BR12.4 の 7 項目を行の中に表示し、もう一度操作すると閉じる。
      チケットごとに別々に開閉でき、複数を同時に開いてもよい。開閉の状態は URL に残さず、一覧を読み込み直すと閉じた状態に戻る。
      ボタンには名前（「詳細を見る」「詳細を閉じる」）を付け、開いているかどうかを支援技術にも伝える（NFR5）
    violation: 該当なし
    source: [FR6.3, NFR5, "Q2"]
  - id: BR14.5
    statement: 受講済みのチケットには「受講済み」と文字で表示する
    category: policy
    applies_to: [WebUi, OwnedTicket]
    trigger: 一覧の表示
    logic: IF attendance = attended THEN 行に「受講済み」と文字で表示する（色だけで伝えない）
    violation: 該当なし
    source: [NFR5, "Q1"]
  - id: BR14.6
    statement: 購入の完了画面に「購入済みチケットを見る」を置き、一覧の画面へ移る
    category: policy
    applies_to: [WebUi]
    trigger: 購入の完了画面の表示
    logic: >
      完了画面（購入直後）に「購入済みチケットを見る」を追加する。U1 の「コース一覧へ戻る」はそのまま残す。
      移った先の一覧は BR14.1 で読み込み直すので、いま購入したチケットが含まれる
    violation: 該当なし
    source: [FR6.3]
  - id: BR14.7
    statement: ヘッダーに「購入済みチケット」を置き、どの画面からでも一覧の画面へ移れる
    category: policy
    applies_to: [WebUi]
    trigger: すべての画面の表示
    logic: U1 のヘッダー（従業員名・残高・切り替え）に「購入済みチケット」のリンクを追加する。U1 のヘッダーの表示は変えない
    violation: 該当なし
    source: [FR6.3]
  - id: BR14.8
    statement: 従業員を切り替えたら、前の従業員の購入済みチケットを画面に残さない
    category: authorization
    applies_to: [WebUi]
    trigger: 従業員の切り替え
    logic: >
      U1 の BR6.4 のとおりコース一覧へ戻り、読み込み済みの購入済みチケットの一覧と開閉の状態を捨てる。
      次に一覧を開いたときは、切り替えた後の従業員の一覧を読み込む
    violation: 前の従業員のチケットを表示し続けない
    source: [NFR3, "前提"]
```

## ルールの要約

| ID | 内容 | 種類 | 出典 |
| --- | --- | --- | --- |
| BR12.1 | 一覧は操作中の従業員のチケットだけ。存在しない従業員は 0 件 | authorization | FR6.3, NFR3 |
| BR12.2 | これからを開催日時の近い順、そのあと受講済みを新しい順。同点は購入日時→チケット番号 | calculation | FR6.3, Q1 |
| BR12.3 | 開始日時が現在時刻以前なら受講済み | calculation | FR6.3, Q1 |
| BR12.4 | 行はコース名・開催日時・会場または形式・チケット番号。開くと 7 項目 | policy | FR6.1, FR6.3, Q3 |
| BR13.1 | Web API は Cookie の従業員について一覧を 1 回の要求で返す | policy | FR6.3, NFR3, NFR10 |
| BR13.2 | 応答に開いたときの項目まで含め、開くたびに通信しない | policy | FR6.3, Q2, Q3 |
| BR14.1 | 一覧は `/my-tickets` の画面。開くたびに読み込み直す | policy | FR6.3 |
| BR14.2 | 読み込み中・0 件・失敗（理由と再試行）を区別 | policy | NFR4 |
| BR14.3 | 0 件は「購入済みのチケットはありません」と「コース一覧へ」 | policy | FR6.3 |
| BR14.4 | その場で開閉して詳細を表示。画面は移らない | policy | FR6.3, NFR5, Q2 |
| BR14.5 | 受講済みは文字で表示 | policy | NFR5, Q1 |
| BR14.6 | 完了画面に「購入済みチケットを見る」 | policy | FR6.3 |
| BR14.7 | ヘッダーに「購入済みチケット」 | policy | FR6.3 |
| BR14.8 | 切り替えたら前の従業員のチケットを残さない | authorization | NFR3 |
