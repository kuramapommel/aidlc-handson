# 業務ルール（U2 コース探索）

U2（`u2-course-search`）の業務ルールを定める。下の YAML がルールの正本で、`functional-spec.md` のルールの要約はここから作った派生の表である。U1 のルール（BR1〜BR7。`../../u1-purchase-skeleton/functional-design/rules.md`）と番号が重ならないよう、U2 は BR8 から始める。

グループ:

- BR8 探索条件の検証（CourseCatalog・WebApi）
- BR9 絞り込み（CourseCatalog）
- BR10 並べ替えと件数（CourseCatalog）
- BR11 画面の探索の操作（WebUi）

`source` には要件（FR・NFR）と、機能設計の確認事項（`functional-design-questions.md` の Q 番号）を書く。

```yaml
rules:
  # ---------- BR8 探索条件の検証 ----------
  - id: BR8.1
    statement: キーワードは自由入力として拒否せず、正規化して空白区切りの検索語にする
    category: validation
    applies_to: [SearchCondition, NormalizedKeyword]
    trigger: 探索の要求
    logic: >
      どんな文字列も正しい値として受け付ける。先頭 200 文字だけを使う（それより後は捨てる。拒否しない）。
      全角英数字・記号を半角に、英字を小文字に、カタカナと半角カタカナをひらがなにそろえる。
      半角・全角の空白で区切り、空の語を捨てる。語が 0 個なら「キーワードの条件なし」として扱う
    violation: 該当なし（キーワードで 400 にはしない）
    source: [FR2.2, "Q2-1", "Q5"]
  - id: BR8.2
    statement: カテゴリは決められた 4 つのどれかか、指定なしだけを受け付ける
    category: validation
    applies_to: [SearchCondition]
    trigger: 探索の要求
    logic: IF カテゴリが指定され、management・technology・business-skills・compliance のどれでもない THEN 不正
    violation: Web API は 400 と不正な条件（category）を返す
    source: [FR2.3, NFR3, "Q2"]
  - id: BR8.3
    statement: 空席ありのみは「はい」か「いいえ（指定なし）」だけを受け付ける
    category: validation
    applies_to: [SearchCondition]
    trigger: 探索の要求
    logic: IF 空席ありのみが true・false 以外の値 THEN 不正
    violation: Web API は 400 と不正な条件（availableOnly）を返す
    source: [FR2.4, NFR3, "Q2"]
  - id: BR8.4
    statement: 期間の開始日・終了日は暦日の形で、終了日は開始日以降でなければならない
    category: validation
    applies_to: [SearchCondition]
    trigger: 探索の要求
    logic: >
      開始日・終了日は YYYY-MM-DD の実在する暦日（日本時間）。形が違えば不正。
      両方あり終了日 < 開始日なら不正（period）
    violation: Web API は 400 と不正な条件（periodFrom・periodTo・period）を返す
    source: [FR2.5, NFR3, "Q2"]
  - id: BR8.5
    statement: 並べ替えは決められた 4 つのどれかだけを受け付け、指定なしは開催日が近い順にする
    category: validation
    applies_to: [SearchCondition]
    trigger: 探索の要求
    logic: IF 並べ替えが date・fee-asc・fee-desc・name のどれでもない THEN 不正。指定なしは date
    violation: Web API は 400 と不正な条件（sort）を返す
    source: [FR2.6, NFR3, "Q2"]
  - id: BR8.6
    statement: 不正な条件があるときは、不正な条件をすべて挙げて 400 を返し、結果は返さない
    category: validation
    applies_to: [WebApi]
    trigger: 探索の要求の検証
    logic: BR8.2〜BR8.5 で不正となった条件を、項目名と理由の組で全部返す
    violation: 該当なし（応答の形のルール）
    source: [NFR3, "Q2"]

  # ---------- BR9 絞り込み ----------
  - id: BR9.1
    statement: キーワードは、コース名または概要にすべての検索語を含むコースに絞り込む
    category: calculation
    applies_to: [Course]
    trigger: キーワードの条件があるとき
    logic: >
      コース名と概要を BR8.1 と同じ規則で正規化し、どちらかに各検索語が含まれるかを語ごとに調べる。
      すべての語が（コース名か概要のどちらかに）含まれるコースだけを残す（AND）
    violation: 該当なし
    source: [FR2.2]
  - id: BR9.2
    statement: カテゴリは、指定したカテゴリのコースだけに絞り込む
    category: calculation
    applies_to: [Course]
    trigger: カテゴリの条件があるとき
    logic: IF コースのカテゴリ = 指定したカテゴリ THEN 残す
    violation: 該当なし
    source: [FR2.3]
  - id: BR9.3
    statement: 空席ありのみは、空席のある今後の日程を 1 つ以上持つコースだけに絞り込む
    category: calculation
    applies_to: [Course, Session]
    trigger: 空席ありのみが true のとき
    logic: 今後の日程（現在時刻 < 開催開始）のうち、残席数（U1 の BR2.1）≥ 1 のものが 1 つ以上あるコースだけを残す。受付終了の日程の空席は数えない
    violation: 該当なし
    source: [FR2.4]
  - id: BR9.4
    statement: 期間は、その期間に開催開始日がある空席のある今後の日程を 1 つ以上持つコースだけに絞り込む
    category: calculation
    applies_to: [Course, Session]
    trigger: 開始日か終了日の条件があるとき
    logic: >
      日程の開催開始日（日本時間の暦日）が、開始日以上（開始日があるとき）かつ終了日以下（終了日があるとき）で、
      今後の日程で、残席数 ≥ 1 のものが 1 つ以上あるコースだけを残す。両端の日を含む
    violation: 該当なし
    source: [FR2.5]
  - id: BR9.5
    statement: 複数の条件はすべてを満たすコースだけを残す
    category: calculation
    applies_to: [SearchCondition]
    trigger: 探索の要求
    logic: BR9.1〜BR9.4 のうち、指定された条件をすべて満たすコースだけを残す（AND）
    violation: 該当なし
    source: [FR2.2, FR2.3, FR2.4, FR2.5]

  # ---------- BR10 並べ替えと件数 ----------
  - id: BR10.1
    statement: 開催日が近い順は、期間の条件に関係なく、今後の日程（満席も含む）の最も早い開催日で並べる
    category: calculation
    applies_to: [Course]
    trigger: 並べ替えが date（または指定なし）
    logic: >
      U1 の BR3.1 の直近の開催日（満席も含む今後の日程の最も早い日）の昇順。期間で絞り込んでいても、期間の外の日程を含めて数える。
      同じ日はコース ID の昇順。今後の日程がないコースは最後に、コース ID の昇順
    violation: 該当なし
    source: [FR2.6, "Q3"]
  - id: BR10.2
    statement: 受講料の安い順は受講料の昇順で、同じ受講料はコース ID の昇順にする
    category: calculation
    applies_to: [Course]
    trigger: 並べ替えが fee-asc
    logic: feePoints の昇順、同値はコース ID の昇順
    violation: 該当なし
    source: [FR2.6]
  - id: BR10.3
    statement: 受講料の高い順は受講料の降順で、同じ受講料はコース ID の昇順にする
    category: calculation
    applies_to: [Course]
    trigger: 並べ替えが fee-desc
    logic: feePoints の降順、同値はコース ID の昇順
    violation: 該当なし
    source: [FR2.6]
  - id: BR10.4
    statement: コース名順は、よみがなの五十音順で、同じよみがなはコース ID の昇順にする
    category: calculation
    applies_to: [Course]
    trigger: 並べ替えが name
    logic: titleReading を BR8.1 の規則で正規化したひらがなの並びを、文字ごとに五十音順（ひらがなの符号の順）で比べる。長音符・濁点の違いも符号の順で決める。同値はコース ID の昇順
    violation: 該当なし
    source: [FR2.6, "Q4"]
  - id: BR10.5
    statement: 件数は、条件に合うコースの数とする
    category: calculation
    applies_to: [WebApi]
    trigger: 探索の応答
    logic: 件数 = 絞り込み後のコースの数。応答に一覧と一緒に含める
    violation: 該当なし
    source: [FR2.7]

  # ---------- BR11 画面の探索の操作 ----------
  - id: BR11.1
    statement: キーワードは入力が止まってから 150 ms 待って探索を送り、ほかの条件は変えたらすぐに送る
    category: policy
    applies_to: [WebUi]
    trigger: 探索条件の入力
    logic: >
      キーワードは、最後の入力から 150 ms 入力がなければ送る（待っている間に入力があれば待ち直す）。
      カテゴリ・空席ありのみ・期間・並べ替えは、変えたらすぐに送る。どちらも全画面を読み込み直さない
    violation: 該当なし
    source: [FR2.2, NFR1, "Q1"]
  - id: BR11.2
    statement: 探索の応答は最後に送った要求のものだけを反映する
    category: policy
    applies_to: [WebUi]
    trigger: 探索の応答の受信
    logic: 要求ごとに番号を付け、受け取った応答の番号が最後に送った番号と同じときだけ一覧に反映する。古い応答は捨てる
    violation: 該当なし
    source: [NFR1, "ADR-006"]
  - id: BR11.3
    statement: 探索条件は URL のクエリに反映し、既定の値は書かない
    category: policy
    applies_to: [WebUi]
    trigger: 探索条件の変更
    logic: >
      q（キーワード）・category・available（空席ありのみ。true のときだけ）・from・to・sort（date 以外のときだけ）を URL に書く。
      条件を変えるたびに、ブラウザの履歴を増やさずに今の URL を書き換える（戻る操作で条件の変更をひとつずつさかのぼらない）
    violation: 該当なし
    source: [FR2.8]
  - id: BR11.4
    statement: URL から条件を復元し、不正な値の項目だけを無視する
    category: validation
    applies_to: [WebUi]
    trigger: URL を開いたとき・再読み込みしたとき
    logic: >
      URL の各項目を BR8.2〜BR8.5 と同じ規則で項目ごとに確かめ、正しい項目だけを入力欄と条件に復元する。
      不正な項目は無視し（エラー画面にしない）、URL からも取り除く。キーワードはそのまま復元する
    violation: 該当なし（不正な項目は無視する）
    source: [FR2.8, "Q2-1"]
  - id: BR11.5
    statement: 形の決まった条件は画面で確かめてから送り、期間の前後が逆なら送らずに案内する
    category: validation
    applies_to: [WebUi]
    trigger: 探索条件の入力
    logic: >
      カテゴリ・並べ替えは選択肢から選ぶ。日付はブラウザの日付の入力欄で、空か正しい日付だけになる。
      終了日 < 開始日のときは探索を送らず、期間の入力欄の近くに「期間の指定が正しくありません（終了日を開始日以降にしてください）」と出し、一覧はそれまでの結果のままにする。
      前後が正しくなったら案内を消し、探索を送る
    violation: 送らない（一覧はそれまでのまま）
    source: [FR2.5, "Q2-1"]
  - id: BR11.6
    statement: 0 件のときは、条件に合うコースがないことと「条件をリセット」を表示し、リセットで条件をすべて既定に戻す
    category: policy
    applies_to: [WebUi]
    trigger: 探索の応答が 0 件
    logic: >
      「条件に合うコースがありません」と「条件をリセット」を表示する。読み込み中・読み込み失敗とは別の表示にする。
      「条件をリセット」はキーワード・カテゴリ・空席ありのみ・期間・並べ替えをすべて既定に戻し、URL のクエリも消して、全コースを探索し直す
    violation: 該当なし
    source: [FR2.9]
  - id: BR11.7
    statement: 読み込みに失敗したときは、失敗したことと「再試行」を表示し、再試行は今の条件で送り直す
    category: policy
    applies_to: [WebUi]
    trigger: 探索の要求の通信の失敗・5xx
    logic: 「コースを読み込めませんでした」と「再試行」を表示する。0 件の表示とは別にする。「再試行」は今の条件で探索を送り直す
    violation: 該当なし
    source: [FR2.9]
  - id: BR11.8
    statement: 詳細から一覧へ戻ったとき、指定していた条件と結果を保つ
    category: policy
    applies_to: [WebUi]
    trigger: コースの詳細から一覧への移動（ブラウザの戻る操作・「コース一覧へ戻る」）
    logic: 一覧へ移るときに、条件を URL に持たせて渡す。戻った一覧は URL の条件を復元し（BR11.4）、同じ条件で探索して同じ結果を表示する
    violation: 該当なし
    source: [FR2.8, "AC1.8.4"]
  - id: BR11.9
    statement: 万一 Web API が 400 を返したときは、一覧をそれまでのままにして条件を確かめるよう案内する
    category: policy
    applies_to: [WebUi]
    trigger: 探索の応答が 400
    logic: 一覧はそれまでの結果のままにし、「条件を確かめてください」と、応答が挙げた不正な条件を表示する
    violation: 該当なし
    source: [NFR3, "Q2"]
  - id: BR11.10
    statement: カテゴリには「すべて」を、並べ替えには 4 つの並べ方を選択肢として出す
    category: policy
    applies_to: [WebUi]
    trigger: 一覧の探索条件の表示
    logic: カテゴリは「すべて」・マネジメント・技術・ビジネススキル・コンプライアンス。並べ替えは開催日が近い順（既定）・受講料の安い順・受講料の高い順・コース名順。どの入力にもラベルを付ける
    violation: 該当なし
    source: [FR2.3, FR2.6, NFR5]
```

