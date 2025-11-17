# Implementation Plan

**Feature ID**: `phase4-mini-horizontal-text`  
**Generated**: 2025-11-17  
**Status**: Tasks Generated

---

## Task Overview

本実装計画は、DirectWriteを使用した横書きテキストレンダリング機能をwintfフレームワークに統合する。設計フェーズで定義された3層アーキテクチャ（COM Wrapper → ECS Component → Message Handling）に従い、既存のRectangle実装パターンを踏襲する。

**実装方針**:
- 既存のIDWriteFactory2統合を活用
- draw_rectanglesと同様のCommandList生成パターンを使用
- Changed検知による効率的な再描画
- 段階的な統合により早期に動作確認

---

## Tasks

### 1. COM APIラッパー拡張

- [ ] 1.1 (P) DirectWriteファクトリー拡張トレイト実装
  - `com/dwrite.rs`に`DWriteFactoryExt`トレイト定義
  - `create_text_layout`メソッド実装: UTF-8文字列をPCWSTRに変換してIDWriteTextLayout生成
  - UTF-8→UTF-16変換はwindows-rsのParam<PCWSTR>トレイトで自動処理
  - max_width/max_heightパラメータでレイアウト領域制限
  - COM API呼び出しのエラーハンドリング（windows::core::Result）
  - _Requirements: 3_

- [ ] 1.2 (P) Direct2Dデバイスコンテキスト拡張トレイト実装
  - `com/d2d/mod.rs`に`D2D1DeviceContextExt`トレイト定義
  - `draw_text_layout`メソッド実装: IDWriteTextLayoutを指定座標に描画
  - D2D1_DRAW_TEXT_OPTIONSパラメータサポート
  - ブラシ（ID2D1Brush）でテキスト色指定
  - 既存のcreate_solid_color_brush拡張メソッドと整合性確保
  - _Requirements: 7_

### 2. ECSコンポーネント定義

- [ ] 2.1 (P) Labelコンポーネント実装
  - `ecs/widget/text/label.rs`作成
  - Labelコンポーネント定義: text, font_family, font_size, color, x, y フィールド
  - bevy_ecsのComponentトレイト実装
  - Default実装: デフォルト値設定（"", "メイリオ", 16.0, 黒色, 0.0, 0.0）
  - on_remove hook実装: GraphicsCommandListをクリア（Changed検出対応）
  - SparseSet storage指定でランダムアクセス最適化
  - _Requirements: 4_

- [ ] 2.2 (P) TextLayoutコンポーネント実装
  - `ecs/widget/text/label.rs`に追加（同ファイルで関連コンポーネントをグループ化）
  - TextLayoutコンポーネント定義: Option<IDWriteTextLayout>をラップ
  - SparseSet storage指定
  - on_remove hook実装（Dropで自動解放されるためログのみ）
  - newメソッド、getメソッド、emptyメソッド提供
  - _Requirements: 5_

### 3. draw_labelsシステム実装

- [ ] 3.1 Labelエンティティクエリとリソース取得
  - `ecs/widget/text/draw_labels.rs`作成
  - draw_labels関数定義: Commands, Query, Res<GraphicsCore>パラメータ
  - Query: (Entity, &Label, &WindowGraphics), Or<(Changed<Label>, Without<GraphicsCommandList>)>
  - GraphicsCoreから dwrite_factory と d2d_device を取得
  - WindowGraphicsの有効性確認（is_valid()）
  - _Requirements: 6, 8_

- [ ] 3.2 TextFormatとTextLayout生成
  - Label情報からTextFormat作成: font_family, font_size, DWRITE_FONT_WEIGHT_NORMAL等
  - ロケール指定（"ja-JP"）
  - create_text_layoutでIDWriteTextLayout生成（max_width/height: f32::MAX）
  - エラー時はeprintln!でログ出力、該当エンティティをスキップ
  - _Requirements: 2, 3_

- [ ] 3.3 CommandList生成とテキスト描画命令記録
  - d2d_device.create_device_context呼び出し（D2D1_DEVICE_CONTEXT_OPTIONS_NONE）
  - unsafe { dc.CreateCommandList() } でCommandList生成
  - unsafe { dc.SetTarget(&command_list) } でターゲット設定
  - unsafe { dc.BeginDraw() } で描画開始
  - dc.clear(Some(&colors::TRANSPARENT)) で透明クリア
  - dc.create_solid_color_brushでブラシ作成（label.color使用）
  - dc.draw_text_layout呼び出し: origin(label.x, label.y), text_layout, brush
  - unsafe { dc.EndDraw(None, None) } で描画終了
  - command_list.close()でCommandList確定
  - Rectangle実装（draw_rectangles）と同パターン厳守
  - _Requirements: 6, 7_

- [ ] 3.4 GraphicsCommandListとTextLayoutコンポーネント挿入
  - commands.entity(entity).insert((GraphicsCommandList::new(command_list), TextLayout::new(text_layout)))
  - Changed<Label>による次フレーム以降の再生成スキップを実現
  - エラー時も処理継続（1エンティティのエラーが全体に影響しない）
  - _Requirements: 5, 9_

### 4. システム統合

