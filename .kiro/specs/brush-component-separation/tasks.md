# Implementation Plan

## Task 1: Brush/Brushes コンポーネント基盤の実装

- [x] 1.1 (P) Brush enum の定義
  - 継承マーカー（Inherit）と単色（Solid）の2バリアントを持つenumを定義する
  - D2D1_COLOR_Fを内部型として使用するSolidバリアントを実装する
  - 色を取得するas_color()メソッドを実装し、Inheritの場合はNoneを返す
  - 継承状態を判定するis_inherit()メソッドを実装する
  - Clone, Debug, PartialEqトレイトを導出する
  - _Requirements: 1.2, 1.3, 6.1_

- [x] 1.2 (P) 色定数の定義
  - TRANSPARENT（透明）、BLACK、WHITE、RED、GREEN、BLUEの基本色定数を関連定数として定義する
  - 各定数はBrush::Solid形式で定義し、D2D1_COLOR_F値を埋め込む
  - 既存のcolorsモジュールの定数を置き換える準備を行う
  - _Requirements: 5.2_

- [x] 1.3 Brushes コンポーネントの定義
  - foregroundとbackgroundの2つのBrushフィールドを持つ構造体を定義する
  - SparseSetストレージ戦略を指定する
  - デフォルト値として両フィールドにBrush::Inheritを設定する
  - with_foreground()、with_background()、with_colors()ファクトリメソッドを実装する
  - Clone, Debug, PartialEqトレイトを導出する
  - _Requirements: 1.1, 2.1, 2.2, 2.3, 2.4_

- [x] 1.4 モジュール配置とエクスポート
  - ecs/widget/brushes.rsファイルを作成する
  - widget/mod.rsでBrush、Brushesを公開エクスポートする
  - 既存のcolorsモジュール参照を削除可能な状態にする
  - _Requirements: 5.3_

## Task 2: BrushInherit マーカーと継承解決システムの実装

- [x] 2.1 BrushInherit マーカーコンポーネントの定義
  - 未解決状態を示すマーカーコンポーネントを定義する
  - SparseSetストレージ戦略を指定する（一時的マーカーに最適）
  - Defaultトレイトを導出する
  - _Requirements: 3.4_

- [x] 2.2 Visual on_add フックの拡張
  - 既存のon_visual_addフックにBrushInheritマーカー挿入を追加する
  - Brushesコンポーネントは挿入しない（オプショナル設計）
  - 既存のArrangement/VisualGraphics/SurfaceGraphics挿入ロジックを維持する
  - _Requirements: 3.4_

- [x] 2.3 resolve_inherited_brushes システムの実装
  - With<BrushInherit>フィルタで未解決エンティティのみをクエリする
  - ChildOfを辿って親エンティティのBrushes値を取得する
  - Brushesがあればそのフィールドを解決、なければ親から継承して新規挿入する
  - ルートまでBrushesがない場合はデフォルト色（foreground=BLACK、background=TRANSPARENT）を適用する
  - 解決完了後にBrushInheritマーカーを除去する
  - _Requirements: 4.4, 4.5_

- [x] 2.4 Drawスケジュールへのシステム登録
  - resolve_inherited_brushesシステムをDrawスケジュールに追加する
  - bevy_ecsの順序最適化により描画システムより前に実行されることを確認する
  - _Requirements: 4.4_

## Task 3: Rectangle ウィジェットのマイグレーション

- [x] 3.1 Rectangle構造体から色プロパティを除去
  - colorフィールドを削除する
  - Rectangle::new()コンストラクタを追加する
  - 既存のフィールド参照箇所をコンパイルエラーで検出する
  - _Requirements: 3.1_

- [x] 3.2 Rectangle描画システムのBrushes対応
  - draw_rectangles/update_rectangle_command_listシステムのクエリにBrushesを追加する
  - rectangle.colorの代わりにbrushes.foregroundから色を取得する
  - Changed<Brushes>フィルタを導入して効率的な再描画を実現する
  - _Requirements: 4.1, 4.6_

- [x] 3.3 colorsサブモジュールの削除
  - rectangle.rs内のcolorsモジュールをdeprecatedにする
  - 色定数の参照をBrush::XXXに置き換える
  - _Requirements: 5.2, 5.3_

## Task 4: Label ウィジェットのマイグレーション

