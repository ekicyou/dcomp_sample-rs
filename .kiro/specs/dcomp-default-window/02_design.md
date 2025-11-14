# Design: ウィンドウ作成時のデフォルトでDirectCompositionを前提としたDefaultパラメーターとする

**Status**: design_complete  
**Updated**: 2025-11-14  
**Feature**: dcomp-default-window

## 1. アーキテクチャ設計

### 1.1 変更の性質
この変更は**最小限のデフォルト値変更**であり、以下は変更しない:
- `WindowStyle` 構造体の定義
- `WindowStyle::from_hwnd()` メソッド
- 他のウィンドウ関連コンポーネント
- 実装ロジック

### 1.2 変更のスコープ

#### 変更対象
```
crates/wintf/src/ecs/window.rs
  └─ impl Default for WindowStyle
       └─ ex_style: WINDOW_EX_STYLE(0)
            ↓
       └─ ex_style: WS_EX_NOREDIRECTIONBITMAP
```

#### 影響を受けるコンポーネント
```
WindowStyle::default()
    ↓ 使用される場所
window_system.rs: create_windows()
    ↓ 影響を受けるサンプル
examples/simple_window.rs  ⭐ テスト用アプリ
examples/areka.rs
```

### 1.3 設計原則

この変更は以下の原則に基づく:
1. **Convention over Configuration**: 標準ユースケースをデフォルトとする
2. **Principle of Least Surprise**: プロダクト方針と実装の一貫性
3. **Minimal Change**: 1行の変更で最大の効果

## 2. 変更対象ファイルの詳細設計

### 2.1 ファイル: `crates/wintf/src/ecs/window.rs`

#### 変更箇所: `impl Default for WindowStyle`
**行番号**: 128-135

```rust
// 変更前
impl Default for WindowStyle {
    fn default() -> Self {
        Self {
            style: WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            ex_style: WINDOW_EX_STYLE(0),  // ← 変更対象
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

#### インポート確認
`WS_EX_NOREDIRECTIONBITMAP` は以下でインポート済み:
```rust
use windows::Win32::UI::WindowsAndMessaging::*;
```

追加のインポートは **不要**。

#### その他の行
変更なし（`WindowStyle` 構造体定義、`from_hwnd()` メソッドは保持）

## 3. データフロー設計

### 3.1 変更前のフロー

```
Window エンティティ作成
  ↓
window_system.rs: create_windows()
  ├─ WindowStyle コンポーネントあり
  │   └─ 指定された値を使用
  └─ WindowStyle コンポーネントなし
      └─ WindowStyle::default()
          └─ ex_style: WINDOW_EX_STYLE(0)  ← DWMリダイレクト有効
              ↓
          CreateWindowExW() に渡される
              ↓
          通常のウィンドウが作成される（DirectComposition非対応）
```

### 3.2 変更後のフロー

```
Window エンティティ作成
  ↓
window_system.rs: create_windows()
  ├─ WindowStyle コンポーネントあり
  │   └─ 指定された値を使用
  └─ WindowStyle コンポーネントなし
      └─ WindowStyle::default()
          └─ ex_style: WS_EX_NOREDIRECTIONBITMAP  ← DWMリダイレクト無効
              ↓
          CreateWindowExW() に渡される
              ↓
          DirectComposition対応ウィンドウが作成される
```

### 3.3 後方互換性の確保

従来の動作が必要な場合:
```rust
// 明示的に旧設定を指定
world.spawn((
    Window { ... },
    WindowStyle {
        style: WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        ex_style: WINDOW_EX_STYLE(0),  // 明示的に指定
    },
));
```

## 4. 実装戦略

### 4.1 実装順序

```
Phase 1: コード変更
  1. window.rs の Default 実装を変更（1行）
  
Phase 2: ビルド検証
  2. cargo build --package wintf
  
Phase 3: 動作検証
  3. cargo run --example simple_window （主要テスト）
  4. cargo run --example areka （追加検証）
  5. cargo run --example dcomp_demo （互換性検証）
