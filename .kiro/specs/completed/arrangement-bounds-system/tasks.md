# Implementation Tasks: arrangement-bounds-system

## Task Overview

本実装は、wintfフレームワークのレイアウトシステムに軸平行バウンディングボックス管理機能を追加する。既存の`Arrangement`コンポーネントにサイズ情報を追加し、`GlobalArrangement`にワールド座標系でのバウンディングボックスを追加する。trait実装の拡張のみで既存の階層伝播システムを再利用する。

**実装規模**: Small (1-2日、約10時間)  
**破壊的変更**: あり（`Arrangement`と`GlobalArrangement`の構造変更）  
**検出方法**: `cargo build --all-targets`でコンパイルエラーを完全検出

---

## Tasks

- [x] 1. 基本データ構造の実装 (P)
- [x] 1.1 (P) Size構造体とRect型エイリアスを定義する
  - `ecs/layout.rs`に`Size`構造体を追加（`width: f32`, `height: f32`）
  - `Component`, `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`トレイトを実装
  - `Rect`型エイリアスを`D2D_RECT_F`として定義
  - _Requirements: 1, 2_

- [x] 1.2 (P) D2DRectExt拡張トレイトを実装する
  - `ecs/layout.rs`に`D2DRectExt`トレイトを定義
  - 構築メソッド: `from_offset_size(offset: Offset, size: Size) -> Self`
  - 取得メソッド: `width()`, `height()`, `offset()`, `size()`（各2行程度）
  - 設定メソッド: `set_offset()`, `set_size()`, `set_left()`, `set_top()`, `set_right()`, `set_bottom()`
  - 判定メソッド: `contains(x, y) -> bool`
  - 演算メソッド: `union(other) -> Self`（2つの矩形の最小外接矩形）
  - バリデーション: `validate()`（デバッグビルドのみ、`left <= right && top <= bottom`を検証）
  - _Requirements: 2, 5_

- [x] 2. Arrangementコンポーネントの拡張
- [x] 2.1 Arrangementにsizeフィールドを追加する
  - 既存の`Arrangement`構造体に`size: Size`フィールドを追加
  - `Arrangement::default()`を更新して`size: Size::default()`を返す
  - `local_bounds() -> Rect`メソッドを実装（`D2DRectExt::from_offset_size()`を使用）
  - コンパイルエラー発生箇所（約10-20箇所）をリストアップ
  - _Requirements: 1_

- [x] 2.2 既存のArrangement初期化コードを修正する
  - `cargo build --all-targets`でコンパイルエラーを確認
  - examples、tests、ライブラリコード内のすべての`Arrangement`初期化箇所で`size`フィールドを追加
  - 各箇所で適切なサイズ値を設定（テストでは明示的なサイズ、examplesでは妥当なデフォルト値）
  - 全ターゲットでコンパイルエラーがなくなることを確認
  - _Requirements: 1_

- [x] 3. GlobalArrangementコンポーネントの拡張
- [x] 3.1 GlobalArrangementを構造体化しboundsフィールドを追加する
  - タプル構造体`GlobalArrangement(Matrix3x2)`を通常の構造体に変更
  - `transform: Matrix3x2`と`bounds: Rect`フィールドを持つ構造体に変更
  - `GlobalArrangement::default()`を実装（`transform: Matrix3x2::identity()`, `bounds: Rect::default()`）
  - 既存のパターンマッチや`.0`アクセスのコンパイルエラー箇所をリストアップ
  - _Requirements: 3_

- [x] 3.2 既存のGlobalArrangementアクセスコードを修正する
  - `cargo build --all-targets`でコンパイルエラーを確認
  - パターンマッチを`GlobalArrangement { transform, bounds }`形式に変更
  - `.0`アクセスを`.transform`に変更
  - 全ターゲットでコンパイルエラーがなくなることを確認
  - _Requirements: 3_

- [x] 4. バウンディングボックス計算の実装 (P)
- [x] 4.1 (P) 軸平行矩形変換関数を実装する
  - `ecs/layout.rs`に`transform_rect_axis_aligned(rect: &Rect, matrix: &Matrix3x2) -> Rect`関数を追加
  - 左上と右下の2点のみを`matrix.transform_point()`で変換（最適化）
  - 変換後の2点から`min/max`で新しい軸平行矩形を構築
  - 回転・スキュー変換が含まれる場合の警告ログ処理（may）
  - _Requirements: 4_