- [x] 4.1 Label構造体から色プロパティを除去
  - colorフィールドを削除する
  - 既存のフィールド参照箇所をコンパイルエラーで検出する
  - _Requirements: 3.2_

- [x] 4.2 Label描画システムのBrushes対応
  - Label描画関連システムのクエリにBrushesを追加する
  - label.colorの代わりにbrushes.foregroundから色を取得する
  - Changed<Brushes>フィルタを導入する
  - _Requirements: 4.2, 4.6_

## Task 5: Typewriter ウィジェットのマイグレーション

- [x] 5.1 Typewriter構造体から色プロパティを除去
  - foregroundフィールドを削除する
  - backgroundフィールドを削除する
  - 既存のフィールド参照箇所をコンパイルエラーで検出する
  - _Requirements: 3.3_

- [x] 5.2 Typewriter描画システムのBrushes対応
  - draw_typewriters/draw_typewriter_backgrounds関連システムのクエリにBrushesを追加する
  - typewriter.foreground/backgroundの代わりにbrushes.foreground/backgroundから色を取得する
  - background色がNone（Inherit未解決後もなし）の場合は背景描画をスキップする
  - Changed<Brushes>フィルタを導入する
  - _Requirements: 4.3, 4.6_

## Task 6: サンプルアプリケーションの更新

- [x] 6.1 (P) typewriter_demo.rsの更新
  - Typewriter生成時にBrushesコンポーネントを別途指定する形式に変更する
  - foreground/background色の指定をBrushes::with_colors()で行う
  - コンパイル確認と動作確認を行う
  - _Requirements: 3.5, 5.1_

- [x] 6.2 (P) taffy_flex_demo.rsの更新
  - Rectangle生成時にBrushesコンポーネントを別途指定する形式に変更する
  - 色指定をBrushes::with_foreground()で行う
  - コンパイル確認と動作確認を行う
  - _Requirements: 3.5, 5.1_

- [x] 6.3 (P) その他のサンプルアプリケーションの確認
  - graphics_reinit_test.rs、taffy_flex_demo_old.rs等で色指定を使用している箇所を更新する
  - 新しいAPI形式に更新した
  - _Requirements: 3.5_

## Task 7: テストの実装と既存テストの修正

- [x] 7.1 (P) Brush/Brushesユニットテストの作成
  - Brush::as_color()がInheritでNone、SolidでSome(color)を返すことを検証する
  - Brush::is_inherit()が各バリアントで正しく判定することを検証する
  - Brushes::default()が両プロパティでBrush::Inheritになることを検証する
  - Brushes::with_foreground/background/colorsが正しい値を設定することを検証する
  - _Requirements: 7.2_

- [x] 7.2 (P) resolve_inherited_brushesシステムテストの作成
  - 親エンティティのBrushes値が子に正しく継承されることを検証する
  - ルートエンティティでデフォルト色が適用されることを検証する
  - 多階層の親子関係で継承が正しく解決されることを検証する
  - ユーザー指定のBrushesがある場合、Inheritフィールドのみ解決されることを検証する
  - _Requirements: 7.6, 7.7_

- [x] 7.3 既存テストの修正
  - Rectangle/Label/Typewriter関連テストでcolor参照を更新する
  - Brushesコンポーネントを使用する形式に修正する
  - 全テストがパスすることを確認する
  - _Requirements: 7.1_

- [ ]\* 7.4 ウィジェット描画の統合テスト
  - RectangleがBrushes.foreground色で描画されることを検証する
  - LabelがBrushes.foreground色で描画されることを検証する
  - TypewriterがBrushes.foreground/background両方の色で描画されることを検証する
  - _Requirements: 7.3, 7.4, 7.5_

## Task 8: 最終統合と検証

- [x] 8.1 全体コンパイル確認
  - cargo build --all-targetsでエラーがないことを確認する
  - 警告を確認し必要に応じて対応する
  - _Requirements: 7.1_

- [x] 8.2 全テスト実行
  - cargo test --all-targetsで全テストがパスすることを確認する
  - 失敗するテストがあれば修正する
  - _Requirements: 7.1_

- [ ] 8.3 サンプルアプリケーション動作確認
  - cargo run --example arekaで視覚的な描画結果を確認する
  - cargo run --example typewriter_demoでTypewriterの色が正しく表示されることを確認する
  - cargo run --example taffy_flex_demoでRectangleの色が正しく表示されることを確認する
  - _Requirements: 7.3, 7.4, 7.5_
