# エンティティ（U1 購入の最小の流れ）

U1（`u1-purchase-skeleton`）で扱うデータの形を定める。下の YAML がデータの形の正本で、`functional-spec.md` の ER 図はここから作った派生の図である。

- 保存するエンティティは、ドメイン設計（`../../../inception/domain-design/components.md`）の 6 つ（Course・Session・Company・Employee・BudgetEntry・Ticket）だけとする。
- 残席数・残高・日程の状態は保存せず、記録から計算する（ADR-002、ADR-003）。計算で得る値は「計算で求める値（派生の値）」として最後にまとめる。
- データは不変として扱う。変更は「新しい値を作る」ことで表し、既存の値は書き換えない（NFR6）。
- 日時はすべてタイムゾーン付きの時刻として持ち、表示は日本時間（Asia/Tokyo）にする（要件 A6）。
- ポイントは整数で表す。小数は使わない（FR4.1、要件 A3）。

```yaml
entities:
  - name: Course
    owner: CourseCatalog
    description: 研修コース。受講料・所要時間・形式など、日程によらない情報を持つ。初期データとしてだけ作られ、変更されない
    attributes:
      - name: courseId
        type: identifier
        required: true
        unique: true
        constraints: "形式は C と 3 桁の数字（例: C001）。並べ替えの同値の基準にする（BR3.3）"
      - name: title
        type: text
        required: true
        constraints: "1〜80 文字"
      - name: category
        type: enum
        required: true
        allowed_values: [management, technology, business-skills, compliance]
        constraints: "表示名はそれぞれ「マネジメント」「技術」「ビジネススキル」「コンプライアンス」"
      - name: summary
        type: text
        required: true
        constraints: "一覧に出す概要。1〜120 文字"
      - name: description
        type: text
        required: true
        constraints: "詳細に出す説明。1〜2000 文字"
      - name: feePoints
        type: integer
        required: true
        min: 1
        constraints: "受講料（pt）。購入時に差し引く金額の唯一の根拠（FR5.5）"
      - name: durationMinutes
        type: integer
        required: true
        min: 1
        constraints: "所要時間（分）"
      - name: deliveryFormat
        type: enum
        required: true
        allowed_values: [online, venue]
        constraints: "表示名は「オンライン」「会場」"
    constraints:
      - "courseId はすべてのコースで重複しない"
    relationships:
      - target: Session
        cardinality: one-to-many
        direction: "Course 1 件に Session が 0 件以上属する"

  - name: Session
    owner: CourseCatalog
    description: コースの開催日程。定員だけを持ち、残席数は持たない（ADR-003）
    attributes:
      - name: sessionId
        type: identifier
        required: true
        unique: true
        constraints: "形式は S と 4 桁の数字（例: S0001）"
      - name: courseId
        type: identifier
        required: true
        references: Course.courseId
      - name: startsAt
        type: datetime
        required: true
        constraints: "開催開始の時刻。受付終了の判定に使う（BR2.2）"
      - name: endsAt
        type: datetime
        required: true
        constraints: "startsAt より後"
      - name: venue
        type: text
        required: false
        constraints: "会場名。Course.deliveryFormat が venue のときは必須、online のときは持たない"
      - name: capacity
        type: integer
        required: true
        min: 1
        constraints: "定員"
    constraints:
      - "sessionId はすべての日程で重複しない"
      - "endsAt > startsAt"
    relationships:
      - target: Course
        cardinality: many-to-one
        direction: "各 Session はちょうど 1 つの Course に属する"
      - target: Ticket
        cardinality: one-to-many
        direction: "Session 1 件に Ticket が 0〜capacity 件発行される"

  - name: Company
    owner: Eligibility
    description: 契約企業。契約状態で購入の可否が決まる
    attributes:
      - name: companyId
        type: identifier
        required: true
        unique: true
        constraints: "形式は K と 3 桁の数字（例: K001）"
      - name: name
        type: text
        required: true
        constraints: "1〜60 文字"
      - name: contractStatus
        type: enum
        required: true
        allowed_values: [active, ended]
        constraints: "active（契約中）のときだけ所属従業員が購入できる（BR1.1）"
    relationships:
      - target: Employee
        cardinality: one-to-many
        direction: "Company 1 件に Employee が 1 件以上所属する"

  - name: Employee
    owner: Eligibility
    description: 従業員。社内ポータルで認証済みとみなし、Cookie に入った従業員 ID で識別する（ADR-008、BR6.1）
    attributes:
      - name: employeeId
        type: identifier
        required: true
        unique: true
        constraints: "形式は E と 3 桁の数字（例: E001）"
      - name: displayName
        type: text
        required: true
        constraints: "1〜40 文字。ヘッダーと e チケットに表示する"
      - name: companyId
        type: identifier
        required: true
        references: Company.companyId
    relationships:
      - target: Company
        cardinality: many-to-one
        direction: "各 Employee はちょうど 1 つの Company に所属する"

  - name: BudgetEntry
    owner: TrainingBudget
    description: 研修予算の取引。追記だけで記録し、変更・削除しない（ADR-002）
    attributes:
      - name: entryId
        type: identifier
        required: true
        unique: true
      - name: employeeId
        type: identifier
        required: true
        references: Employee.employeeId
      - name: kind
        type: enum
        required: true
        allowed_values: [grant, consume]
        constraints: "grant は付与（初期データだけが作る）、consume は購入による消費"
      - name: points
        type: integer
        required: true
        min: 1
        constraints: "取引の額（正の整数）。消費でも正の値で持ち、kind で向きを表す"
      - name: occurredAt
        type: datetime
        required: true
      - name: ticketId
        type: identifier
        required: false
        references: Ticket.ticketId
        constraints: "kind が consume のときは必須、grant のときは持たない"
    constraints:
      - "consume の points は、それを生んだ Ticket の paidPoints と同じ"
      - "記録後の従業員の残高（BR4.1）が負になる consume は作らない（BR4.5）"
    relationships:
      - target: Employee
        cardinality: many-to-one
        direction: "各 BudgetEntry は 1 人の Employee のもの"
      - target: Ticket
        cardinality: one-to-one
        direction: "consume の BudgetEntry 1 件は、Ticket 1 件とちょうど対応する"

  - name: Ticket
    owner: Purchasing
    description: e チケット。1 名・1 日程につき 1 枚（FR5.4）
    attributes:
      - name: ticketId
        type: identifier
        required: true
        unique: true
        constraints: "内部の識別子。画面の URL などに使う"
      - name: ticketNumber
        type: text
        required: true
        unique: true
        constraints: "利用者に見せる番号。BR5.6 の形式（例: K7Q2-M9XD）"
      - name: employeeId
        type: identifier
        required: true
        references: Employee.employeeId
      - name: sessionId
        type: identifier
        required: true
        references: Session.sessionId
      - name: paidPoints
        type: integer
        required: true
        min: 1
        constraints: "購入時点の Course.feePoints（BR5.2）"
      - name: purchasedAt
        type: datetime
        required: true
        constraints: "Web API の時計で決める（BR5.7）"
    constraints:
      - "(employeeId, sessionId) の組はすべてのチケットで重複しない（BR5.5）"
      - "ある Session の Ticket の数は、その Session の capacity 以下（BR2.1）"
    relationships:
      - target: Employee
        cardinality: many-to-one
        direction: "各 Ticket は 1 人の Employee が購入したもの"
      - target: Session
        cardinality: many-to-one
        direction: "各 Ticket は 1 つの Session の受講権"
      - target: BudgetEntry
        cardinality: one-to-one
        direction: "各 Ticket には消費の BudgetEntry がちょうど 1 件ある"

derived_values:
  - name: remainingSeats
    owner: CourseCatalog
    definition: "Session.capacity − その日程の発行済みチケット数（0 未満にならない）"
    rule: BR2.1
  - name: sessionStatus
    owner: CourseCatalog
    allowed_values: [available, full, closed]
    definition: "受付終了（closed）→ 満席（full）→ 購入可（available）の順で判定する"
    rule: BR2.3
  - name: sessionAvailabilityForEmployee
    owner: Purchasing
    allowed_values: [available, full, closed, purchased]
    definition: "sessionStatus に、操作中の従業員が購入済みかどうか（purchased）を重ねた表示用の状態"
    rule: BR2.4
  - name: nextSessionDate
    owner: CourseCatalog
    definition: "今後の日程（満席も含む）のうち最も早い startsAt。今後の日程がなければ値なし"
    rule: BR3.1
  - name: openSessionCount
    owner: CourseCatalog
    definition: "今後の日程のうち remainingSeats が 1 以上の数"
    rule: BR3.2
  - name: balance
    owner: TrainingBudget
    definition: "従業員ごとの grant の合計 − consume の合計"
    rule: BR4.1
  - name: eligibility
    owner: Eligibility
    allowed_values: [eligible, company-not-active, employee-not-found]
    definition: "従業員 ID から判定する利用資格"
    rule: BR1.1
```

