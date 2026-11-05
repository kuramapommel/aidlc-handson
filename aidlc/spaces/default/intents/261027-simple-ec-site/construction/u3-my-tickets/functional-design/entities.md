# エンティティ（U3 購入済みチケット）

U3（`u3-my-tickets`）で扱うデータの形を定める。下の YAML がデータの形の正本で、`functional-spec.md` の ER 図はここから作った派生の図である。

- U3 は新しい保存のエンティティを作らない。U1 のエンティティ（`../../u1-purchase-skeleton/functional-design/entities.md`）の Ticket・Session・Course・Employee を読み出すだけで、属性も変えない。
- 購入済みチケットの一覧の 1 件は、保存しない値（値オブジェクト）`OwnedTicket` として定める。Web API の応答と画面で同じ形を使う。
- 「受講済み」かどうかは保存せず、一覧を作るたびに Web API の時計で導く（導出属性 `attendance`）。
- データは不変として扱う（NFR6）。

```yaml
entities: []  # U3 で追加・変更する保存のエンティティはない（U1 の Ticket・Session・Course・Employee を読み出す）

value_objects:
  - name: OwnedTicket
    owner: Purchasing
    description: >
      操作中の従業員が購入した e チケット 1 枚を、一覧で表示するためにまとめた値。
      Ticket に、日程・コース・従業員から引いた表示用の項目と、受講済みかどうかを足したもの
    attributes:
      - name: ticketId
        type: identifier
        required: true
        unique: true
        references: Ticket.ticketId
        constraints: "画面で開閉の状態を覚えるキーに使う"
      - name: ticketNumber
        type: text
        required: true
        references: Ticket.ticketNumber
        constraints: "一覧の行に表示する（確認事項 Q3）"
      - name: courseTitle
        type: text
        required: true
        references: Course.title
        constraints: "一覧の行に表示する"
      - name: startsAt
        type: datetime
        required: true
        references: Session.startsAt
        constraints: "一覧の行に表示し、並びと受講済みの判定に使う"
      - name: endsAt
        type: datetime
        required: true
        references: Session.endsAt
        constraints: "開催日時（開始〜終了）の表示に使う"
      - name: venue
        type: text
        required: false
        references: Session.venue
        constraints: "会場または形式の表示に使う（U1 と同じ表示の規則）"
      - name: deliveryFormat
        type: enum
        required: true
        references: Course.deliveryFormat
        constraints: "会場または形式の表示に使う"
      - name: employeeName
        type: text
        required: true
        references: Employee.displayName
        constraints: "開いたときに表示する"
      - name: purchasedAt
        type: datetime
        required: true
        references: Ticket.purchasedAt
        constraints: "開いたときに表示する。並びの同点の決め方にも使う"
      - name: paidPoints
        type: integer
        required: true
        min: 0
        references: Ticket.paidPoints
        constraints: "開いたときに表示する"
      - name: attendance
        type: enum
        required: true
        allowed_values: [upcoming, attended]
        constraints: "導出属性。startsAt が Web API の現在時刻以前なら attended（受講済み）、それより後なら upcoming（これから）。保存しない"
    relationships:
      - entity: Ticket
        cardinality: "1 対 1"
        direction: "OwnedTicket は 1 枚の Ticket から作る"
      - entity: Employee
        cardinality: "多 対 1"
        direction: "一覧のすべての OwnedTicket は、操作中の 1 人の従業員のもの"

  - name: OwnedTicketList
    owner: Purchasing
    description: 操作中の従業員の購入済みチケットの一覧。並びは rules.md の BR12.2 で決める
    attributes:
      - name: tickets
        type: list
        required: true
        constraints: "OwnedTicket の並び。0 件もありうる（AC4.2.2）"
    relationships:
      - entity: OwnedTicket
        cardinality: "1 対 0..多"
        direction: "一覧は OwnedTicket を並べて持つ"
```

## 要約

| 名前 | 種類 | 持ち主 | 内容 |
| --- | --- | --- | --- |
| OwnedTicket | 値オブジェクト | Purchasing | 一覧の 1 件。チケット・日程・コース・従業員の表示用の項目と、受講済みかどうか（導出） |
| OwnedTicketList | 値オブジェクト | Purchasing | 操作中の従業員の購入済みチケットの並び（0 件以上） |

保存のエンティティ（Ticket・Session・Course・Employee）は U1 のまま使い、U3 では変えない。
