# Requirements: ウィンドウ作成時のデフォルトでDirectCompositionを前提としたDefaultパラメーターとする

**Status**: requirements_defined  
**Updated**: 2025-11-14  
**Feature**: dcomp-default-window

## 1. 変更の目的

### 1.1 主要目標
`WindowStyle::default()` のデフォルト値を変更し、DirectComposition使用を前提とした設定とする。

### 1.2 背景と動機

#### プロダクト方針との整合性
プロジェクトは **DirectComposition を標準的な描画方法** として位置づけている:
- Core Capabilities: "DirectComposition、Direct2D、DirectWriteを活用した高品質な2D描画"
- Target Use Cases: "DirectComposition/Direct2Dを使用した高パフォーマンスな2D描画アプリケーション"

#### 現在の問題点
- `WindowStyle::default()` は `ex_style: WINDOW_EX_STYLE(0)` を返す
- DirectCompositionを使用するには、開発者が **毎回明示的に** `WS_EX_NOREDIRECTIONBITMAP` を設定する必要がある
- 既存のサンプルコード（`dcomp_demo.rs`）でも明示的に設定している

#### 改善の効果
- デフォルトでDirectCompositionが使用可能になる
- 開発者の負担を軽減（標準ユースケースでの設定不要）
- プロダクト方針との一貫性を保つ

## 2. 機能要件

### 2.1 変更すべき要素

#### 対象コード
**ファイル**: `crates/wintf/src/ecs/window.rs`  
**関数**: `impl Default for WindowStyle`  
**行番号**: 128-135

```rust
// 現在の実装
impl Default for WindowStyle {
    fn default() -> Self {
        Self {
            style: WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            ex_style: WINDOW_EX_STYLE(0),  // ← ここを変更
        }
    }
}

// 変更後
impl Default for WindowStyle {
    fn default() -> Self {
        Self {
            style: WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            ex_style: WS_EX_NOREDIRECTIONBITMAP,  // ← DirectComposition対応
        }
    }
}
```

### 2.2 保持すべき機能

以下は **変更しない**:
- `style` フィールドの値（`WS_OVERLAPPEDWINDOW | WS_VISIBLE`）
- `WindowStyle` 構造体の定義
- `WindowStyle::from_hwnd()` メソッド
- 他のウィンドウ関連コンポーネント（`Window`, `WindowPos` など）

### 2.3 影響を受けるコード

#### 直接的な影響
1. **window_system.rs** (L24)
   ```rust
   let style_comp = opt_style.copied().unwrap_or_default();
   ```
   - `WindowStyle` コンポーネントが指定されていない場合に `default()` を使用
   - 変更後は自動的に `WS_EX_NOREDIRECTIONBITMAP` が設定される

2. **examples/simple_window.rs** ⭐ **テスト用アプリケーション**
   - `WindowStyle` を明示的に指定していない
   - 変更後は自動的にDirectComposition対応になる
   - **この動作確認が主要なテストケースとなる**

#### 間接的な影響
3. **examples/dcomp_demo.rs**
   - 現在は明示的に `.WS_EX_NOREDIRECTIONBITMAP(true)` を設定
   - 変更後は冗長な設定となるが、動作に影響なし（後で削除可能）

4. **examples/areka.rs**
   - `WindowStyle` を指定していないため、変更の影響を受ける
   - DirectComposition対応が自動的に有効になる

### 2.4 新規要件

特になし（既存の動作を変更するのみ）

## 3. 非機能要件

### 3.1 互換性

#### API互換性
- **破壊的変更**: あり（デフォルト動作が変わる）
- **影響範囲**: `WindowStyle::default()` または `WindowStyle` 省略時の動作
- **軽減策**: 従来の動作が必要な場合、明示的に設定可能

```rust
// 従来の動作が必要な場合
WindowStyle {
    style: WS_OVERLAPPEDWINDOW | WS_VISIBLE,
    ex_style: WINDOW_EX_STYLE(0),
}
```

#### プラットフォーム互換性
- **最小要件**: Windows 8 以降（`WS_EX_NOREDIRECTIONBITMAP` の導入バージョン）
- **既存の要件**: Windows 10/11（DirectComposition対応）
- **結論**: プラットフォーム要件への影響なし

### 3.2 パフォーマンス

#### 期待される影響
- **DirectComposition使用時**: パフォーマンス向上（ハードウェアアクセラレーション）
- **オーバーヘッド**: なし（ウィンドウスタイルの変更のみ）

#### 注意点
- `WS_EX_NOREDIRECTIONBITMAP` 自体はパフォーマンスオーバーヘッドを持たない
- DirectCompositionを使用しない場合でも、悪影響はない

### 3.3 セキュリティ

特に影響なし。ウィンドウスタイルの変更はセキュリティに影響を与えない。

### 3.4 保守性

#### コードの明確性
- プロダクト方針（DirectComposition標準）とコードの一貫性が向上
- 開発者がデフォルト設定を理解しやすくなる

#### ドキュメント要件
以下のドキュメント更新が推奨される:
- `WindowStyle` の Rustdoc コメント（デフォルトがDirectComposition対応であることを明記）
- README.md またはガイドドキュメント（DirectCompositionが標準であることを記載）

## 4. 制約条件

### 4.1 技術的制約