- [x] 4.2 GlobalArrangementのtrait実装を拡張する
  - `Mul<Arrangement>`実装に`bounds`計算を追加
  - 親の`transform`と子の`Arrangement`から結果の`transform`と`bounds`を計算
  - `transform_rect_axis_aligned()`を使用して子の`local_bounds()`を変換
  - `From<Arrangement>`実装に`bounds`設定を追加（`arrangement.local_bounds()`）
  - 既存の`propagate_parent_transforms`システムがそのまま動作することを確認
  - _Requirements: 4_

- [x] 5. バリデーションとエラーハンドリング (P)
- [x] 5.1 (P) サイズとスケールのバリデーションを実装する
  - `Size`の負の値チェック（`width < 0.0`または`height < 0.0`）で警告ログ出力
  - `LayoutScale`のゼロ値チェック（`x == 0.0`または`y == 0.0`）で警告ログ出力
  - 警告ログはmayレベル（必須ではないが推奨）
  - _Requirements: 5_

- [x] 6. ユニットテスト実装
- [x] 6.1 (P) Size構造体とArrangement.local_bounds()をテストする
  - `Size`のデフォルト値、Copy/Clone動作を検証
  - `Arrangement.local_bounds()`の正常ケース（正のサイズ）をテスト
  - エッジケース（ゼロサイズ、負のサイズ）をテスト
  - _Requirements: 6_

- [x] 6.2 (P) transform_rect_axis_aligned関数をテストする
  - 恒等変換のテスト（入力と出力が同じ）
  - 平行移動のみのテスト
  - スケールのみのテスト
  - 平行移動とスケールの組み合わせテスト
  - _Requirements: 6_

- [x] 6.3 (P) D2DRectExt拡張トレイトをテストする
  - `from_offset_size`の動作検証
  - `width`, `height`, `offset`, `size`取得メソッドのテスト
  - `set_offset`, `set_size`設定メソッドのテスト
  - `contains`判定メソッドのテスト（矩形内外の点）
  - `union`演算メソッドのテスト（2つの矩形の最小外接矩形）
  - _Requirements: 6_

- [x] 6.4 (P) GlobalArrangementのtrait実装をテストする
  - `From<Arrangement>`実装: 初期`bounds`が`local_bounds()`から設定されることを検証
  - `Mul<Arrangement>`実装: 親子の`transform`と`bounds`が正しく計算されることを検証
  - _Requirements: 6_

- [x] 7. 統合テスト実装 (P)
- [x] 7.1 (P) 階層的バウンディングボックス計算をテストする
  - 親子3階層のWidgetツリーを構築
  - 各階層で`Arrangement.size`を設定
  - `propagate_global_arrangements`システムを実行
  - 最終的な`GlobalArrangement.bounds`が期待値と一致することを検証
  - 既存の`propagate_parent_transforms`システムが変更なしで動作することを確認
  - _Requirements: 6_

- [x] 8. ドキュメント追加 (P)
- [x] 8.1 (P) コードドキュメントを追加する
  - `Arrangement.size`に関するドキュメントコメント（taffyレイアウト計算との関係）
  - `GlobalArrangement.bounds`に関するドキュメントコメント（座標系とSurface生成との関連）
  - `transform_rect_axis_aligned`に関するドキュメントコメント（軸平行変換のみサポート）
  - `D2DRectExt`の各メソッドに使用例とパフォーマンス特性を記載
  - _Requirements: 6_

---

## Task Summary

- **合計タスク数**: 8メジャータスク、18サブタスク
- **並列実行可能タスク**: 10サブタスク（`(P)`マーク付き）
- **推定工数**: 約10時間（1-2日）
- **要件カバレッジ**: 全6要件をカバー

## Dependencies

- タスク2（Arrangement拡張）はタスク1（基本データ構造）に依存
- タスク3（GlobalArrangement拡張）はタスク1に依存
- タスク4（bounds計算）はタスク1-3に依存
- タスク6-8（テスト・ドキュメント）は並列実行可能

## Quality Gates

- 各タスク完了時: `cargo build --all-targets`でコンパイル成功
- タスク6-7完了時: `cargo test`ですべてのテスト成功
- 全タスク完了時: 統合テストで階層的bounds計算が正しく動作
