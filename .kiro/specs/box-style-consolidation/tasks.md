# Implementation Plan

## Task Format

- [ ] Major task
- [ ] Sub-task with details

---

## Tasks

- [ ] 1. BoxStyle統合コンポーネントの実装
- [ ] 1.1 (P) BoxStyle構造体を定義し、全レイアウトプロパティを統合する
  - Box系5種（size, margin, padding, position, inset）をOption型でネスト構造として含める
  - Flex系7種（flex_direction, justify_content, align_items, flex_grow, flex_shrink, flex_basis, align_self）をフラットなOption型フィールドとして含める
  - `#[derive(Component, Debug, Clone, PartialEq, Default)]`を付与
  - _Requirements: 1.1, 1.4_

- [ ] 1.2 (P) 既存のBox系・Flex系コンポーネントからComponent deriveを削除し、値オブジェクト化する
  - BoxSize, BoxMargin, BoxPadding, BoxPosition, BoxInsetから`#[derive(Component)]`を削除
  - FlexContainer, FlexItemから`#[derive(Component)]`を削除
  - 構造体・列挙型としての機能は維持し、既存フィールドはそのまま保持
  - _Requirements: 3.1_

- [ ] 1.3 BoxStyleからtaffy::Styleへの変換トレイトを実装する
  - `From<&BoxStyle> for taffy::Style`を実装
  - 各フィールドのNone時にtaffyデフォルト値を適用（flex_grow: 0.0, flex_shrink: 1.0）
  - Flex系プロパティ設定時にdisplay: Flexを自動設定
  - _Requirements: 1.2, 3.2_

- [ ] 2. レイアウトシステムのクエリ簡略化
- [ ] 2.1 build_taffy_styles_systemを新しいBoxStyle単一クエリに更新する
  - 8コンポーネントクエリを`&BoxStyle`と`Option<&BoxStyle>`の2クエリに置き換え
  - 変更検出を`Changed<BoxStyle>`に統一
  - LayoutRootのみのエンティティには`BoxStyle::default()`相当を適用
  - 重複していたスタイル構築ロジックを統合
  - _Requirements: 2.1, 2.3_

- [ ] 2.2 (P) 仮想デスクトップ矩形取得用ヘルパー関数を追加する
  - GetSystemMetrics APIを使用してSM_XVIRTUALSCREEN等から座標・サイズを取得
  - 関数シグネチャ: `fn get_virtual_desktop_bounds() -> (i32, i32, i32, i32)`
  - _Requirements: 2.4_

- [ ] 2.3 initialize_layout_rootをBoxStyle使用に更新する
  - LayoutRootエンティティに仮想デスクトップ矩形を設定したBoxStyleを付与
  - BoxPosition::AbsoluteとBoxInsetで絶対座標を指定
  - Monitorエンティティも同様にBoxStyleに移行
  - update_monitor_layout_systemも合わせて更新
  - _Requirements: 2.4_

- [ ] 3. テスト・サンプルの移行
- [ ] 3.1 (P) 既存のレイアウトテストを新APIに移行する
  - taffy_layout_integration_test.rsを更新
  - taffy_flex_layout_pure_test.rsを更新
  - taffy_advanced_test.rsを更新
  - その他レイアウト関連テストのコンパイルエラーを解消
  - _Requirements: 3.3, 3.4_

- [ ] 3.2 (P) サンプルアプリケーションを新APIに移行する
  - taffy_flex_demo.rsのコンポーネント使用箇所をBoxStyleに変更
  - 動作確認（表示結果が現行と同一であること）
  - _Requirements: 3.4_

- [ ] 4. ビルド検証と最終確認
- [ ] 4.1 全ターゲットビルドとテスト実行で移行漏れがないことを確認する
  - `cargo build --all-targets`の成功を確認
  - `cargo test`の全パスを確認
  - `cargo run --example taffy_flex_demo`の正常動作を確認
  - コンパイル警告がないことを確認
  - _Requirements: 2.2, 3.3_

---

## Requirements Coverage

| Requirement | Tasks |
|-------------|-------|
| 1.1 (BoxStyle全フィールド統合) | 1.1 |
| 1.2 (TaffyStyleとの1:1対応) | 1.3 |
| 1.3 (1コンポーネントクエリ) | 2.1 |
| 1.4 (Flex系フラットフィールド) | 1.1 |
| 2.1 (8→1コンポーネント削減) | 2.1 |
| 2.2 (アーキタイプ断片化最小化) | 4.1 |
| 2.3 (Changed<BoxStyle>統一) | 2.1 |
| 2.4 (LayoutRoot仮想デスクトップ矩形) | 2.2, 2.3 |
| 3.1 (非コンポーネント型維持) | 1.2 |
| 3.2 (From/Intoトレイト) | 1.3 |
| 3.3 (コンパイルエラーで移行明示) | 3.1, 4.1 |
| 3.4 (テスト・サンプル移行例) | 3.1, 3.2 |
