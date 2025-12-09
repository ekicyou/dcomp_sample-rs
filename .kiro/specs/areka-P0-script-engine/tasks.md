# Implementation Tasks: areka-P0-script-engine

## Overview

Pasta DSL スクリプトエンジンの実装タスク。要件定義とデザインに基づき、Phase順に実装を進める。

**推定総工数**: 40-50時間
**並行実装可能**: 一部のサブタスク（P マーク付き）

---

## Task 1: Foundation（基盤構築）

pasta crate の基盤を構築する。

### 1.1 プロジェクト構造の作成 (P)

**Description**: pasta crate のディレクトリ構造と Cargo.toml を作成する。モジュール階層（parser, transpiler, runtime, ir, stdlib）を定義し、依存関係（rune, thiserror, pest, glob）を設定する。

**Requirements**: 1.1

### 1.2 PastaError の実装 (P)

**Description**: thiserror を使用してエラー型を定義する。ParseError, LabelNotFound, NameConflict, RuneError, IoError の各バリアントを実装し、エラーメッセージのフォーマットとソース情報の付与を行う。

**Requirements**: NFR-2.1, NFR-2.2, NFR-2.3

### 1.3 ScriptEvent IR 型の定義 (P)

**Description**: スクリプトイベント中間表現を定義する。Talk, Wait, ChangeSpeaker, ChangeSurface, BeginSync, SyncPoint, EndSync, Error, FireEvent の各バリアントと ContentPart 型を実装する。

**Requirements**: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7

---

## Task 2: Parser（構文解析）

pest を使用した DSL パーサーを実装する。

### 2.1 pest 文法定義の作成

**Description**: Pasta DSL の PEG 文法を定義する。スクリプト、ラベル定義、発言行、さくらスクリプトエスケープ、同期セクション、変数参照、制御構文のルールを記述する。

**Requirements**: 1.1, 1.2, 1.3, 1.4, 1.5, 3.1, 3.2

### 2.2 PastaAst 型の定義

**Description**: AST のノード型を定義する。Script, Label, Statement, SyncSection, Expression, Variable 等の構造体と enum を実装する。

**Requirements**: 1.2, 1.3, 1.4, 1.5

### 2.3 PastaParser の実装

**Description**: pest パーサーを実装し、パース結果を AST に変換する。エラー発生時は行番号とカラム情報を含む ParseError を返す。

**Requirements**: 1.1, 1.2, NFR-2.3

### 2.4 パーサー単体テストの作成

**Description**: 各構文要素のパーステストを作成する。正常ケースとエラーケースの両方をカバーし、エラーメッセージに行番号が含まれることを検証する。

**Requirements**: 1.1, 1.2, 1.3, 1.4, 1.5, NFR-2.3

---

## Task 3: Transpiler（コード変換）

AST から Rune コードへの変換を実装する。

### 3.1 Transpiler の基本実装

**Description**: AST を Rune コードに変換するトランスパイラを実装する。ラベルを Rune 関数に、発言文を emit 関数呼び出しに変換する。

**Requirements**: 1.2, 5.1, 5.2

### 3.2 変数アクセスの変換実装

**Description**: 変数参照（波カッコ記法）を Rune の変数アクセス構文に変換する。スコープ（local, global, system）に応じた適切なアクセサ呼び出しを生成する。

**Requirements**: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6

### 3.3 制御構文の変換実装

**Description**: 条件分岐、ループ、関数呼び出しを Rune の対応構文に変換する。

**Requirements**: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6

### 3.4 同期セクションの変換実装

**Description**: 同期セクションを BeginSync, SyncPoint, EndSync の emit 呼び出しに変換する。同期マーカーと ID の生成ロジックを実装する。

**Requirements**: 6.4, 6.5, 6.6, 6.7

**Status**: ⚠️ Partial - Standard Library関数として設計済み。Rune 0.14 API統合は Task 5.1で実施。

**Dependencies**: Task 4.1完了後

**Notes**: 
- 関数定義は完了（`begin_sync`, `sync_point`, `end_sync`）
- Rune Module登録は Task 5.1で対応

### 3.5 トランスパイラ単体テストの作成