```

### 4.2 テスト戦略

#### 主要テストケース: simple_window.rs
**検証内容**:
```rust
// simple_window.rs のコード（抜粋）
world.borrow_mut().world_mut().spawn((
    Window {
        title: "wintf - ECS Window 1 (will close after 5s)".to_string(),
        ..Default::default()
    },
    WindowPos { ... },
    // WindowStyle は指定されていない ← default() が使用される
));
```

**期待される動作**:
- ウィンドウが正常に作成・表示される
- DirectCompositionが使用可能な状態である
- 自動クローズ機能（5秒ごと）が正常に動作する

#### 追加検証: areka.rs
- `WindowStyle` を指定していない
- デフォルトでDirectComposition対応になる

#### 互換性検証: dcomp_demo.rs
- 既に明示的に `.WS_EX_NOREDIRECTIONBITMAP(true)` を設定
- 冗長な設定となるが、動作に影響なし

### 4.3 エラーハンドリング

| エラーシナリオ | 検出方法 | 対処法 |
|--------------|---------|--------|
| ビルドエラー | `cargo build` | インポート不足の確認 |
| ウィンドウ作成失敗 | サンプル実行 | Windows APIエラーログを確認 |
| DirectComposition非対応環境 | Windows 7以前での実行 | プラットフォーム要件の確認（Windows 10/11が対象） |

## 5. インターフェース設計

### 5.1 公開APIの変更

#### 破壊的変更
- `WindowStyle::default()` の戻り値が変わる
- デフォルトの `ex_style` が `WINDOW_EX_STYLE(0)` から `WS_EX_NOREDIRECTIONBITMAP` に変更

#### API互換性の保持
以下は変更なし:
- `WindowStyle` 構造体のフィールド
- `WindowStyle::from_hwnd()` メソッド
- 他の関連コンポーネント（`Window`, `WindowPos` など）

### 5.2 使用例の変化

#### 変更前（明示的な設定が必要）
```rust
use wintf::*;

let style = WindowStyleBuilder::new()
    .WS_OVERLAPPEDWINDOW(true)
    .WS_VISIBLE(true)
    .WS_EX_NOREDIRECTIONBITMAP(true)  // ← 必須の設定
    .build();

world.spawn((Window { ... }, style));
```

#### 変更後（デフォルトで対応）
```rust
use wintf::*;

// WindowStyle を省略するだけでDirectComposition対応
world.spawn((Window { ... }));

// または明示的にdefault()を使用
world.spawn((
    Window { ... },
    WindowStyle::default(),  // ← 自動的にWS_EX_NOREDIRECTIONBITMAPが設定される
));
```

## 6. テスト設計

### 6.1 テスト計画

#### テストレベル
- **単体テスト**: なし（デフォルト値の変更のみ）
- **統合テスト**: サンプルアプリケーションの実行で検証
- **手動テスト**: ウィンドウの表示確認

#### テストケース

| ID | テスト名 | 検証内容 | 期待結果 |
|----|---------|---------|---------|
| TC-01 | ビルド検証 | `cargo build --package wintf` | 成功 |
| TC-02 | simple_window実行 | デフォルト動作の確認 | ウィンドウ正常表示 |
| TC-03 | areka実行 | デフォルト動作の確認 | ウィンドウ正常表示 |
| TC-04 | dcomp_demo実行 | 互換性の確認 | ウィンドウ正常表示 |

### 6.2 検証コマンド

```bash
# ビルド検証
cargo build --package wintf

# 主要テスト
cargo run --example simple_window

