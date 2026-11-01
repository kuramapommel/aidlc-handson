# エンティティ（U2 コース探索）

U2（`u2-course-search`）で扱うデータの形を定める。下の YAML がデータの形の正本で、`functional-spec.md` の ER 図はここから作った派生の図である。

- U2 は新しい保存のエンティティを作らない。U1 のエンティティ（`../../u1-purchase-skeleton/functional-design/entities.md`）に、コース名順の並べ替えのための **よみがな**（Course.titleReading）を 1 つ足す（確認事項 Q4）。
- 探索条件は保存しない値（値オブジェクト）として定める。画面・URL・Web API で同じ形を使う。
- データは不変として扱う（NFR6）。

```yaml
entities:
  - name: Course
    owner: CourseCatalog
    description: 研修コース（U1 で定義済み）。U2 では、コース名順の並べ替えに使うよみがなを足す
    change: "U1 の Course に titleReading を追加する。ほかの属性は U1 のまま"
    attributes:
      - name: titleReading
        type: text
        required: true
        constraints: "コース名の読みを、ひらがなと長音符（ー）・英数字・空白で書く（例: 「ちーむまねじめんとにゅうもん」）。1〜120 文字。初期データのすべてのコースに付ける。英字の部分は読みに直す（例: AI → えーあい）"
    relationships: []

value_objects:
  - name: SearchCondition
    owner: CourseCatalog
    description: コースの探索条件。すべての項目は省略でき、省略は「条件なし」を表す。項目どうしは AND で組み合わせる
    attributes:
      - name: keyword
        type: text
        required: false
        constraints: "自由入力。どんな文字列も正しい値として受け付け、拒否しない（確認事項 Q2-1）。長すぎる入力は先頭 200 文字だけを探索に使う。空白（半角・全角）だけの文字列は「条件なし」と同じ"
      - name: category
        type: enum
        required: false
        allowed_values: [management, technology, business-skills, compliance]
        constraints: "省略は「すべて」"
      - name: availableOnly
        type: boolean
        required: false
        default: false
        constraints: "true のとき、空席のある今後の日程を持つコースだけにする"
      - name: periodFrom
        type: date
        required: false
        constraints: "日本時間の暦日（YYYY-MM-DD）。期間の開始日（その日を含む）"
      - name: periodTo
        type: date
        required: false
        constraints: "日本時間の暦日（YYYY-MM-DD）。期間の終了日（その日を含む）。periodFrom と両方あるときは periodFrom 以降"
      - name: sort
        type: enum
        required: false
        allowed_values: [date, fee-asc, fee-desc, name]
        default: date
        constraints: "date は開催日が近い順、fee-asc は受講料の安い順、fee-desc は受講料の高い順、name はコース名順（よみがなの五十音順）"
    constraints:
      - "periodFrom と periodTo の両方があるとき、periodTo ≥ periodFrom（BR8.4）"

  - name: NormalizedKeyword
    owner: CourseCatalog
    description: 照合のために正規化したキーワード。保存しない
    attributes:
      - name: terms
        type: list
        required: true
        constraints: "正規化した検索語の並び。空白（半角・全角）で区切り、空の語は捨てる。語が 0 個なら「条件なし」"
    constraints:
      - "正規化: 全角英数字・記号を半角に、英字を小文字に、カタカナをひらがなに、半角カタカナを全角ひらがなにそろえる（BR8.1）"

  - name: SearchConditionError
    owner: CourseCatalog
    description: 形の決まった条件が不正なときの理由。Web API が 400 で返す
    attributes:
      - name: field
        type: enum
        required: true
        allowed_values: [category, availableOnly, periodFrom, periodTo, period, sort]
      - name: reason
        type: text
        required: true
        constraints: "利用者が読める短い説明（例: 「存在しないカテゴリです」「終了日が開始日より前です」）"
```

## まとめ

| 名前 | 種類 | 所有 | 役割 |
| --- | --- | --- | --- |
| Course.titleReading | エンティティの属性の追加 | CourseCatalog | コース名順の並べ替え（よみがなの五十音順） |
| SearchCondition | 値オブジェクト | CourseCatalog | 探索条件（画面・URL・Web API で共通の形） |
| NormalizedKeyword | 値オブジェクト | CourseCatalog | 照合用に正規化したキーワード |
| SearchConditionError | 値オブジェクト | CourseCatalog | 不正な条件の理由（400 で返す） |

U1 のほかのエンティティ（Session・Company・Employee・BudgetEntry・Ticket）は変えない。残席数と日程の状態は、U1 と同じく記録から計算する。

## 参照した資料

- 要件定義書: `../../../inception/requirements-analysis/requirements.md`（FR2.2〜FR2.8）
- ユニット定義: `../../../inception/units-generation/unit-of-work.md`
- コンポーネント設計: `../../../inception/domain-design/components.md`
- U1 のエンティティ: `../../u1-purchase-skeleton/functional-design/entities.md`
- 機能設計の確認事項: `functional-design-questions.md`（Q2-1・Q4・Q5）