- [ ] 4.1 textモジュール登録とエクスポート
  - `ecs/widget/text/mod.rs`作成
  - `pub mod label;`, `pub mod draw_labels;`
  - `pub use label::{Label, TextLayout};`, `pub use draw_labels::draw_labels;`
  - `ecs/widget/mod.rs`に`pub mod text;`追加
  - _Requirements: 4, 5, 6_

- [ ] 4.2 Drawスケジュールにdraw_labelsシステム登録
  - `ecs/world.rs`のDrawスケジュールに`draw_labels`追加
  - render_surfaceシステムの前に実行されるよう順序制御（.before(render_surface)）
  - 既存のdraw_rectanglesと並列実行可能（依存関係なし）
  - _Requirements: 6_

### 5. サンプルアプリケーション

- [ ] 5.1 simple_window.rs拡張またはlabel_demo.rs作成
  - 既存の`examples/simple_window.rs`にLabel使用例追加、または`examples/label_demo.rs`新規作成
  - "Hello, World!"と"こんにちは"を異なる座標に表示
  - 異なるフォントサイズ（例: 16.0, 24.0, 32.0）と色（白、赤、青）の複数Label
  - WindowGraphicsコンポーネントとの関連付け
  - タイマースレッドパターン（simple_window.rsと同様）でテキスト動的変更デモ（任意）
  - cargo run --example simple_window または cargo run --example label_demo で実行可能
  - _Requirements: 11_

### 6. 統合テストと動作検証

- [ ] 6.1* ベースライン描画テスト（任意）
  - `tests/`ディレクトリにテストファイル作成（例: `label_rendering_test.rs`）
  - Label + TextLayoutコンポーネント生成を検証
  - GraphicsCommandList生成を検証
  - エラーハンドリング動作確認（存在しないフォント等）
  - サンプルアプリケーション実行で視覚的確認を優先し、自動テストは補助的位置づけ
  - _Requirements: 1, 2, 3, 4, 5, 7, 10_

- [ ] 6.2 パフォーマンス検証
  - 10個のLabelエンティティを生成してフレームレート確認
  - Changed<Label>によるTextLayout再生成スキップを確認（ログまたはデバッグ出力）
  - Vsync同期で60fps維持を確認
  - 必要に応じてTextFormat生成キャッシング検討（Phase 4ではスコープ外）
  - _Requirements: 9_

- [ ] 6.3 複数Label表示と描画順序確認
  - 単一Windowに対して複数Label（異なる座標・色・サイズ）を配置
  - 全てのLabelが正しく描画されることを確認
  - 描画順序（エンティティID順）を視覚的に検証
  - _Requirements: 8_

---

## Task Dependencies & Execution Order

**Parallel Execution**: `(P)`マーク付きタスクは並列実行可能

- **Phase 1**: タスク1.1, 1.2, 2.1, 2.2 は並列実行可能（独立したファイル作成）
- **Phase 2**: タスク3.1→3.2→3.3→3.4 は順次実行（draw_labelsシステム内部で依存）
- **Phase 3**: タスク4.1→4.2 は順次実行（モジュール登録後にスケジュール統合）
- **Phase 4**: タスク5.1 はPhase 3完了後に実行（統合後のサンプル）
- **Phase 5**: タスク6.1, 6.2, 6.3 はPhase 4完了後に実行（動作検証）

**Critical Path**: 3.1→3.2→3.3→3.4→4.1→4.2→5.1（最短6ステップ）

---

## Requirements Coverage

全11要件を網羅:

- **Requirement 1**: GraphicsCore統合（既存）
- **Requirement 2**: タスク3.2（TextFormat作成）
- **Requirement 3**: タスク1.1, 3.2（TextLayout生成）
- **Requirement 4**: タスク2.1（Labelコンポーネント）
- **Requirement 5**: タスク2.2, 3.4（TextLayoutキャッシュ）
- **Requirement 6**: タスク3.1, 3.3, 4.2（draw_labelsシステム）
- **Requirement 7**: タスク1.2, 3.3（DrawTextLayout呼び出し）
- **Requirement 8**: タスク3.1, 6.3（複数Label表示）
- **Requirement 9**: タスク3.4, 6.2（パフォーマンス）
- **Requirement 10**: 全タスク（エラーハンドリング統合）
- **Requirement 11**: タスク5.1（サンプルアプリケーション）

---

## Implementation Notes

### Architecture Alignment
- **3層アーキテクチャ**: COM Wrapper (タスク1) → ECS Component (タスク2, 3) → Message Handling (既存)
- **Rectangle実装パターン踏襲**: create_device_context → CreateCommandList → SetTarget → BeginDraw/EndDraw → close
- **Changed検知**: bevy_ecsのChanged<Label>で効率的な再描画

### Key Technical Decisions
- **IDWriteFactory2継続**: 既存統合を活用、横書きテキストに必要な機能は全て利用可能
- **TextFormat毎回生成**: Phase 4ではキャッシングなし（10 Labels @ 60fps達成可能）
- **API命名汎用化**: `Label`, `draw_labels`（縦書き拡張時に方向性を含めない）

### Error Handling Strategy
- **COM API**: Result<T>による明示的な伝播
- **System Level**: Graceful degradation（エンティティスキップ、処理継続）
- **User Facing**: eprintln!でログ出力、GraphicsCore初期化失敗時のみ停止

---

_Tasks generated on 2025-11-17_