**Description**: AST から Rune コードへの変換テストを作成する。生成されたコードの構文正当性と意味的正確性を検証する。

**Requirements**: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6

---

## Task 4: Runtime Core（実行時コア）

Rune 実行環境と Generator を実装する。

### 4.1 StandardLibrary の実装

**Description**: Rune の標準ライブラリモジュールを実装する。emit_text, change_speaker, change_surface, wait, 同期セクション関数等を Generator 用に yield 付きで実装する。

**Requirements**: 2.1, 2.2, 2.3, 2.4, 2.5, 8.3

**Status**: ⚠️ Partial - 全関数実装完了。Rune 0.14 Module登録APIは Task 5.1で対応。

**Implementation Notes**:
- ✅ 全9関数の実装完了（emit_text, emit_sakura_script, change_speaker, change_surface, wait, begin_sync, sync_point, end_sync, fire_event）
- ✅ ScriptEvent IRを返す純粋関数として実装
- ⚠️ Rune 0.14で`#[rune::function]`マクロが廃止されたため、Module登録は保留
- **Resolution**: Task 5.1でRune 0.14の正しいModule::function() APIを調査・実装

**Known Issue**: 
```rust
// Rune 0.13のAPI（動作しない）
#[rune::function]
fn emit_text(text: String) -> ScriptEvent { ... }

// Rune 0.14で必要な実装（要調査）
// 手動でFunction traitを実装、またはビルダーパターンを使用
```

### 4.2 ScriptGenerator の実装

**Description**: Rune Generator のラッパーを実装する。resume メソッドで次の ScriptEvent を取得し、状態を保持する機能を提供する。

**Requirements**: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6

### 4.3 VariableManager の実装

**Description**: 変数管理システムを実装する。get/set メソッド、スコープ管理（local, global, system）、型変換（String, i64, bool）を提供する。

**Requirements**: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6

### 4.4 RandomSelector trait と実装の作成 (P)

**Description**: ランダム選択のための trait を定義し、DefaultRandomSelector（実際の乱数）と MockRandomSelector（テスト用）を実装する。

**Requirements**: 6.2, 6.3

### 4.5 LabelTable の実装

**Description**: ラベル管理システムを実装する。同名ラベルのランダム選択（RandomSelector 経由）、実行履歴の記録、ラベル検索機能を提供する。

**Requirements**: 6.1, 6.2, 6.3, 7.1

### 4.6 ランタイムコンポーネントの単体テスト作成

**Description**: Generator, VariableManager, LabelTable の単体テストを作成する。MockRandomSelector を使用した決定論的テストを含める。

**Requirements**: 8.1, 8.2, 8.3, 8.4, 8.5, 4.1, 4.2, 6.1, 6.2, 6.3

---

## Task 5: Engine Integration（エンジン統合）

PastaEngine としての統合と公開 API を実装する。

### 5.1 PastaEngine の実装

**Description**: パーサー、トランスパイラ、ランタイムを統合したメインエンジンを実装する。コンストラクタで DSL をパース、トランスパイル、Rune VM 初期化を行う。

**Requirements**: 1.1, 2.1, 8.1, 8.7

**Additional Tasks** (from Task 3.4 and 4.1 completion):

#### 5.1.1 Rune 0.14 API調査と対応

**Description**: Rune 0.14の正しいModule::function()登録APIを調査し、StandardLibrary関数を登録する。

**Steps**:
1. Rune 0.14ドキュメント/examplesを確認
2. `Module::function()`の新しいAPI仕様を理解
3. 手動でFunction trait実装、またはビルダーパターンを使用
4. `crates/pasta/src/stdlib/mod.rs`の`create_module()`を完成
5. 統合テストで動作確認

**Priority**: High（PastaEngine初期化に必須）

**Estimated**: 2-3時間

**Reference Issue**: 
```
error[E0277]: the trait bound `fn() -> Result<FunctionMetaData, Error> {begin_sync}: Function<_, _>` is not satisfied
```

**Possible Solutions**:
- Rune 0.14では`module.function()`の戻り値に`.build()`を呼ぶ必要がある
- または、`module.function_meta()`など別のAPIを使用
- Rust FFI経由で直接登録する方法も検討

