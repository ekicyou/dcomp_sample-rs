# Requirements Document

## Introduction
wintf プロジェクトおよびワークスペース全体の依存パッケージを最新安定バージョンに更新し、ビルドおよびテストが正常に通る状態にする。非互換更新（bevy 0.18, ambassador 0.5, rand 0.10 等）を含む全面更新を実施し、API変更に伴うコード修正も本仕様の範囲とする。対象はワークスペースルートの `Cargo.toml` で管理される `[workspace.dependencies]` と、各クレートの `Cargo.toml` に記載されたクレート固有の依存関係の両方を含む。

## Requirements

### Requirement 1: ワークスペース依存の最新化
**Objective:** 開発者として、ワークスペースの全依存パッケージを最新安定バージョンに更新したい。これにより、セキュリティ修正・バグ修正・パフォーマンス改善・最新機能の恩恵を受けられるようにする。

#### Acceptance Criteria
1. When 依存パッケージの更新を実行した場合, the wintf workspace shall ワークスペースルート `Cargo.toml` の `[workspace.dependencies]` に記載された全パッケージを最新安定バージョンに更新する
2. When 依存パッケージの更新を実行した場合, the wintf workspace shall 各クレート固有の `Cargo.toml` に記載された非ワークスペース依存（例: `async-io`, `image`）も最新安定バージョンに更新する
3. When 非互換更新（メジャーバージョンアップ）が含まれる場合, the wintf workspace shall bevy 0.18.0, ambassador 0.5.0, rand 0.10.0 等の最新メジャーバージョンに更新する

### Requirement 2: ビルド成功の保証
**Objective:** 開発者として、依存更新後にプロジェクト全体のビルドが成功することを保証したい。これにより、更新に起因する破壊的変更がないことを確認できる。

#### Acceptance Criteria
1. When 依存パッケージの更新が完了した場合, the wintf workspace shall `cargo build` がエラーなしで成功する
2. When 依存パッケージの更新が完了した場合, the wintf workspace shall `cargo build --release` がエラーなしで成功する
3. If 破壊的変更（API変更）が発生した場合, the wintf workspace shall 影響を受けるコードを新しいAPIに適合するよう修正する

### Requirement 3: テスト・サンプル通過の保証
**Objective:** 開発者として、依存更新後に既存テストおよびサンプルアプリケーションが全て正常に動作することを保証したい。これにより、機能的な退行がないことを確認できる。

#### Acceptance Criteria
1. When 依存パッケージの更新が完了した場合, the wintf workspace shall `cargo test` で全テストがパスする
2. When 依存パッケージの更新が完了した場合, the wintf workspace shall `cargo build --examples` で全サンプルがビルドに成功する
3. If テストまたはサンプルコードに破壊的変更の影響がある場合, the wintf workspace shall 影響を受けるコードを新しいAPIに適合するよう修正する

### Requirement 4: ステアリング情報の整合性維持
**Objective:** 開発者として、依存更新後にステアリングドキュメント（`tech.md`）のバージョン情報が実態と一致することを保証したい。

#### Acceptance Criteria
1. When 依存パッケージのバージョンが変更された場合, the wintf workspace shall `.kiro/steering/tech.md` の `Key Libraries` セクションに記載されたバージョン番号を実際のバージョンに更新する
