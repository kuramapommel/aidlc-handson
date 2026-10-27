# AI-DLC 状態管理

## プロジェクト情報

| 項目 | 値 |
| --- | --- |
| プロジェクト名 | シンプル EC サイト「Hibi Market（ひびマーケット）」 |
| プロジェクト種別 | Greenfield（新規開発） |
| スコープ | workshop |
| 開始日 | 2026-10-27 |
| 言語 | ドキュメント・チャット応答はすべて日本語 |
| デプロイ範囲 | ローカルでの動作検証まで（クラウドへのデプロイは対象外） |

## ワークフロー設定

| 設定 | 値 | 備考 |
| --- | --- | --- |
| レビュー上限（review limit） | **none** | ユーザー指示により変更。各ステージのレビュー反復回数に上限を設けない |
| 実行モード | 自律実行 | `/aidlc` コマンドがセッションに未インストールのため、AI-DLC の手順を手動で再現。承認ゲートは本ドキュメントと `audit.md` に記録する |

## スキップ指定されたステージ

`--skip` で指定された以下のステージは実行しない。

- reverse-engineering（Greenfield のため既存コード解析不要）
- practices-discovery
- refined-mockups
- contract-design
- nfr-design
- infrastructure-design
- ci-pipeline
- deployment-pipeline
- environment-provisioning
- deployment-execution
- observability-setup
- incident-response
- performance-validation
- feedback-optimization

## ステージ進捗

### 🔵 INCEPTION フェーズ

- [x] ワークスペース検出（Workspace Detection）
- [-] リバースエンジニアリング — スキップ
- [-] プラクティス探索 — スキップ
- [x] 要件分析（Requirements Analysis）
- [x] ユーザーストーリー（User Stories）
- [x] ワークフロー計画（Workflow Planning）
- [x] アプリケーション設計（Application Design）
- [x] ユニット生成（Units Generation）

### 🟢 CONSTRUCTION フェーズ（ユニット: ec-app）

- [x] 機能設計（Functional Design）
- [x] NFR 要件（NFR Requirements）
- [-] NFR 設計 — スキップ
- [-] インフラ設計 — スキップ
- [-] コントラクト設計 — スキップ
- [x] コード生成（Code Generation）
- [x] ビルド＆テスト（Build and Test）— ローカル動作検証まで

### 🟡 OPERATIONS フェーズ

- [-] すべてスキップ（CI/CD・デプロイ・監視・インシデント対応・性能検証・フィードバック最適化）

## 現在のステータス

**完了** — ローカル環境でのビルド・ユニットテスト・E2E テストが成功。
