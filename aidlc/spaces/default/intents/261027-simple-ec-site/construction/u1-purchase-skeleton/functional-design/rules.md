# 業務ルール（U1 購入の最小の流れ）

U1（`u1-purchase-skeleton`）の業務ルールを定める。下の YAML がルールの正本で、`functional-spec.md` のルールの要約はここから作った派生の表である。各ルールは `BR{グループ}.{連番}` で識別する。

グループ:

- BR1 利用資格（Eligibility）
- BR2 日程の状態（CourseCatalog）
- BR3 コース一覧の集計と並び（CourseCatalog）
- BR4 研修予算（TrainingBudget）
- BR5 購入とチケット（Purchasing・InMemoryStore）
- BR6 操作中の従業員と切り替え（WebApi・WebUi）
- BR7 画面の表示と操作（WebUi）

`source` には要件（FR・NFR）と、機能設計の確認事項（`functional-design-questions.md` の Q 番号）を書く。

```yaml
rules:
  # ---------- BR1 利用資格 ----------
  - id: BR1.1
    statement: 契約中の企業に所属する従業員だけが購入できる
    category: authorization
    applies_to: [Employee, Company, Purchasing]
    trigger: 購入の確定（Web API での最終チェック）
    logic: >
      IF 従業員 ID に当たる従業員がいない THEN 利用資格は employee-not-found。
      ELSE IF 所属企業の contractStatus が ended THEN company-not-active。
      ELSE eligible。eligible のときだけ購入の決定に進む
    violation: 購入を拒否し、予算・残席・チケットを一切変えない（理由は BR5.1 の順で 1 つ）
    source: [FR1.3, FR5.2, NFR3]
  - id: BR1.2
    statement: 閲覧（コース一覧・詳細・残高の表示）は契約状態にかかわらず許可する
    category: policy
    applies_to: [WebApi, WebUi]
    trigger: 一覧・詳細・操作中の従業員の取得
    logic: IF 従業員が存在する THEN 契約状態が ended でも閲覧の要求に応える
    violation: 該当なし（閲覧を拒否しない）
    source: [FR1.3, "US5.1"]
  - id: BR1.3
    statement: 契約が終了した企業の従業員には、閲覧中の画面に利用できない旨の案内を出す
    category: policy
    applies_to: [WebUi]
    trigger: 操作中の従業員の利用資格が company-not-active のときの一覧・詳細・購入確認の表示
    logic: >
      IF 利用資格が company-not-active THEN 一覧・詳細・購入確認の上部に
      「ご所属の企業は現在このサービスをご利用いただけません」という案内を文字で表示する
    violation: 該当なし（表示のルール）
    source: [FR1.3, "US5.1"]
  - id: BR1.4
    statement: 利用資格のない従業員の購入確認では確定操作を無効にする
    category: validation
    applies_to: [WebUi]
    trigger: 購入確認の表示
    logic: IF 利用資格が eligible でない THEN 確定ボタンを無効にし、BR1.3 の案内を表示する
    violation: 確定できない（Web API 側でも BR1.1 で拒否する）
    source: [FR1.3, FR4.3]

  # ---------- BR2 日程の状態 ----------
  - id: BR2.1
    statement: 残席数は「定員 − 発行済みチケット数」で計算し、0 未満にしない
    category: calculation
    applies_to: [Session, Ticket]
    trigger: 日程の状態の判定、一覧の集計、購入の決定
    logic: 残席数 = max(0, capacity − その日程の Ticket の数)。発行済み数は呼び出し側から関数として受け取る（ADR-005）
    violation: 発行済み数が定員を超える記録は作らない（BR5.4 で防ぐ）
    source: [FR3.2, NFR2]
  - id: BR2.2
    statement: 開催開始の時刻を過ぎた日程は受付終了とする
    category: constraint
    applies_to: [Session]
    trigger: 日程の状態の判定、購入の決定
    logic: IF 現在時刻 ≥ startsAt THEN 受付終了。現在時刻は Web API の時計で決める
    violation: 受付終了の日程は選べず、購入も拒否する（理由 closed）
    source: [FR3.3, FR5.2, "要件 A6"]
  - id: BR2.3
    statement: 日程の状態は「受付終了 → 満席 → 購入可」の順に判定する
    category: calculation
    applies_to: [Session]
    trigger: 日程の状態の判定
    logic: >
      IF BR2.2 で受付終了 THEN closed。ELSE IF 残席数 = 0 THEN full。ELSE available。
      過去の満席の日程は「受付終了」と表示する
    violation: 該当なし（判定のルール）
    source: [FR3.3]
  - id: BR2.4
    statement: 操作中の従業員が購入済みの日程は「購入済み」と表示し、ほかの状態より優先する
    category: calculation
    applies_to: [Session, Ticket]
    trigger: コースの詳細での日程の表示
    logic: IF 操作中の従業員がその日程の Ticket を持つ THEN 表示用の状態は purchased。ELSE BR2.3 の状態
    violation: 該当なし（表示のルール）
    source: [FR3.4]
  - id: BR2.5
    statement: 選択できるのは表示用の状態が「購入可」の日程だけで、選べない日程は理由を文字で示す
    category: validation
    applies_to: [WebUi]
    trigger: コースの詳細での日程の選択
    logic: >
      IF 表示用の状態が available THEN 選択できる。
      ELSE 選択できない部品として表示し、「受付終了」「満席」「購入済み」を色ではなく文字で示す。
      理由は読み上げ可能なテキストとして日程の部品に含め、キーボードの移動でも選択されない
    violation: 選べない日程は選択の状態にならない
    source: [FR3.3, FR3.4, NFR5]

  # ---------- BR3 コース一覧の集計と並び ----------
  - id: BR3.1
    statement: 一覧の「直近の開催日」は、今後の日程（満席も含む）のうち最も早い開催日とする
    category: calculation
    applies_to: [Course, Session]
    trigger: コース一覧の集計
    logic: >
      今後の日程 = 現在時刻 < startsAt の日程。直近の開催日 = 今後の日程の startsAt の最小値。
      今後の日程がなければ「今後の開催予定なし」と表示する
    violation: 該当なし（計算のルール）
    source: [FR2.1, "Q4"]
  - id: BR3.2
    statement: 一覧の「空席のある日程の数」は、今後の日程のうち残席が 1 以上の数とする
    category: calculation
    applies_to: [Course, Session]
    trigger: コース一覧の集計
    logic: 空席のある日程の数 = 今後の日程のうち BR2.1 の残席数 ≥ 1 のものの数
    violation: 該当なし（計算のルール）
    source: [FR2.1]
  - id: BR3.3
    statement: U1 の一覧は、直近の開催日の早い順に並べ、同じ日はコース ID の昇順、今後の日程がないコースは最後に置く
    category: calculation
    applies_to: [Course]
    trigger: コース一覧の取得（探索条件なし）
    logic: >
      直近の開催日（BR3.1）の昇順。同値はコース ID の昇順。
      今後の日程がないコースは最後に、コース ID の昇順で並べる
    violation: 該当なし（並びのルール。表示順は毎回同じになる）
    source: [FR2.1, FR2.6]
  - id: BR3.4
    statement: 一覧には表示しているコースの件数を表示する
    category: calculation
    applies_to: [WebUi]
    trigger: コース一覧の表示
    logic: 件数 = 一覧の応答に含まれるコースの数
    violation: 該当なし（表示のルール）
    source: [FR2.7]

  # ---------- BR4 研修予算 ----------
  - id: BR4.1
    statement: 残高は従業員ごとの「付与の合計 − 消費の合計」で計算する
    category: calculation
    applies_to: [BudgetEntry]
    trigger: 残高の表示、購入確認、購入の決定
    logic: 残高 = その従業員の grant の points の合計 − consume の points の合計
    violation: 該当なし（計算のルール）
    source: [FR4.1]
  - id: BR4.2
    statement: 残高が受講料以上のときだけ支払える。同額なら支払えて購入後の残高は 0
    category: validation
    applies_to: [BudgetEntry, Course]
    trigger: 購入確認、購入の決定
    logic: IF 残高 ≥ 受講料 THEN 支払える。ELSE 支払えない（理由 insufficient-balance）
    violation: 購入を拒否する。画面では確定ボタンを無効にする
    source: [FR4.3, FR4.4]
  - id: BR4.3
    statement: 支払えないときは不足額を「受講料 − 残高」で示す
    category: calculation
    applies_to: [WebUi, TrainingBudget]
    trigger: 購入確認の表示
    logic: IF 残高 < 受講料 THEN 不足額 = 受講料 − 残高 を表示し、確定ボタンを無効にする
    violation: 該当なし（表示のルール）
    source: [FR4.3]
  - id: BR4.4
    statement: 購入後の残高は「残高 − 受講料」で示す
    category: calculation
    applies_to: [WebUi, TrainingBudget]
    trigger: 購入確認の表示
    logic: IF 残高 ≥ 受講料 THEN 購入後の残高 = 残高 − 受講料 を表示する
    violation: 該当なし（表示のルール）
    source: [FR4.2, FR5.1]
  - id: BR4.5
    statement: 消費の取引は受講料と同額で、チケットを指し、残高を負にしない
    category: constraint
    applies_to: [BudgetEntry]
    trigger: 購入の決定
    logic: 消費の取引 = { kind consume, points 受講料, ticketId 発行するチケット }。BR4.2 を満たすときだけ作る
    violation: 作らない（購入全体を拒否する）
    source: [FR4.2, NFR2]

  # ---------- BR5 購入とチケット ----------
  - id: BR5.1
    statement: 購入確定時の最終チェックは決められた順に調べ、最初に当てはまった理由 1 つだけを返す
    category: validation
    applies_to: [Purchasing]
    trigger: 購入の確定（決定関数）
    logic: >
      1. 従業員が存在しない → employee-not-found
      2. 所属企業が契約中でない → company-not-active
      3. 日程が存在しない → session-not-found
      4. 受付終了（BR2.2）→ closed
      5. その従業員が購入済み（BR5.5）→ already-purchased
      6. 満席（残席数 0）→ full
      7. 残高不足（BR4.2）→ insufficient-balance
      どれにも当たらなければ承認する
    violation: 拒否の理由 1 つを返し、何も生成しない
    source: [FR5.2, FR5.7, "Q1"]
  - id: BR5.2
    statement: 受講料はコースのデータから決め、要求に含まれる金額は使わない
    category: policy
    applies_to: [Purchasing, WebApi]
    trigger: 購入の確定
    logic: 支払うポイント = 対象の日程のコースの feePoints。要求に金額があっても読み取らない
    violation: 該当なし（要求の金額は無視する）
    source: [FR5.5, NFR3]
  - id: BR5.3
    statement: 承認した購入では、チケット 1 枚と消費の取引 1 件を 1 回の記録で同時に追加する
    category: constraint
    applies_to: [Ticket, BudgetEntry, InMemoryStore]
    trigger: 購入の決定が承認のとき
    logic: >
      決定（チケット＋消費の取引）を 1 回の状態更新で反映する。
      残席は発行済み数から、残高は取引から計算されるため、反映後に両方が同時に変わる
    violation: 一部だけが反映された状態を残さない。拒否のときは何も反映しない
    source: [FR5.3, NFR2]
  - id: BR5.4
    statement: 購入の記録は直列に行い、記録の直前に最新の状態で決定をやり直す
    category: constraint
    applies_to: [InMemoryStore, Purchasing]
    trigger: 購入の記録
    logic: >
      状態の更新は 1 か所で 1 件ずつ順に適用する。各更新では最新の状態から写し（PurchaseSnapshot）を作り、
      決定関数（BR5.1）を評価し直して、承認なら BR5.3 で反映する
    violation: 最後の 1 席に同時に 2 件届いたら、先に適用した 1 件だけが承認され、もう 1 件は full で拒否される
    source: [FR5.8, NFR2, "要件 OQ3"]
  - id: BR5.5
    statement: 同じ従業員が同じ日程を購入できるのは 1 枚まで
    category: constraint
    applies_to: [Ticket]
    trigger: 購入の確定
    logic: IF その従業員のチケットに同じ sessionId がある THEN already-purchased
    violation: 購入を拒否し、残高と残席を変えない
    source: [FR5.4, FR5.6, "要件 A2"]
  - id: BR5.6
    statement: チケット番号は推測されにくいランダムな英数字 8 文字を 4 文字ずつハイフンで区切り、ほかのチケットと重複させない
    category: calculation
    applies_to: [Ticket]
    trigger: 購入の決定が承認のとき
    logic: >
      使う文字は、紛らわしい文字（0・O・1・I・L）を除いた英大文字と数字。
      暗号論的に安全な乱数で 8 文字を選び、「XXXX-XXXX」の形にする。
      発行済みの番号と重なったら作り直す（乱数は決定関数の外から関数として渡す）
    violation: 重複した番号は発行しない
    source: [FR6.2, "Q6"]
  - id: BR5.7
    statement: 購入日時は Web API の時計で決める
    category: policy
    applies_to: [Ticket, BudgetEntry]
    trigger: 購入の決定が承認のとき
    logic: Ticket.purchasedAt と BudgetEntry.occurredAt = 記録時の現在時刻（Web API の時計）
    violation: 画面から送られた時刻は使わない
    source: [FR6.1, "要件 A6"]
  - id: BR5.8
    statement: 購入の拒否は理由を記録（ログ）に残し、個人の連絡先は出さない
    category: policy
    applies_to: [WebApi]
    trigger: 購入の拒否
    logic: 拒否のたびに、従業員 ID・日程 ID・拒否の理由をログに出す
    violation: 該当なし（記録のルール）
    source: [NFR10]
  - id: BR5.9
    statement: e チケットには決められた項目を表示する
    category: policy
    applies_to: [Ticket, WebUi]
    trigger: 購入の完了画面、チケット 1 枚の詳細
    logic: チケット番号・従業員名・コース名・開催日時・会場または形式・購入日時・支払ったポイントを表示する
    violation: 該当なし（表示のルール）
    source: [FR6.1]
  - id: BR5.10
    statement: チケット 1 枚の詳細は、そのチケットを購入した従業員にだけ返す
    category: authorization
    applies_to: [WebApi, Ticket]
    trigger: チケット詳細の取得
    logic: IF チケットの employeeId ≠ 操作中の従業員 ID THEN 存在しないものとして扱う（404）
    violation: 他人のチケットの内容を返さない
    source: [NFR3]

  # ---------- BR6 操作中の従業員と切り替え ----------
  - id: BR6.1
    statement: 操作中の従業員 ID は Cookie で受け渡し、Cookie は従業員の切り替えのときに Web API が発行する
    category: authorization
    applies_to: [WebApi, WebUi]
    trigger: 画面からのすべての要求、従業員の切り替え
    logic: >
      Web API は切り替えの要求で、指定された従業員が切り替え用の一覧にいることを確かめてから、
      従業員 ID を入れた Cookie（画面のスクリプトから読めない設定、同じサイト内でだけ送る設定）を発行する。
      以後の要求では Cookie の従業員 ID を認証済みの従業員として扱う（ローカル検証専用。ADR-008）
    violation: 一覧にない従業員への切り替えは 4xx で拒否し、Cookie を変えない
    source: [FR1.1, FR1.2, "Q2"]
  - id: BR6.2
    statement: Cookie がないときは初期データの先頭の従業員を操作中とし、切り替えた従業員は再読み込みしても保たれる
    category: policy
    applies_to: [WebApi, WebUi]
    trigger: 画面の起動（操作中の従業員の取得）
    logic: >
      IF 操作中の従業員の取得で Cookie がない THEN 初期データの先頭の従業員（高橋 遥）の Cookie を発行してその従業員を返す。
      Cookie があればその従業員を返す。Cookie はブラウザに保存されるため、再読み込みしても同じ従業員になる。
      Cookie の従業員 ID に当たる従業員がいない場合は、閲覧には応え、購入は BR1.1 で拒否する
    violation: 該当なし
    source: [FR1.1, "Q3"]
  - id: BR6.3
    statement: Cookie を使う書き込みの要求には、なりすまし送信（CSRF）の対策をかける
    category: authorization
    applies_to: [WebApi]
    trigger: 購入と従業員の切り替えの要求
    logic: >
      書き込みの要求は、送信元（Origin）が画面のオリジンであり、かつ画面だけが付ける専用のヘッダーがある場合だけ受け付ける。
      資格情報付きの通信を許可するのは画面のオリジンだけにする
    violation: 条件を満たさない要求は 403 で拒否し、何も変えない
    source: [NFR3, "Q2"]
  - id: BR6.4
    statement: 従業員を切り替えたら、どの画面からでもコース一覧へ戻り、前の従業員の情報を画面に残さない
    category: policy
    applies_to: [WebUi]
    trigger: ヘッダーでの従業員の切り替えの成功
    logic: >
      切り替えが成功したら、選んでいた日程・購入確認・購入の結果・表示中のチケットを捨て、
      コース一覧へ移動して、新しい従業員で一覧と操作中の従業員を読み込み直す
    violation: 該当なし（画面の遷移のルール）
    source: [FR1.2, "Q5", "Q5-1"]

  # ---------- BR7 画面の表示と操作 ----------
  - id: BR7.1
    statement: 読み込み中・0 件・読み込み失敗は別の表示にする
    category: policy
    applies_to: [WebUi]
    trigger: コース一覧・コースの詳細の読み込み
    logic: >
      読み込み中は読み込み中の表示、応答が 0 件なら 0 件の表示、失敗なら失敗の表示と「再試行」を出す。
      3 つは文字が異なり、見分けられる
    violation: 該当なし（表示のルール）
    source: [FR2.9, NFR4]
  - id: BR7.2
    statement: 購入確認へ進む操作は、購入可の日程を選んだときだけ有効にする
    category: validation
    applies_to: [WebUi]
    trigger: コースの詳細での日程の選択
    logic: >
      IF 購入可の日程を選んでいる THEN 選んだ日程が分かる表示にし、「購入確認へ進む」を有効にする。
      ELSE 無効にし、「日程を選んでください」と案内する
    violation: 日程を選ばずに購入確認へは進めない
    source: [FR3.3]
  - id: BR7.3
    statement: 確定の処理中は確定ボタンを無効にし、処理中であることを表示する
    category: validation
    applies_to: [WebUi]
    trigger: 購入の確定
    logic: 確定を送ってから応答を受けるまで、確定ボタンを無効にし「購入を処理しています」と表示する。処理中の再送は行わない
    violation: 連打しても要求は 1 回だけ送られる（Web API 側でも BR5.5 で重複を拒否する）
    source: [FR5.6]
  - id: BR7.4
    statement: 購入の拒否では理由ごとに決めたメッセージと次の操作を示す
    category: policy
    applies_to: [WebUi]
    trigger: 購入の拒否の応答
    logic: >
      full →「この日程は満席になりました」＋「別の日程を選ぶ」。
      closed →「この日程は受付を終了しました」＋「別の日程を選ぶ」。
      already-purchased →「この日程はすでに購入済みです」＋「別の日程を選ぶ」。
      insufficient-balance →「研修予算の残高が不足しています」＋不足額＋「別の日程を選ぶ」。
      company-not-active / employee-not-found →「ご所属の企業は現在このサービスをご利用いただけません」＋「コース一覧へ戻る」。
      session-not-found →「この日程は見つかりません」＋「コース一覧へ戻る」。
      どの場合も残高の表示は変えない
    violation: 該当なし（表示のルール）
    source: [FR5.7]
  - id: BR7.5
    statement: 「別の日程を選ぶ」はコースの詳細を読み込み直し、最新の残席を表示する
    category: policy
    applies_to: [WebUi]
    trigger: 拒否の表示での「別の日程を選ぶ」
    logic: 同じコースの詳細へ移動し、日程と残席を Web API から取得し直す（前の表示を使い回さない）
    violation: 該当なし
    source: [FR5.7]
  - id: BR7.6
    statement: 確定時に通信できなかったときは「もう一度試す」と、購入できたかの確かめ方を示す
    category: policy
    applies_to: [WebUi]
    trigger: 購入の確定で応答が得られない（通信の失敗）
    logic: >
      「通信に失敗しました」と「もう一度試す」を表示し、購入できたかどうかはコースの詳細の日程の表示（「購入済み」かどうか）で確かめられると案内する。
      もう一度試して前回の購入が成功していた場合は already-purchased が返り、BR7.4 で表示する
    violation: 該当なし
    source: [FR5.7]
  - id: BR7.7
    statement: 購入の成功後は、ヘッダーの残高を購入後の値に更新する
    category: policy
    applies_to: [WebUi]
    trigger: 購入の成功
    logic: 購入の応答に含まれる購入後の残高でヘッダーを更新し、以後どの画面でもその値を表示する
    violation: 該当なし
    source: [FR4.1]
  - id: BR7.8
    statement: 購入確認から「日程を選び直す」でコースの詳細へ戻れ、何も購入されない
    category: policy
    applies_to: [WebUi]
    trigger: 購入確認での「日程を選び直す」
    logic: 要求を送らずにコースの詳細へ戻る
    violation: 該当なし
    source: [FR5.1]
  - id: BR7.9
    statement: 存在しないコースを開いたときは「コースが見つかりません」と一覧へ戻る操作を示す
    category: policy
    applies_to: [WebUi, WebApi]
    trigger: コースの詳細の取得で 404
    logic: Web API は存在しないコース ID に 404 を返し、画面は「コースが見つかりません」と「コース一覧へ戻る」を表示する
    violation: 該当なし
    source: [FR3.5]
  - id: BR7.10
    statement: 日時は日本時間で、ポイントは 3 桁区切りに「pt」を付けて表示する
    category: policy
    applies_to: [WebUi]
    trigger: 日時とポイントの表示
    logic: 日時は Asia/Tokyo に変換して「2026年11月5日（木）13:00〜17:00」の形、ポイントは「60,000 pt」の形にする
    violation: 該当なし（表示のルール）
    source: [FR4.1, "要件 A6"]
  - id: BR7.11
    statement: ヘッダーには、どの画面でも操作中の従業員名と残高を表示する
    category: policy
    applies_to: [WebUi]
    trigger: すべての画面の表示
    logic: ヘッダーに操作中の従業員の表示名・所属企業名・残高（BR4.1）と、従業員の切り替えの部品を置く
    violation: 該当なし
    source: [FR1.2, FR4.1]
```

