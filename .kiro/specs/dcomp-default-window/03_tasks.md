# Tasks: ウィンドウ作成時のデフォルトでDirectCompositionを前提としたDefaultパラメーターとする

**Status**: tasks_defined  
**Updated**: 2025-11-14  
**Feature**: dcomp-default-window

## タスク概要

このリファクタリングは5つのタスクに分解される。非常にシンプルな変更（1行のみ）で、主要な動作検証は `simple_window.rs` で行う。`areka.rs` はビルド確認のみ（実行テストは不要）。

## タスクリスト

### Task 1: window.rs のデフォルト実装を変更
**ID**: TASK-001  
**優先度**: 高  
**依存関係**: なし  
**推定時間**: 1分

#### 説明
`crates/wintf/src/ecs/window.rs` の `impl Default for WindowStyle` を変更し、`ex_style` フィールドに `WS_EX_NOREDIRECTIONBITMAP` を設定する。

#### 実装手順
ファイル: `crates/wintf/src/ecs/window.rs`  
行番号: 132

```rust
// 変更前
ex_style: WINDOW_EX_STYLE(0),

// 変更後
ex_style: WS_EX_NOREDIRECTIONBITMAP,
```

#### 技術的詳細
- `WS_EX_NOREDIRECTIONBITMAP` は既にインポート済み（`use windows::Win32::UI::WindowsAndMessaging::*;`）
- 追加のインポートは不要
- 変更は1行のみ

#### 成果物
- `crates/wintf/src/ecs/window.rs` - 1行変更

#### 検証方法
```rust
// 変更後の実装を確認
impl Default for WindowStyle {
    fn default() -> Self {
        Self {
            style: WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            ex_style: WS_EX_NOREDIRECTIONBITMAP,  // ← この行を確認
        }
    }
}
```

#### 完了条件
- [ ] `ex_style: WS_EX_NOREDIRECTIONBITMAP,` に変更されている
- [ ] 他の行は変更されていない
- [ ] インポート文に変更がない

---

### Task 2: ビルド検証
**ID**: TASK-002  
**優先度**: 高  
**依存関係**: TASK-001  
**推定時間**: 1-2分

#### 説明
`cargo build` を実行し、コードが正しくコンパイルされることを確認する。

#### 実装手順
```bash
cargo build --package wintf
```

#### 成果物
- ビルド成功の確認

#### 検証方法
- ビルドが成功すること（exit code 0）
- 新しい警告が追加されていないこと
- コンパイルエラーが発生しないこと

#### 完了条件
- [ ] `cargo build --package wintf` が成功する
- [ ] 警告数が変わっていない（既存の2つのみ）
- [ ] `WS_EX_NOREDIRECTIONBITMAP` が正しく認識される

---

### Task 3: simple_window.rs で動作検証（主要テスト）
**ID**: TASK-003  
**優先度**: 高  
**依存関係**: TASK-002  
**推定時間**: 2-3分

#### 説明
`examples/simple_window.rs` を実行し、デフォルトでDirectComposition対応ウィンドウが作成されることを確認する。

#### 実装手順
```bash
cargo run --example simple_window
```

#### 検証内容
`simple_window.rs` は以下の特性を持つ:
- `WindowStyle` コンポーネントを明示的に指定していない
- したがって `WindowStyle::default()` が使用される
- 2つのウィンドウを作成（5秒ごとに1つずつクローズ）

#### 期待される動作
- ✅ ウィンドウが正常に作成・表示される
- ✅ 2つのウィンドウが表示される
  - "wintf - ECS Window 1 (will close after 5s)" at (100, 100), 800x600
  - "wintf - ECS Window 2" at (950, 150), 600x400
- ✅ 5秒後に最初のウィンドウがクローズする
- ✅ さらに5秒後に2つ目のウィンドウがクローズする
- ✅ ウィンドウの操作（移動、リサイズ）が正常に動作
- ✅ DirectCompositionが使用可能な状態（ビジュアル品質の確認）

#### 成果物
- 動作確認完了

#### 検証方法
目視確認:
- ウィンドウが正常に表示される
- タイトルバーとクライアント領域が正しく表示される
- 自動クローズ機能が動作する

コンソール出力確認:
```
[Test] Auto-close timer started. Will close windows every 5 seconds.
[Test] Two windows created. Windows will close every 5 seconds.
...
[Test] Closing one window (remaining: 2)...
```

#### 完了条件
- [ ] ウィンドウが正常に表示される
- [ ] 自動クローズ機能が動作する
- [ ] エラーやクラッシュが発生しない
- [ ] DirectComposition対応ウィンドウとして作成される

---

### Task 4: areka.rs でビルド確認（コンパイル検証のみ）
**ID**: TASK-004  
**優先度**: 低  
**依存関係**: TASK-003  
**推定時間**: 1分

#### 説明
`examples/areka.rs` がビルドできることを確認する。**実行テストは不要**（areka.rsは作り直しが必要なため）。

#### 実装手順
```bash
# ビルドのみ確認（実行しない）
cargo build --example areka
```