## 補助の型（保存しない値）

業務の関数の入出力として使う、保存しない値の形。詳細は `rules.md` と `functional-spec.md` で使う。

| 型 | 中身 | 使う場所 |
| --- | --- | --- |
| PurchaseRequest | 従業員 ID（Cookie から）、日程 ID。受講料などの金額は含まない（BR5.2） | Web API → Purchasing |
| PurchaseSnapshot | 対象の日程とコース、その日程の発行済み数、従業員の利用資格、残高、その従業員がその日程を購入済みか、現在時刻 | Purchasing の決定関数の入力（ADR-004） |
| PurchaseDecision | 「承認: 発行するチケット 1 枚＋消費取引 1 件」または「拒否: 理由 1 つ」 | Purchasing の決定関数の出力 |
| RejectionReason | employee-not-found、company-not-active、session-not-found、closed、already-purchased、full、insufficient-balance のいずれか 1 つ（調べる順は BR5.1） | Web API の応答、画面の表示、ログ |
| EmployeeContext | 操作中の従業員の表示名・所属企業名・契約状態・残高 | ヘッダーの表示 |
| SwitchableEmployee | 従業員 ID・表示名・企業名・契約状態 | 従業員の切り替え（FR1.2） |

## まとめ

| エンティティ | 所有 | 役割 | 書き込み |
| --- | --- | --- | --- |
| Course | CourseCatalog | 研修コース | 初期データのみ |
| Session | CourseCatalog | 開催日程と定員 | 初期データのみ |
| Company | Eligibility | 契約企業と契約状態 | 初期データのみ |
| Employee | Eligibility | 従業員 | 初期データのみ |
| BudgetEntry | TrainingBudget | 予算の付与・消費の取引 | 付与は初期データ、消費は購入の記録で追記 |
| Ticket | Purchasing | e チケット | 購入の記録で追記 |

購入で増えるのは Ticket と消費の BudgetEntry の 2 件だけで、2 件は 1 回の記録で同時に追加する（BR5.3）。それ以外のエンティティは起動時の初期データから変わらない。
