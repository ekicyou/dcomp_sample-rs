# Implementation Plan

| 項目 | 内容 |
|------|------|
| **Feature** | event-hit-test |
| **Version** | 1.0 |
| **Date** | 2025-12-02 |
| **Requirements** | 8 Functional, 4 Non-Functional |
| **Design** | Approved |

---

## Tasks

- [x] 1. 深さ優先・逆順・後順走査イテレータの実装
- [x] 1.1 (P) DepthFirstReversePostOrder イテレータを作成
  - スタック + フラグ方式で後順走査を実現
  - `(Entity, bool)` タプルでスタック管理（bool = 子展開済みフラグ）
  - Children 配列を逆順で積むことで最前面の子から走査
  - `ecs::common` モジュールに配置し、汎用的に再利用可能にする
  - _Requirements: 3_

- [x] 1.2 (P) イテレータのユニットテストを作成
  - 基本走査順序テスト（6ノードツリー）
  - 単一ノード（子なし）テスト
  - 深い階層テスト（4階層）
  - 幅広ツリーテスト（4兄弟）
  - _Requirements: 3_

- [x] 2. HitTestMode と HitTest コンポーネントの実装
- [x] 2.1 (P) HitTestMode enum を定義
  - `None`: ヒットテスト対象外
  - `Bounds`: 矩形領域でヒットテスト（デフォルト）
  - 将来の拡張（AlphaMask 等）を考慮した設計
  - `ecs::layout` モジュールに配置
  - _Requirements: 1_

- [x] 2.2 (P) HitTest コンポーネントを実装
  - `mode: HitTestMode` フィールドを持つ
  - `Default` トレイト実装（Bounds）
  - `HitTest::none()` と `HitTest::bounds()` コンストラクタ
  - `#[derive(Component)]` でECSコンポーネント化
  - _Requirements: 1_

- [x] 3. 単一エンティティヒットテストの実装
- [x] 3.1 hit_test_entity 関数を実装
  - 指定エンティティのみを判定（子孫は走査しない）
  - `world.get::<GlobalArrangement>()` で bounds 取得
  - `world.get::<HitTest>()` でモード取得（None なら暗黙的 Bounds）
  - `HitTestMode::None` の場合は false を返す
  - `GlobalArrangement.bounds.contains()` で判定
  - _Requirements: 1, 2, 4, 6_

- [x] 3.2 hit_test_entity のユニットテストを作成
  - ヒットあり（bounds 内）
  - ヒットなし（bounds 外）
  - HitTestMode::None のスキップ
  - HitTest コンポーネントなし（デフォルト動作）
  - GlobalArrangement なしの場合
  - _Requirements: 1, 2, 4, 6_

- [x] 4. ツリー走査ヒットテストの実装
- [x] 4.1 hit_test 関数を実装
  - `DepthFirstReversePostOrder` イテレータで走査
  - 各エンティティで `hit_test_entity` を呼び出し
  - 最初のヒットで early return
  - `ecs::layout` モジュールに配置
  - _Requirements: 2, 3, 8_

- [x] 4.2 hit_test の統合テストを作成
  - ヒットあり（最前面優先）
  - ヒットあり（背面のみ）
  - ヒットなし
  - HitTestMode::None のスキップ（子は引き続き調査）
  - 親子関係での優先度確認
  - _Requirements: 2, 3, 4, 8_

- [x] 5. ウィンドウ座標ヒットテストの実装
- [x] 5.1 hit_test_in_window 関数を実装
  - `WindowPos.position` からウィンドウ左上座標を取得
  - `client_point + window_position = screen_point` で変換
  - `hit_test` に委譲
  - _Requirements: 5, 8_

- [x] 5.2 hit_test_in_window のテストを作成
  - クライアント座標からスクリーン座標への変換確認
  - WindowPos なしの場合の None 返却
  - _Requirements: 5, 8_

- [x] 6. モジュール統合とエクスポート
- [x] 6.1 ecs::layout モジュールへの統合
  - `hit_test.rs` を作成し API を実装
  - `mod.rs` に `pub mod hit_test; pub use hit_test::*;` 追加
  - `ecs::common` モジュールの公開設定を確認
  - _Requirements: 1, 6_

- [x] 6.2 cargo test でのビルド確認とドキュメントコメント整備
  - 全テストが通過することを確認
  - doc comments でAPI使用例を記載
  - _Requirements: 8_

- [ ] 7. taffy_flex_demo での統合テスト
- [ ] 7.1 デモにヒットテスト検証を追加
  - 表示1秒後にヒットテストAPIを呼び出し、結果をログ出力
  - 表示6秒後に再度ヒットテストを実行
  - `hit_test` 関数でルートからの走査をテスト
  - ヒットしたエンティティの Name をログ出力
  - _Requirements: 8_

- [ ] 7.2 ヒットテスト結果の目視確認
  - ログ出力で正しいエンティティがヒットすることを確認
  - 座標とヒット対象の対応関係を検証
  - _Requirements: 2, 3, 8_

---

## Requirements Coverage

| Requirement | Tasks |
|-------------|-------|
| 1. HitTestMode/HitTest 定義 | 2.1, 2.2, 3.1, 6.1 |
| 2. 矩形ヒットテスト | 3.1, 3.2, 4.1, 4.2, 7.2 |
| 3. Z順序によるヒット優先度 | 1.1, 1.2, 4.1, 4.2, 7.2 |
| 4. ヒットテスト除外 | 3.1, 3.2, 4.2 |
| 5. 座標変換 | 5.1, 5.2 |
| 6. ECSシステム統合 | 3.1, 3.2, 6.1 |
| 7. ヒットテスト呼び出しタイミング | （Phase 2 - 本スコープ外） |
| 8. ヒットテストAPI | 4.1, 4.2, 5.1, 5.2, 6.2, 7.1, 7.2 |

**Note**: Requirement 7（ヒットテスト呼び出しタイミング）は `WM_MOUSEMOVE` ハンドラ統合であり、`event-mouse-basic` 仕様で対応予定。本タスクではAPI実装のみをスコープとする。

---

## Parallel Execution Notes

- **Task 1.1, 1.2**: イテレータ実装とテストは並列実行可能
- **Task 2.1, 2.2**: コンポーネント定義は互いに依存しないため並列実行可能
- **Task 3.x**: Task 1.x, 2.x の完了後に実行
- **Task 4.x**: Task 3.x の完了後に実行
- **Task 5.x**: Task 4.x の完了後に実行
- **Task 6.x**: 全タスク完了後に統合

---

## Estimated Effort

| Task | Estimated Time |
|------|----------------|
| 1.1 | 1.5 hours |
| 1.2 | 1 hour |
| 2.1 | 0.5 hours |
| 2.2 | 0.5 hours |
| 3.1 | 1.5 hours |
| 3.2 | 1 hour |
| 4.1 | 1 hour |
| 4.2 | 1.5 hours |
| 5.1 | 1 hour |
| 5.2 | 0.5 hours |
| 6.1 | 0.5 hours |
| 6.2 | 0.5 hours |
| **Total** | **~11 hours** |