- **Windows バージョン**: Windows 8 以降（既に満たしている）
- **DirectComposition API**: 既存の実装で対応済み
- **ビルド環境**: 変更なし

### 4.2 プロジェクト制約

- **Kiroワークフロー**: 仕様駆動開発プロセスに従う
- **Git履歴**: 意図が明確なコミットメッセージ
- **既存コードへの影響**: 最小限に抑える

## 5. 受け入れ基準

### 5.1 必須条件

- [ ] `WindowStyle::default()` が `WS_EX_NOREDIRECTIONBITMAP` を返す
- [ ] `cargo build` が成功する
- [ ] **テスト用アプリ `simple_window.rs` が正常に動作する**（主要検証）
- [ ] `areka.rs` が正常に動作する（追加検証）
- [ ] `dcomp_demo.rs` が引き続き動作する（互換性検証、冗長な設定があっても問題なし）

### 5.2 検証方法

#### ビルド検証
```bash
cargo build --package wintf
```

#### サンプル実行検証（主要テスト）
**テスト用アプリケーション**: `examples/simple_window.rs`

```bash
# simple_window.rs でデフォルト動作を確認（主要テストケース）
cargo run --example simple_window
```

**検証内容**:
- `simple_window.rs` は `WindowStyle` コンポーネントを明示的に指定していない
- したがって `WindowStyle::default()` が使用される
- 変更後は自動的に `WS_EX_NOREDIRECTIONBITMAP` が設定される
- ウィンドウが正常に作成・表示されることを確認

#### 追加の互換性検証（オプション）
```bash
# areka.rs でデフォルト動作を確認
cargo run --example areka

# dcomp_demo.rs で既存の明示的設定との互換性を確認
cargo run --example dcomp_demo
```

#### 期待される結果
- **simple_window.rs**: ウィンドウが正常に表示され、DirectCompositionが使用可能
- **areka.rs**: 変更の影響を受けるが、正常に動作
- **dcomp_demo.rs**: 冗長な設定があっても引き続き正常に動作

### 5.3 ドキュメント更新（オプション）

以下は推奨だが、必須ではない:
- [ ] `WindowStyle` の Rustdoc にデフォルト設定を明記
- [ ] README.md にDirectCompositionがデフォルトであることを記載

## 6. リスクと軽減策

### 6.1 識別されたリスク

| リスク | 影響度 | 発生確率 | 軽減策 |
|--------|--------|----------|--------|
| 既存コードの破壊的変更 | 中 | 高 | 明示的に古い動作を選択可能にする |
| DirectCompositionを使わないケースでの問題 | 低 | 低 | `WS_EX_NOREDIRECTIONBITMAP` は他の描画方法にも影響しない |
| サンプルコードの動作不良 | 中 | 低 | 実行テストで検証 |

### 6.2 ロールバック計画

問題が発生した場合:
```bash
# コミットを取り消す
git revert HEAD

# または元の実装に戻す
ex_style: WINDOW_EX_STYLE(0),
```

## 7. 依存関係

### 7.1 前提条件

- `.kiro/specs/dcomp-default-window/00_init.md` が作成済み
- 既存のコードが正常にビルド・実行できる

### 7.2 後続フェーズ

- **設計フェーズ**: `/kiro-spec-design dcomp-default-window`
- **タスク分解**: `/kiro-spec-tasks dcomp-default-window`
- **実装フェーズ**: `/kiro-spec-impl dcomp-default-window`

### 7.3 関連する将来のタスク

以下は別のタスクとして扱うべき:
1. `dcomp_demo.rs` の冗長な `.WS_EX_NOREDIRECTIONBITMAP(true)` 設定を削除
2. ドキュメント（Rustdoc、README.md）の更新
3. DirectCompositionを前提とした新しいサンプルコードの作成

## 8. 追加コンテキスト

### 8.1 Windows API 背景

#### WS_EX_NOREDIRECTIONBITMAP の役割
- DWM（Desktop Window Manager）によるビットマップリダイレクトを無効化
- DirectCompositionが直接ウィンドウのビジュアルツリーを制御できるようにする
- Windows 8 で導入

#### DirectComposition との関係
DirectCompositionを使用するウィンドウには必須の設定:
```cpp
// Win32 APIでの典型的な使用例
HWND hwnd = CreateWindowEx(
    WS_EX_NOREDIRECTIONBITMAP,  // DirectComposition用
    className,
    windowName,
    WS_OVERLAPPEDWINDOW,
    ...
);
```

### 8.2 既存実装の分析

#### win_style.rs のヘルパーメソッド
```rust
/// このウィンドウとその子ウィンドウの描画を、画面外のビットマップにリダイレクトしません。
pub fn WS_EX_NOREDIRECTIONBITMAP(self, flag: bool) -> Self {
    set_ex(self, WS_EX_NOREDIRECTIONBITMAP, flag)
}
```

このヘルパーメソッドは引き続き有効で、動的な切り替えが必要な場合に使用可能。

### 8.3 設計哲学

この変更は以下の原則に基づく:
1. **Convention over Configuration**: 標準的なユースケースをデフォルトとする
2. **Principle of Least Surprise**: プロダクト方針と実装の一貫性
3. **Developer Experience**: 繰り返しの設定を削減

---

**次のステップ**: `/kiro-spec-design dcomp-default-window`