# 追加検証
cargo run --example areka
cargo run --example dcomp_demo
```

### 6.3 検証基準

#### 成功基準
- ✅ ビルドエラーなし
- ✅ ウィンドウが正常に作成・表示される
- ✅ ウィンドウの操作（移動、リサイズ、クローズ）が正常に動作
- ✅ DirectCompositionが使用可能（透過処理、高性能描画）

#### 失敗時の対応
1. ビルドエラー → インポート不足の確認
2. ウィンドウ作成失敗 → Windows APIエラーを調査
3. 表示不良 → ウィンドウスタイルの組み合わせを確認

## 7. パフォーマンス設計

### 7.1 パフォーマンス影響

**影響なし** - 理由:
- デフォルト値の変更のみ
- ウィンドウ作成時のオーバーヘッドは変わらない
- `WS_EX_NOREDIRECTIONBITMAP` 自体はパフォーマンスコストを持たない

### 7.2 DirectCompositionによる恩恵

変更後に期待される効果:
- **ハードウェアアクセラレーション**: GPU利用による高速描画
- **スムーズな合成処理**: 透過処理のパフォーマンス向上
- **低レイテンシ**: DWMリダイレクトを経由しない直接描画

## 8. セキュリティとエラーハンドリング

### 8.1 セキュリティへの影響

**影響なし**:
- ウィンドウスタイルの変更はセキュリティに影響を与えない
- `WS_EX_NOREDIRECTIONBITMAP` は描画方法の指定であり、権限やアクセス制御とは無関係

### 8.2 エラー処理

変更によるエラー処理への影響はない:
- `CreateWindowExW()` の成功/失敗判定は既存のコードで処理済み
- `WS_EX_NOREDIRECTIONBITMAP` による追加のエラーは発生しない

## 9. 制約と前提条件

### 9.1 技術的制約

- **Windows バージョン**: Windows 8 以降（`WS_EX_NOREDIRECTIONBITMAP` の要件）
- **DirectComposition**: Windows 8 以降で利用可能
- **プロジェクト要件**: 既にWindows 10/11を対象としている

### 9.2 プロジェクト制約

- **変更の最小化**: 1行の変更に留める
- **後方互換性**: 明示的な設定で旧動作を選択可能
- **ドキュメント**: コード変更のみ（ドキュメント更新はオプション）

## 10. ドキュメント設計（オプション）

### 10.1 コードコメント（推奨）

```rust
impl Default for WindowStyle {
    /// デフォルトのウィンドウスタイルを返します。
    /// 
    /// DirectCompositionによる描画を標準とするため、
    /// `WS_EX_NOREDIRECTIONBITMAP` が設定されています。
    /// 
    /// 従来のDWMリダイレクトを使用したい場合は、
    /// 明示的に `ex_style: WINDOW_EX_STYLE(0)` を指定してください。
    fn default() -> Self {
        Self {
            style: WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            ex_style: WS_EX_NOREDIRECTIONBITMAP,
        }
    }
}
```

### 10.2 README.md への追記（推奨）

```markdown
## ウィンドウ作成

デフォルトでDirectCompositionに対応したウィンドウが作成されます。

\`\`\`rust
// WindowStyleを省略するとDirectComposition対応
world.spawn((Window { title: "My Window".to_string(), ..Default::default() }));
\`\`\`

従来のDWMリダイレクトを使用する場合は、明示的に指定してください。

\`\`\`rust
world.spawn((
    Window { ... },
    WindowStyle {
        style: WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        ex_style: WINDOW_EX_STYLE(0),
    },
));
\`\`\`
```

## 11. 検証計画

### 11.1 検証チェックリスト

```
[ ] Phase 1: コード変更
    [ ] window.rs のDefault実装を変更
    [ ] インポートを確認（追加不要）
    
[ ] Phase 2: ビルド検証
    [ ] cargo build --package wintf が成功
    [ ] 警告が増えていない
    
[ ] Phase 3: 動作検証
    [ ] simple_window.rs が正常に動作（主要テスト）
    [ ] areka.rs が正常に動作（追加検証）
    [ ] dcomp_demo.rs が正常に動作（互換性検証）
    
[ ] Phase 4: ドキュメント（オプション）
    [ ] Rustdoc コメントを追加
    [ ] README.md に記載
```

### 11.2 ロールバック手順

問題が発生した場合:
```bash
# コミット前
git restore crates/wintf/src/ecs/window.rs

# コミット後
git revert HEAD
```

## 12. 次のステップ

### 12.1 タスク分解フェーズ
`/kiro-spec-tasks dcomp-default-window`

期待されるタスク:
1. window.rs のDefault実装を変更（1行）
2. ビルド検証
3. simple_window.rs でテスト（主要）
4. areka.rs でテスト（追加）
5. dcomp_demo.rs でテスト（互換性）
6. Rustdocコメント追加（オプション）

### 12.2 実装フェーズ
`/kiro-spec-impl dcomp-default-window`

---

**設計承認待ち** - 次のコマンド: `/kiro-spec-tasks dcomp-default-window [-y]`
