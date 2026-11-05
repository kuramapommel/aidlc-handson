# 画面の部品（U3 購入済みチケット）

U3 で WebUi（MoonBit、The Elm Architecture）に足す部品・状態・メッセージを定める。U1・U2 の部品の振る舞いは変えず、足すだけにする。

## 1. 部品の構成

```text
App
├── Header（U1）
│   └── 「購入済みチケット」リンク …… 追加（BR14.7）
├── PurchaseCompletePage（U1）
│   └── 「購入済みチケットを見る」リンク …… 購入直後の表示に追加（BR14.6）
└── MyTicketsPage …… 追加
    ├── 読み込み中 / 失敗と「再試行」 / 0 件の案内（BR14.2・BR14.3）
    └── OwnedTicketItem（チケットの数だけ）
        ├── 行: コース名・開催日時・会場または形式・チケット番号・「受講済み」（BR12.4・BR14.5）
        ├── 「詳細を見る」/「詳細を閉じる」ボタン（BR14.4）
        └── 開いたときの詳細: AC4.1.1 の 7 項目
```

## 2. 状態（Model に足すもの）

| 名前 | 形 | 内容 |
| --- | --- | --- |
| myTickets | 読み込みの状態（NotAsked / Loading / Loaded(OwnedTicket の並び) / Failed(理由)） | 一覧（5.1 の状態遷移） |
| openTicketIds | チケット ID の集合 | 開いているチケット。最初は空 |

従業員を切り替えたら、両方を初期値（NotAsked・空）に戻す（BR14.8）。

## 3. 画面と URL

| Route | URL | 内容 |
| --- | --- | --- |
| MyTicketsPage | `/my-tickets` | 購入済みチケットの一覧（BR14.1） |

URL と Route の相互変換（U1 の純粋関数）に 1 つ足す。開閉の状態は URL に含めない。

## 4. メッセージと更新（足すもの）

| メッセージ | 更新 | コマンド |
| --- | --- | --- |
| 一覧の画面へ移る（URL が `/my-tickets` になる） | myTickets = Loading、openTicketIds = 空 | GET `/api/tickets` |
| 一覧を受け取った（成功） | 一覧の画面にいるときだけ myTickets = Loaded | なし |
| 一覧を受け取った（失敗） | 一覧の画面にいるときだけ myTickets = Failed | なし |
| 再試行 | myTickets = Loading | GET `/api/tickets` |
| 詳細の開閉（チケット ID） | openTicketIds にあれば外し、なければ入れる | なし |
| 従業員を切り替えた（U1） | myTickets = NotAsked、openTicketIds = 空（U1 の処理に追加） | U1 のまま |

更新関数は純粋関数とし、通信はコマンドとして外に出す（U1・U2 と同じ）。

## 5. 描画のルール

- 並びは Web API の応答のとおりに描き、画面で並べ替えない（BR12.2 は Web API 側）。
- 受講済みのチケットは「受講済み」を文字で表示する（BR14.5）。
- 開閉のボタンは `button` 要素にし、名前を「詳細を見る」「詳細を閉じる」、開いているかどうかを `aria-expanded` で伝え、詳細の部分と `aria-controls` で結ぶ（BR14.4、NFR5）。キーボードだけで操作できる。
- 開いたときの 7 項目の表示形式（日時は日本時間、ポイントは 3 桁区切りに「pt」）は U1 の完了画面と同じ表示の関数を使う（U1 の BR7.10）。
- 0 件の案内「購入済みのチケットはありません」と「コース一覧へ」（BR14.3）。

## 6. 入力のチェック

入力欄はない。開閉のボタンとリンクだけ。

## 7. Web API との連携

| 要求 | 応答 | 画面での使い方 |
| --- | --- | --- |
| GET `/api/tickets`（Cookie 付き） | OwnedTicket の並び（各要素に ticketId・ticketNumber・courseTitle・startsAt・endsAt・venue・deliveryFormat・employeeName・purchasedAt・paidPoints・attendance） | 一覧の描画と、開いたときの詳細（BR13.2） |

通信は U1 と同じ部品（Cookie 付きの fetch と、2xx 以外の本文の受け取り）を使う。
