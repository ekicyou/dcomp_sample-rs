# Requirements Document

## Project Description (Input)
Graphicリソース破棄時の、ECS的に安定した再初期化手法の検討・設計・実装

## Introduction

wintfライブラリにおいて、GraphicsCoreリソース（DirectComposition、Direct2D、Direct3D11デバイス）が破棄された際に、ECSアーキテクチャの整合性を保ちながら安全に再初期化する仕組みを実装します。現在の実装では、GraphicsCoreが存在しない場合の警告表示のみで、依存するコンポーネント（WindowGraphics、Visual、Surface）の状態管理と再初期化が不完全です。

この要件は、デバイスロスト、GPUリセット、ウィンドウの再作成などの状況で、アプリケーションが安定して動作し続けることを保証します。

### 検証済み技術制約

調査の結果、以下の技術制約が確認されました：

1. **Bevy ECS Resource削除検出**: `RemovedComponents<T>`はComponentのみ対応。Resource削除検出にはポーリング方式（`Option<Res<T>>`チェック）が必要。
2. **コンポーネント状態管理**: 遅延初期化パターン（Lazy Initialization Pattern）を採用。内部を`Option<T>`でラップし、`get_or_init()`で自動初期化。
3. **Changed<T>検出の制御**: `Mut<T>`は`deref_mut()`時点で自動的にChangedマークするため、`bypass_change_detection()`経由で`get_or_init()`を呼び出し、不要な変更検出を回避。`invalidate()`と実際の再初期化のみが変更を検知。

検証テスト:
- `tests/resource_removal_detection_test.rs` - Resource削除検出の動作確認（6テスト合格）
- `tests/component_state_pattern_test.rs` - 状態管理パターン比較（6テスト合格）
- `tests/lazy_reinit_pattern_test.rs` - 遅延初期化パターン検証（5テスト合格）

## Requirements

### Requirement 1: GraphicsCore破棄検知と遅延初期化パターン

**Objective:** Graphicsリソース管理者として、GraphicsCoreが破棄されたことを検知し、依存コンポーネントに遅延初期化パターンを適用したい。これにより、無効なリソースへのアクセスを防止し、必要なタイミングで自動的に再初期化する。

#### Acceptance Criteria (1-1: Resource削除検知)

1. When GraphicsCoreリソースが削除される、the Graphicsシステムはポーリング方式（`Option<Res<GraphicsCore>>`チェック）で削除を検知しなければならない
2. When GraphicsCore削除が検知される、the GraphicsシステムはすべてのWindowGraphics、Visual、Surfaceコンポーネントに対して`invalidate()`を呼び出さなければならない
3. The GraphicsCore削除検知システムは毎フレーム実行され、1フレームの遅延を許容しなければならない

#### Acceptance Criteria (1-2: 遅延初期化パターン)

1. The WindowGraphics、Visual、Surfaceコンポーネントは内部データを`Option<T>`でラップしなければならない
2. The 各コンポーネントは`invalidate()`メソッドで内部をNoneに設定しなければならない
3. The 各コンポーネントは`get_or_init()`メソッドでNone検出時に自動初期化を実行しなければならない
4. When 有効なデータが存在する、the `get_or_init()`は再初期化せずにデータ参照を返さなければならない
5. The 世代番号（generation）フィールドで初期化回数を追跡しなければならない

### Requirement 2: GraphicsCore再初期化トリガー

**Objective:** システム管理者として、GraphicsCoreを適切なタイミングで再初期化したい。これにより、グラフィックス機能を復旧させる。

#### Acceptance Criteria

1. When アプリケーションがGraphicsCore再初期化を要求する、the Graphicsシステムは新しいGraphicsCoreリソースを作成しなければならない
2. If GraphicsCore作成に失敗する、then the Graphicsシステムはエラーログを出力し、再試行可能な状態を維持しなければならない
3. When GraphicsCoreが正常に再作成される、the Graphicsシステムは依存コンポーネントの再初期化フローを開始しなければならない
4. The ensure_graphics_coreシステムはGraphicsCore不在時に自動的に初期化を試みなければならない

### Requirement 3: WindowGraphics遅延再初期化

**Objective:** ウィンドウ管理者として、GraphicsCore再作成後に既存ウィンドウのWindowGraphicsを遅延初期化したい。これにより、ウィンドウの描画機能を効率的に復旧させる。

#### Acceptance Criteria (3-1: 自動再初期化)

1. When WindowGraphicsの`get_or_init()`が呼ばれる、if 内部がNoneならば、the 新しいIDCompositionTargetとID2D1DeviceContextを生成しなければならない
2. When WindowGraphics初期化に成功する、the `get_or_init()`はデータ参照を返し、世代番号をインクリメントしなければならない
3. When WindowGraphics初期化に失敗する、the `get_or_init()`はエラーログを出力し、panicまたはデフォルト値を返さなければならない

#### Acceptance Criteria (3-2: 使用パターン)