## ルールの要約

| ID | ルール | 種別 | 出典 |
| --- | --- | --- | --- |
| BR1.1 | 契約中の企業の従業員だけが購入できる | authorization | FR1.3, FR5.2 |
| BR1.2 | 閲覧は契約状態にかかわらず許可 | policy | FR1.3 |
| BR1.3 | 契約終了企業の従業員に利用できない旨を案内 | policy | FR1.3 |
| BR1.4 | 利用資格がなければ確定操作を無効 | validation | FR1.3 |
| BR2.1 | 残席数 = 定員 − 発行済み数（0 未満にしない） | calculation | FR3.2, NFR2 |
| BR2.2 | 開催開始を過ぎたら受付終了 | constraint | FR3.3 |
| BR2.3 | 受付終了 → 満席 → 購入可の順に判定 | calculation | FR3.3 |
| BR2.4 | 購入済みの日程は「購入済み」を優先表示 | calculation | FR3.4 |
| BR2.5 | 購入可の日程だけ選択でき、理由は文字で示す | validation | FR3.3, NFR5 |
| BR3.1 | 直近の開催日は満席も含めた今後の最も早い日 | calculation | FR2.1, Q4 |
| BR3.2 | 空席のある日程の数 | calculation | FR2.1 |
| BR3.3 | 直近の開催日順、同値はコース ID 順、予定なしは最後 | calculation | FR2.6 |
| BR3.4 | 表示中の件数を表示 | calculation | FR2.7 |
| BR4.1 | 残高 = 付与の合計 − 消費の合計 | calculation | FR4.1 |
| BR4.2 | 残高 ≥ 受講料なら支払える（同額で 0） | validation | FR4.3, FR4.4 |
| BR4.3 | 不足額 = 受講料 − 残高 | calculation | FR4.3 |
| BR4.4 | 購入後の残高 = 残高 − 受講料 | calculation | FR4.2 |
| BR4.5 | 消費取引は受講料と同額でチケットを指す | constraint | FR4.2 |
| BR5.1 | 最終チェックの順と、理由を 1 つだけ返すこと | validation | FR5.2, Q1 |
| BR5.2 | 受講料はコースのデータから決める | policy | FR5.5 |
| BR5.3 | チケットと消費取引を 1 回で同時に記録 | constraint | FR5.3 |
| BR5.4 | 記録は直列、直前に決定をやり直す | constraint | FR5.8 |
| BR5.5 | 同じ日程は 1 人 1 枚まで | constraint | FR5.4 |
| BR5.6 | チケット番号はランダム英数字 8 文字・重複なし | calculation | FR6.2, Q6 |
| BR5.7 | 購入日時は Web API の時計 | policy | FR6.1 |
| BR5.8 | 拒否の理由をログに残す | policy | NFR10 |
| BR5.9 | e チケットの表示項目 | policy | FR6.1 |
| BR5.10 | チケット詳細は本人にだけ返す | authorization | NFR3 |
| BR6.1 | 従業員 ID は Cookie、切り替えで Web API が発行 | authorization | FR1.2, Q2 |
| BR6.2 | Cookie がなければ先頭の従業員、再読み込みで保持 | policy | FR1.1, Q3 |
| BR6.3 | 書き込みに CSRF 対策 | authorization | NFR3, Q2 |
| BR6.4 | 切り替えたらどの画面からも一覧へ戻す | policy | FR1.2, Q5, Q5-1 |
| BR7.1 | 読み込み中・0 件・失敗を区別 | policy | FR2.9 |
| BR7.2 | 購入可の日程を選んだときだけ購入確認へ進める | validation | FR3.3 |
| BR7.3 | 確定の処理中はボタン無効・処理中表示 | validation | FR5.6 |
| BR7.4 | 拒否の理由ごとのメッセージと次の操作 | policy | FR5.7 |
| BR7.5 | 「別の日程を選ぶ」で最新の残席を読み直す | policy | FR5.7 |
| BR7.6 | 通信失敗時の「もう一度試す」と確かめ方の案内 | policy | FR5.7 |
| BR7.7 | 購入成功後にヘッダーの残高を更新 | policy | FR4.1 |
| BR7.8 | 「日程を選び直す」で購入せずに戻る | policy | FR5.1 |
| BR7.9 | 存在しないコースの表示 | policy | FR3.5 |
| BR7.10 | 日時は日本時間、ポイントは 3 桁区切り | policy | FR4.1, A6 |
| BR7.11 | ヘッダーに従業員名と残高を常に表示 | policy | FR1.2, FR4.1 |

## 参照した資料

- 要件定義書: `../../../inception/requirements-analysis/requirements.md`（FR・NFR・前提 A の番号）
- ユニット定義: `../../../inception/units-generation/unit-of-work.md`
- ストーリーとユニットの対応: `../../../inception/units-generation/unit-of-work-story-map.md`
- コンポーネント設計: `../../../inception/domain-design/components.md`（ADR は同じ場所の `decisions.md`）
- 機能設計の確認事項: `functional-design-questions.md`（Q1〜Q7、Q5-1）
