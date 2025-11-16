# Specification: ウィンドウ作成時のデフォルトでDirectCompositionを前提としたDefaultパラメーターとする

**Status**: implementation_complete  
**Created**: 2025-11-14  
**Updated**: 2025-11-14  
**Feature**: dcomp-default-window

## 概要

ウィンドウ作成時のデフォルトパラメーターを、DirectComposition利用を前提とした設定に変更する。

## 目的

DirectCompositionによる高性能な2D描画と透過処理を標準とし、開発者が明示的に設定しなくてもDirectCompositionが利用できるようにする。

## 背景

### 現在の状態
- `WindowStyle::default()`: `WS_OVERLAPPEDWINDOW | WS_VISIBLE` と `WINDOW_EX_STYLE(0)`
- DirectCompositionを使用するには、開発者が明示的に `WS_EX_NOREDIRECTIONBITMAP` を設定する必要がある

### プロダクト方針との整合性
`.kiro/steering/product.md` より:
- **Core Capabilities**: "DirectComposition、Direct2D、DirectWriteを活用した高品質な2D描画"
- **透過ウィンドウ**: "レイヤードウィンドウまたはDirectCompositionによる透過処理"
- **Target Use Cases**: "DirectComposition/Direct2Dを使用した高パフォーマンスな2D描画アプリケーション"

→ DirectCompositionが標準的な描画方法として位置づけられている

## スコープ

### 対象ファイル
- `crates/wintf/src/ecs/window.rs` - `WindowStyle::default()` の実装
- 関連する可能性: `crates/wintf/src/win_style.rs` - スタイル設定ヘルパー

### 変更対象
`WindowStyle::default()` の `ex_style` フィールド:
- **現在**: `WINDOW_EX_STYLE(0)`
- **変更後**: `WS_EX_NOREDIRECTIONBITMAP` を含む設定

## 初期分析

### DirectComposition要件
DirectCompositionを使用するウィンドウには以下が必要:
- `WS_EX_NOREDIRECTIONBITMAP`: DWM (Desktop Window Manager) によるビットマップリダイレクトを無効化
- これにより、DirectCompositionが直接ウィンドウに描画できる

### 既存の実装
- `win_style.rs` に `WS_EX_NOREDIRECTIONBITMAP` を設定するメソッドが存在（L304-306）
- `WindowStyle` 構造体は `ex_style: WINDOW_EX_STYLE` フィールドを持つ

### 影響を受ける可能性のあるコード
- `window_system.rs`: `create_windows` システムが `WindowStyle::default()` を使用
- 既存のサンプルコード（`examples/` ディレクトリ）

## 次のステップ

1. 要件定義フェーズ: `/kiro-spec-requirements dcomp-default-window`
2. 既存コードとのギャップ分析: `/kiro-validate-gap dcomp-default-window` (オプション)
3. 設計フェーズ: `/kiro-spec-design dcomp-default-window`
4. タスク分解: `/kiro-spec-tasks dcomp-default-window`
5. 実装: `/kiro-spec-impl dcomp-default-window`

## メモ

### 技術的考慮事項
- `WS_EX_NOREDIRECTIONBITMAP` は Windows 8 以降で利用可能
- DirectCompositionも Windows 8 以降が対象
- プロジェクトは既にWindows 10/11を対象としている（steering/tech.mdより）

### 互換性への影響
- DirectCompositionを使わない従来型の描画を行う場合、開発者が明示的に設定を上書きする必要がある
- しかし、プロダクト方針上、DirectCompositionが標準であるため、これは妥当なトレードオフ

### 関連する将来のタスク
- DirectComposition使用を前提としたサンプルコードの整備
- ドキュメントの更新（デフォルトがDirectCompositionであることを明記）