### 5.2 execute_label メソッドの実装

**Description**: 指定ラベルを実行し ScriptEvent のイテレータを返すメソッドを実装する。同名ラベルからのランダム選択を含む。

**Requirements**: 6.1, 6.2, 6.3, 8.1, 8.2

### 5.3 resume メソッドの実装

**Description**: Generator を一歩進めて次の ScriptEvent を返すメソッドを実装する。Completed 状態の判定を含む。

**Requirements**: 8.3, 8.4, 8.5, 8.6

### 5.4 チェイントーク（連続 yield）のサポート

**Description**: 単一の発言行から複数の ScriptEvent を順次 yield する機能を実装する。

**Requirements**: 8.8

### 5.5 Drop trait による永続化の実装

**Description**: エンジン破棄時に変数とラベルキャッシュを永続化する機能を実装する。

**Requirements**: 4.6

### 5.6 エンジン統合テストの作成

**Description**: PastaEngine の公開 API を使用した統合テストを作成する。ラベル実行から ScriptEvent 生成までのパイプライン全体を検証する。

**Requirements**: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 8.1, 8.2, 8.3

---

## Task 6: さくらスクリプト互換性

さくらスクリプトエスケープの処理を実装する。

### 6.1 さくらスクリプトエスケープのパース

**Description**: DSL 内のバックスラッシュエスケープシーケンスをパースし、ContentPart::SakuraScript として AST に保持する機能を実装する。

**Requirements**: 3.1, 3.2, 3.3, 3.4, 3.5

### 6.2 エスケープシーケンスの IR 出力

**Description**: パースしたさくらスクリプトエスケープを ScriptEvent::Talk の content に ContentPart::SakuraScript として含める機能を実装する。

**Requirements**: 3.5, 3.6

### 6.3 さくらスクリプト互換性テストの作成

**Description**: 各種エスケープシーケンス（表情、ウェイト、改行等）のパースと IR 出力をテストする。

**Requirements**: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6

---

## Task 7: イベントハンドリング

イベント発火と登録の仕組みを実装する。

### 7.1 イベントラベル命名規則の実装

**Description**: ラベル名によるイベント種別の識別（起動時、終了時等）を実装する。

**Requirements**: 7.1, 7.2

### 7.2 OnEvent メカニズムの実装

**Description**: 外部イベント受信時に対応ラベルを検索・実行する機能を実装する。

**Requirements**: 7.3, 7.4, 7.5

### 7.3 ScriptEvent::FireEvent の生成

**Description**: スクリプトからイベント発火を要求する ScriptEvent の生成を実装する。

**Requirements**: 7.5

### 7.4 イベントハンドリングテストの作成

**Description**: イベント登録、発火、ラベル実行の連携をテストする。

**Requirements**: 7.1, 7.2, 7.3, 7.4, 7.5

---

## Task 8: エラーハンドリング強化

エラー処理とユーザー通知を強化する。

### 8.1 動的エラー（ScriptEvent::Error）の実装

**Description**: ランタイムエラーを ScriptEvent::Error として yield する機能を実装する。エラーメッセージと発生位置情報を含める。

**Requirements**: NFR-2.4, NFR-2.5

### 8.2 エラーリカバリの実装

**Description**: ScriptEvent::Error 発生後も実行を継続可能にする機能を実装する。

**Requirements**: NFR-2.4

### 8.3 エラーハンドリングテストの作成

**Description**: 各種エラーケース（パースエラー、ラベル未発見、ランタイムエラー）のテストを作成する。

**Requirements**: NFR-2.1, NFR-2.2, NFR-2.3, NFR-2.4, NFR-2.5

---

## Task 9: パフォーマンス最適化

パフォーマンス要件を満たす最適化を実装する。

### 9.1 パース結果のキャッシュ実装

**Description**: パース済み AST とトランスパイル済み Rune コードをキャッシュし、再パースを回避する機能を実装する。

**Requirements**: NFR-1

### 9.2 ラベル検索の最適化

**Description**: HashMap ベースの高速ラベル検索を実装し、同名ラベルのグルーピングを事前計算する。

**Requirements**: NFR-1, 6.2

