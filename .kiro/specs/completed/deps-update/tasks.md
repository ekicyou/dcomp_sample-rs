# Implementation Plan: deps-update

## Task Overview
依存パッケージを段階的に最新バージョンに更新し、破壊的変更に対応するコード修正を実施する。互換更新を先行させることで安全な基盤を確保し、bevy → ambassador → rand の順に非互換更新を適用する。

## Tasks

- [x] 1. 互換パッケージ更新と検証
- [x] 1.1 互換パッケージのバージョンを更新
  - ワークスペースルート Cargo.toml で taffy を 0.9.2 に更新
  - ワークスペースルート Cargo.toml で human-panic を 2.0.6 に更新
  - クレート固有 Cargo.toml で async-io を 2.6 に更新
  - _Requirements: 1.1, 1.2_

- [x] 1.2 互換更新のビルド・テスト検証
  - cargo build でビルドが成功することを確認
  - cargo test で全テストがパスすることを確認
  - _Requirements: 2.1, 3.1_

- [x] 2. bevy 0.18 Cargo.toml 更新
- [x] 2.1 bevy 系パッケージのバージョンを 0.18.0 に更新
  - ワークスペースルート Cargo.toml で bevy_ecs を 0.18.0 に更新
  - ワークスペースルート Cargo.toml で bevy_app を 0.18.0 に更新
  - ワークスペースルート Cargo.toml で bevy_tasks を 0.18.0 に更新
  - ワークスペースルート Cargo.toml で bevy_utils を 0.18.0 に更新
  - _Requirements: 1.3_

- [x] 3. bevy 0.18 API修正とビルド検証
- [x] 3.1 bevy 0.18 コンパイルエラーの修正
  - cargo build を実行してコンパイルエラーを特定
  - import パス変更に対応（必要な場合）
  - DetectChangesMut のインポートパスを調整（必要な場合）
  - HookContext のパス変更に対応（必要な場合）
  - IntoSystem ジェネリクスの新パラメータ In に対応（テストコード）
  - Message/Messages API のパス変更に対応（必要な場合）
  - Mutable, lifetimeless のパス変更に対応（必要な場合）
  - ExecutorKind の enum variant 名称変更に対応（必要な場合）
  - コンパイルエラーがなくなるまで修正を反復
  - _Requirements: 2.3_

- [x] 3.2 bevy 0.18 ビルド・テスト検証
  - cargo build でビルドが成功することを確認
  - cargo build --release でリリースビルドが成功することを確認
  - cargo test で全テストがパスすることを確認
  - _Requirements: 2.1, 2.2, 3.1_

- [x] 4. ambassador 0.5 更新と検証
- [x] 4.1 ambassador のバージョンを 0.5.0 に更新
  - ワークスペースルート Cargo.toml で ambassador を 0.5.0 に更新
  - _Requirements: 1.3_

- [x] 4.2 ambassador 0.5 ビルド検証とコード修正
  - cargo build でコンパイルエラーの有無を確認
  - エラーがある場合は delegatable_trait, Delegate マクロ使用箇所を修正
  - コンパイルエラーがなくなるまで修正を反復
  - _Requirements: 2.3_

- [x] 5. rand 0.10 更新と検証
- [x] 5.1 rand のバージョンを 0.10.0 に更新
  - クレート固有 Cargo.toml で rand を 0.10.0 に更新（dev-dependency）
  - _Requirements: 1.3_

- [x] 5.2 rand 0.10 サンプルコード修正
  - examples/dcomp_demo.rs で cargo build --examples を実行しエラーを確認
  - RngExt トレイトのインポート問題に対応（必要な場合 use rand::RngExt を追加）
  - random_range(), shuffle() の呼び出しを修正（必要な場合）
  - コンパイルエラーがなくなるまで修正を反復
  - _Requirements: 2.3, 3.3_

- [x] 5.3 rand 0.10 サンプルビルド検証
  - cargo build --examples で全サンプルがビルドできることを確認
  - _Requirements: 3.2_

- [x] 6. ステアリングドキュメント更新
- [x] 6.1 tech.md のバージョン情報を更新
  - tech.md の Key Libraries セクションで bevy_ecs を 0.18.0 に更新
  - tech.md の Key Libraries セクションで taffy を 0.9.2 に更新
  - tech.md の Key Libraries セクションで human-panic を 2.0.6 に更新（記載がある場合）
  - _Requirements: 4.1_

- [x] 7. 最終検証
- [x] 7.1 全体ビルド・テスト・サンプル検証
  - cargo build でビルドが成功することを確認
  - cargo build --release でリリースビルドが成功することを確認
  - cargo test で全テストがパスすることを確認
  - cargo build --examples で全サンプルがビルドできることを確認
  - cargo clippy で新規警告がないことを確認（ベストエフォート）
  - _Requirements: 2.1, 2.2, 3.1, 3.2_

## Requirements Coverage

| Requirement | Mapped Tasks |
|-------------|--------------|
| 1.1 | 1.1 |
| 1.2 | 1.1 |
| 1.3 | 2.1, 4.1, 5.1 |
| 2.1 | 1.2, 3.2, 7.1 |
| 2.2 | 3.2, 7.1 |
| 2.3 | 3.1, 4.2, 5.2 |
| 3.1 | 1.2, 3.2, 7.1 |
| 3.2 | 5.3, 7.1 |
| 3.3 | 5.2 |
| 4.1 | 6.1 |

## Execution Notes

### 段階的実行の重要性
- タスクは順序通り（1 → 2 → 3 → 4 → 5 → 6 → 7）に実行すること
- 各タスク完了時に Git commit を作成し、ロールバックポイントを確保すること
- エラーが発生した場合は前のタスクに戻れる状態を維持すること

### コンパイル駆動修正
- タスク 3.1, 4.2, 5.2 はコンパイルエラーに従って修正を反復する
- research.md の調査結果を参照しつつ、実際のエラーメッセージを優先すること
- 修正が困難な場合は該当パッケージのバージョンを戻すことも検討すること