## ルールの要約

| ID | ルール | 種別 | 出典 |
| --- | --- | --- | --- |
| BR8.1 | キーワードは拒否せず正規化（200 文字・全角半角・大小・かな） | validation | FR2.2, Q2-1, Q5 |
| BR8.2 | カテゴリは 4 つか指定なし | validation | FR2.3, Q2 |
| BR8.3 | 空席ありのみは true・false | validation | FR2.4, Q2 |
| BR8.4 | 期間は暦日の形で、終了日 ≥ 開始日 | validation | FR2.5, Q2 |
| BR8.5 | 並べ替えは 4 つ、指定なしは開催日が近い順 | validation | FR2.6, Q2 |
| BR8.6 | 不正な条件を全部挙げて 400 | validation | NFR3, Q2 |
| BR9.1 | キーワードはコース名か概要にすべての語 | calculation | FR2.2 |
| BR9.2 | カテゴリの一致 | calculation | FR2.3 |
| BR9.3 | 空席のある今後の日程が 1 つ以上 | calculation | FR2.4 |
| BR9.4 | 期間内に空席のある今後の日程が 1 つ以上（両端を含む） | calculation | FR2.5 |
| BR9.5 | 条件は AND | calculation | FR2.2〜FR2.5 |
| BR10.1 | 開催日が近い順（期間に関係なく、満席も含む） | calculation | FR2.6, Q3 |
| BR10.2 | 受講料の安い順（同値はコース ID） | calculation | FR2.6 |
| BR10.3 | 受講料の高い順（同値はコース ID） | calculation | FR2.6 |
| BR10.4 | コース名順はよみがなの五十音順 | calculation | FR2.6, Q4 |
| BR10.5 | 件数は条件に合うコースの数 | calculation | FR2.7 |
| BR11.1 | キーワードは 150 ms 待って送る、ほかはすぐ | policy | FR2.2, NFR1, Q1 |
| BR11.2 | 最後の要求の応答だけを反映 | policy | NFR1, ADR-006 |
| BR11.3 | 条件を URL に反映（既定値は書かない、履歴を増やさない） | policy | FR2.8 |
| BR11.4 | URL から復元し、不正な項目だけ無視 | validation | FR2.8 |
| BR11.5 | 形の決まった条件は画面で確かめてから送る | validation | FR2.5, Q2-1 |
| BR11.6 | 0 件の表示と「条件をリセット」 | policy | FR2.9 |
| BR11.7 | 読み込み失敗の表示と「再試行」 | policy | FR2.9 |
| BR11.8 | 詳細から戻っても条件と結果を保つ | policy | FR2.8 |
| BR11.9 | 400 のときは一覧を保ち案内 | policy | NFR3, Q2 |
| BR11.10 | カテゴリと並べ替えの選択肢 | policy | FR2.3, FR2.6, NFR5 |

## 参照した資料

- 要件定義書: `../../../inception/requirements-analysis/requirements.md`（FR2.2〜FR2.9、NFR1・NFR3・NFR5）
- ユニット定義: `../../../inception/units-generation/unit-of-work.md`
- ストーリーとユニットの対応: `../../../inception/units-generation/unit-of-work-story-map.md`
- コンポーネント設計: `../../../inception/domain-design/components.md`（ADR は同じ場所の `decisions.md`）
- U1 の業務ルール: `../../u1-purchase-skeleton/functional-design/rules.md`
- 機能設計の確認事項: `functional-design-questions.md`（Q1〜Q5、Q2-1）