### 9.3 パフォーマンステストの作成

**Description**: 1000 行規模のスクリプト解析と 100 回連続ラベル実行のベンチマークを作成する。

**Requirements**: NFR-1

---

## Task 10: ドキュメントとサンプル

ドキュメントとサンプルスクリプトを作成する。

### 10.1 API ドキュメントの作成

**Description**: 公開 API（PastaEngine, ScriptEvent, PastaError）の rustdoc コメントを作成する。

**Requirements**: NFR-3

### 10.2 DSL 文法リファレンスの作成

**Description**: Pasta DSL の文法リファレンスドキュメントを作成する。

**Requirements**: 1.1, 1.2, 1.3, 1.4, 1.5

### 10.3 サンプルスクリプトの作成

**Description**: 基本会話、同期セクション、変数使用、制御構文のサンプルスクリプトを作成する。

**Requirements**: 1.1, 1.2, 1.3, 1.4, 1.5, 4.1, 5.1, 6.4

---

## Task 11: Rune Block サポート完成（必須機能）

インラインRuneコードブロック（ローカル関数定義）のサポートを完成させる。

### 11.1 Rune Block文法の修正

**Description**: pest文法で`rune_block`ルールを修正し、```で囲まれたRuneコード埋め込みを正しくパースできるようにする。負先読みパターンの調整が必要。

**Requirements**: 1.6 (ローカル関数定義)

**優先度**: 高（必須機能）

**推定工数**: 2-3時間

**現状**: Task 2.1で文法テストが失敗。技術的な課題は負先読みパターン`!(indent ~ rune_end)`が正しく動作していないこと。

### 11.2 Rune Block ASTノードの実装

**Description**: `rune_block`をパースしてASTに含める。Runeコード自体はパースせず、文字列として保持する。

**Requirements**: 1.6

**Dependencies**: 11.1完了後

### 11.3 Rune Block Transpilerサポート

**Description**: ASTのrune_blockノードをRuneコードに変換する。インラインコードをそのまま出力するか、外部ファイルとして分離するか選択可能にする。

**Requirements**: 1.6, 2.1

**Dependencies**: 11.2完了後、Task 3実装中に統合

### 11.4 Rune Block統合テスト

**Description**: 以下のテストを実装・パス確認：
- `grammar_tests::test_rune_block` (現在ignored)
- `grammar_diagnostic::test_rune_block_minimal` (現在ignored)
- パーサー統合テスト（Rune blockを含むスクリプト全体）
- トランスパイラテスト（Rune blockが正しく変換される）
- 実行時テスト（Runeローカル関数が呼び出せる）

**Requirements**: 1.6, NFR-2.3

**推定工数**: 1-2時間

---

## Dependencies Between Tasks

```
Task 1 (Foundation)
  ├── Task 2 (Parser) ─────┐
  │                        │
  └── Task 1.3 (IR) ──────>├── Task 3 (Transpiler) ──┐
                           │                         │
                           └── Task 4 (Runtime) ─────┤
                                                     │
                           Task 6 (Sakura Script) ───┤
                                                     │
                           Task 7 (Events) ──────────┤
                                                     │
                                                     v
                                          Task 5 (Engine Integration)
                                                     │
                                                     v
                                          Task 8 (Error Handling)
                                                     │
                                                     v
                                          Task 9 (Performance)
                                                     │
                                                     v
                                          Task 10 (Documentation)
```

---

## Summary

| Task | Name | Sub-tasks | Est. Hours |
|------|------|-----------|------------|
| 1 | Foundation | 3 | 3-4 |
| 2 | Parser | 4 | 6-8 |
| 3 | Transpiler | 5 | 6-8 |
| 4 | Runtime Core | 6 | 8-10 |
| 5 | Engine Integration | 6 | 6-8 |
| 6 | さくらスクリプト互換性 | 3 | 3-4 |
| 7 | イベントハンドリング | 4 | 4-5 |
| 8 | エラーハンドリング強化 | 3 | 2-3 |
| 9 | パフォーマンス最適化 | 3 | 3-4 |
| 10 | ドキュメントとサンプル | 3 | 2-3 |
| **Total** | | **40** | **43-57** |
