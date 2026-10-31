# NFR 要件（U1 購入の最小の流れ）— 確認事項

U1（`u1-purchase-skeleton`）の性能・セキュリティ・信頼性・可観測性の目標と、技術の選定を決めます。数値の目標（Web API の応答 200 ms 以内、画面の反映 300 ms 以内など）は要件定義書（NFR1〜NFR10）で決まっているため質問しません。要件定義書で「後の工程で決める」とした技術の選定のうち、作り方や起動の手順が大きく変わる 2 点だけを質問します。

Rust 側の crate は、要件の「デファクトスタンダードの最新 stable 版を使う」（C5）に従って選び、`tech-stack-decisions.md` に理由とともに書きます（Web フレームワークは axum、非同期の実行環境は tokio、JSON は serde など）。

---

## Q1. 画面（MoonBit）の作り方

画面は The Elm Architecture（TEA）で組み立てると決めています（ADR-007）。それを MoonBit でどう実現しますか？（要件の未解決事項 OQ1）

なお、MoonBit のツールとパッケージの取得先は、この開発環境では今もネットワークの設定でブロックされています（外部依存 ED-1）。そのため、どちらの案でも、最新版で実際に使えるかはブロックが解除されてから確かめます。

A. MoonBit コミュニティの TEA ライブラリ（rabbit-tea）を使う。TEA の実行環境・仮想 DOM・HTTP の呼び出しを自分で書かずに済む。ブロック解除後に最新版で使えることを確かめ、使えなければ B に切り替える
B. TEA の実行環境を自作する（DOM を直接操作する薄い層を、MoonBit から JavaScript を呼ぶ仕組みで書く）。外部のパッケージに依存しないが、書く量とテストが増える
X. Other (please specify)

[Answer]: A. MoonBit コミュニティの TEA ライブラリ（rabbit-tea）を使う（回答者のことば: 「rabbit-tea をつかいます。ダメなら B は選ばずにダメな原因と対策を教えてください。」。最新版で使えなかった場合は自作（B）に切り替えず、使えない原因と対策を報告して判断を仰ぐ）

## Q2. 画面用の静的ファイルサーバー

ローカルでは「Web API サーバー」と「画面用の静的ファイルサーバー」の 2 プロセスで動かすと決めています（ユニット生成 Q3）。画面用の静的ファイルサーバーには何を使いますか？（ユニット定義で「コンストラクションで決める」とした点）

A. Rust のワークスペースに、静的ファイルを配信する小さな実行ファイルを 1 つ足す。前提のツール（Rust と MoonBit）のほかに何も入れずに、リポジトリのルートからのコマンドで起動できる（NFR9）
B. 開発者の環境にある汎用の静的サーバー（例: Python の `http.server`、Node.js の `npx serve`）を使う。コードは増えないが、前提のツールが増える
X. Other (please specify)

[Answer]: A. Rust のワークスペースに、静的ファイルを配信する小さな実行ファイルを 1 つ足す

---

## Consolidated Summary Confirmation

- 画面の作り方: MoonBit コミュニティの TEA ライブラリ（rabbit-tea）を使う。MoonBit のツールの取得先（`cli.moonbitlang.com`、`download.moonbitlang.com`、`mooncakes.io`）が許可されたあとに、最新版で使えるかを確かめる。使えなかった場合は自作に切り替えず、使えない原因と対策を報告して判断を仰ぐ（Q1）
- 画面用の静的ファイルサーバー: Rust のワークスペースに、静的ファイルを配信する小さな実行ファイルを 1 つ足す。Rust と MoonBit のほかに前提のツールを増やさない（Q2）
- そのほかの数値の目標（Web API の応答 200 ms 以内、画面の反映 300 ms 以内、整合性、セキュリティ、ログの項目など）は要件定義書の NFR1〜NFR10 と機能設計の決定（Cookie と CSRF 対策など）をそのまま引き継ぐ。Rust の crate はデファクトスタンダードの最新 stable 版から選び、理由とともに記録する

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