#### 検証内容
- `areka.rs` がコンパイルエラーなくビルドできること
- 実行テストは不要（アプリケーション自体が未完成）

#### 期待される動作
- ✅ ビルドが成功する
- ✅ コンパイルエラーが発生しない

#### 成果物
- ビルド成功の確認

#### 検証方法
```bash
cargo build --example areka
# 実行はしない
```

#### 完了条件
- [ ] `cargo build --example areka` が成功する
- [ ] コンパイルエラーが発生しない
- [ ] **実行テストは不要**

---

### Task 5: dcomp_demo.rs で互換性検証
**ID**: TASK-005  
**優先度**: 中  
**依存関係**: TASK-004  
**推定時間**: 1-2分

#### 説明
`examples/dcomp_demo.rs` を実行し、明示的に `WS_EX_NOREDIRECTIONBITMAP` を設定している既存コードとの互換性を確認する。

#### 実装手順
```bash
cargo run --example dcomp_demo
```

#### 検証内容
`dcomp_demo.rs` は以下の特性を持つ:
- 既に明示的に `.WS_EX_NOREDIRECTIONBITMAP(true)` を設定している
- 変更後は冗長な設定となるが、動作に影響しないはず

#### 期待される動作
- ✅ アプリケーションが正常に起動する
- ✅ DirectCompositionデモが正常に動作する
- ✅ 冗長な設定があっても問題なく動作する

#### 成果物
- 互換性確認完了

#### 検証方法
- アプリケーションが起動する
- DirectCompositionベースの描画が正常に行われる
- 既存の動作が維持される

#### 完了条件
- [ ] アプリケーションが起動する
- [ ] DirectCompositionデモが正常に動作する
- [ ] エラーやクラッシュが発生しない
- [ ] 既存の動作が維持される

---

## タスク実行順序

```
TASK-001: window.rs のデフォルト実装変更
    ↓
TASK-002: ビルド検証
    ↓
TASK-003: simple_window.rs テスト（主要）
    ↓
TASK-004: areka.rs ビルド確認（コンパイルのみ）
    ↓
TASK-005: dcomp_demo.rs テスト（互換性）
```

## リスク管理

### 高リスクポイント
1. **ウィンドウ作成失敗**: DirectComposition対応環境でない場合
   - 対策: Windows 10/11が前提（既に満たしている）
   
2. **サンプル動作不良**: 予期しない副作用
   - 対策: simple_window.rs（主要）とdcomp_demo.rs（互換性）でテスト
   - areka.rsはコンパイル確認のみ（実行テストは不要）
   - 軽減策: 1行の変更なので影響範囲は限定的

3. **既存コードとの競合**: 明示的な設定との重複
   - 対策: dcomp_demo.rs で互換性を検証
   - 軽減策: 冗長な設定は問題ない（上書きされるだけ）

### ロールバック手順
```bash
# コミット前
git restore crates/wintf/src/ecs/window.rs

# または手動で元に戻す
ex_style: WINDOW_EX_STYLE(0),
```

## 検証チェックリスト

### 変更前の確認
- [ ] 現在のコードが正常にビルドできる
- [ ] 現在のサンプルが正常に動作する
- [ ] Gitの作業ディレクトリがクリーン（または意図的な変更のみ）

### 変更後の確認
- [ ] 1つのファイルのみが変更されている（window.rs）
- [ ] 変更は1行のみ
- [ ] `cargo build --package wintf` が成功
- [ ] `simple_window.rs` が正常に動作（主要検証）
- [ ] `areka.rs` がビルドできる（コンパイル確認のみ）
- [ ] `dcomp_demo.rs` が正常に動作（互換性検証）
- [ ] 警告数が増えていない

### コミット前の確認
- [ ] `git status` で変更内容を確認
- [ ] `git diff` で差分が意図通り（1行のみ）
- [ ] コミットメッセージが明確

## 推定総所要時間

- **タスク実行**: 5-10分
- **検証**: 5-10分
- **合計**: 10-20分

## オプショナルタスク（実装後に実施可能）

以下は必須ではないが、推奨されるタスク:

### Task 6 (オプション): Rustdoc コメント追加
**説明**: `WindowStyle::default()` にDocコメントを追加し、DirectCompositionがデフォルトであることを明記する。

```rust
impl Default for WindowStyle {
    /// デフォルトのウィンドウスタイルを返します。
    /// 
    /// DirectCompositionによる描画を標準とするため、
    /// `WS_EX_NOREDIRECTIONBITMAP` が設定されています。
    fn default() -> Self {
        Self {
            style: WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            ex_style: WS_EX_NOREDIRECTIONBITMAP,
        }
    }
}
```

### Task 7 (オプション): README.md 更新
**説明**: README.md にDirectCompositionがデフォルトであることを記載する。

## 次のステップ

タスク完了後:
```bash
# 実装フェーズへ進む
/kiro-spec-impl dcomp-default-window
```

または、手動実装の場合:
1. TASK-001から順次実行
2. 各タスクの完了条件を確認
3. 最終的にビルドとテストを実行
4. Gitコミット

---

**タスク定義完了** - 次のコマンド: `/kiro-spec-impl dcomp-default-window`