1. The 描画システムはWindowGraphicsアクセス時に`wg.bypass_change_detection().get_or_init()`を呼び出さなければならない
2. The `bypass_change_detection()`は不要なChanged検出を回避し、実際の再初期化のみが変更をマークしなければならない
3. The `get_or_init()`は可変参照（`&mut self`）を要求し、並行アクセスを防止しなければならない
4. When 複数のエンティティが再初期化を必要とする、the システムは必要なもののみを初期化しなければならない（遅延初期化の利点）

### Requirement 4: Visual/Surface再初期化

**Objective:** 描画管理者として、WindowGraphics再作成後にVisualとSurfaceコンポーネントを再初期化したい。これにより、ビジュアルツリーと描画サーフェスを復旧させる。

#### Acceptance Criteria

1. When WindowGraphicsが再作成される、the Graphicsシステムは関連するVisualコンポーネントを無効としてマークしなければならない
2. When 無効なVisualが検出される、the Graphicsシステムは新しいIDCompositionVisual3を生成し、CompositionTargetに設定しなければならない
3. When Visualが再作成される、the Graphicsシステムは関連するSurfaceコンポーネントを無効としてマークしなければならない
4. When 無効なSurfaceが検出される、the Graphicsシステムは新しいIDCompositionSurfaceを生成し、Visualに関連付けなければならない
5. The Graphicsシステムは再初期化の順序（WindowGraphics → Visual → Surface）を保証しなければならない

### Requirement 5: ECS整合性とChanged<T>検出

**Objective:** ECSアーキテクトとして、リソース再初期化中のECS Worldの整合性を保証し、変更検出機構を活用したい。これにより、他のシステムへの影響を最小化し、効率的な更新フローを実現する。

#### Acceptance Criteria (5-1: Changed検出)

1. When コンポーネントの`invalidate()`が呼ばれる、the Bevy ECSの変更検出機構（`Changed<T>`）は変更を検知しなければならない
2. When コンポーネントの`get_or_init()`が再初期化を実行する、the Bevy ECSの変更検出機構（`Changed<T>`）は変更を検知しなければならない
3. The 依存システムは`Query<&T, Changed<T>>`で再初期化されたコンポーネントのみを効率的に処理できなければならない
4. When 有効なデータが存在し`get_or_init()`が再初期化しない、the 変更検出は発生してはならない

#### Acceptance Criteria (5-2: ECS整合性)

1. While GraphicsCore再初期化が進行中、the Graphicsシステムは並行する描画リクエストを安全に処理しなければならない
2. When コンポーネントが無効状態の時、the 描画システムは`get_or_init()`を呼び出すか処理をスキップしなければならない
3. The コンポーネントの状態遷移（Valid→Invalid→Valid）はアトミックに実行されなければならない

### Requirement 6: エラーハンドリングとログ

**Objective:** 開発者として、再初期化プロセスの詳細なログとエラー情報を取得したい。これにより、問題の診断と修正が容易になる。

#### Acceptance Criteria

1. When GraphicsCore初期化が失敗する、then the Graphicsシステムは詳細なエラー情報（HRESULT、エラーメッセージ）をログに出力しなければならない
2. When コンポーネント再初期化が失敗する、then the Graphicsシステムはエンティティ情報とエラー理由をログに出力しなければならない
3. The Graphicsシステムは再初期化の各ステップ（開始、進行中、完了、失敗）をログに記録しなければならない
4. While デバッグモード、the Graphicsシステムは無効なコンポーネントへのアクセス試行を検出し、警告を出力しなければならない
5. The Graphicsシステムは再初期化の統計情報（成功数、失敗数、所要時間）を記録しなければならない

### Requirement 7: テスト可能性と検証済みパターン

**Objective:** QAエンジニアとして、GraphicsCore再初期化機能を体系的にテストしたい。これにより、実装の正確性を検証する。

#### Acceptance Criteria (7-1: 検証済みテスト)

1. The `tests/resource_removal_detection_test.rs`はResource削除検出パターンを検証済みでなければならない（6テスト合格）
2. The `tests/component_state_pattern_test.rs`は3つの状態管理パターンを比較検証済みでなければならない（6テスト合格）
3. The `tests/lazy_reinit_pattern_test.rs`は遅延初期化パターンを検証済みでなければならない（5テスト合格）
4. The すべてのテストケースは`cargo test`で実行可能でなければならない

#### Acceptance Criteria (7-2: 本番コードテスト)

1. The テストフレームワークはGraphicsCoreの破棄と再作成を模擬できなければならない
2. The テストフレームワークはコンポーネントの状態遷移を検証できなければならない
3. The テストフレームワークは再初期化の失敗シナリオを再現できなければならない
4. The テストフレームワークは複数ウィンドウの同時再初期化を検証できなければならない
5. The 統合テストは`tests/`ディレクトリに配置され、実装完了後に実行可能でなければならない
