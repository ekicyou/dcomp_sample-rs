# Requirements Document

## Project Description (Input)
Graphicリソース破棄時の、ECS的に安定した再初期化手法の検討・設計・実装

## Introduction

wintfライブラリにおいて、GraphicsCoreリソース（DirectComposition、Direct2D、Direct3D11デバイス）が破棄された際に、ECSアーキテクチャの整合性を保ちながら安全に再初期化する仕組みを実装します。現在の実装では、GraphicsCoreが存在しない場合の警告表示のみで、依存するコンポーネント（WindowGraphics、Visual、Surface）の状態管理と再初期化が不完全です。

この要件は、デバイスロスト、GPUリセット、ウィンドウの再作成などの状況で、アプリケーションが安定して動作し続けることを保証します。

### コンポーネント命名規則

現在の実装では以下のコンポーネントを使用していますが、将来的には論理情報とグラフィックスリソースを分離する予定です：

- **Visual** → 将来的に **VisualGraphics** に改名予定（現在は実質的にグラフィックスリソースのみ）
- **Surface** → 将来的に **SurfaceGraphics** に改名予定（現在は実質的にグラフィックスリソースのみ）
- **WindowGraphics** → すでに適切な命名（変更なし）

本要件定義では現在の実装に合わせて `Visual`、`Surface` という名称を使用します。

### 検証済み技術制約

調査の結果、以下の技術制約が確認されました：

1. **Bevy ECS Resource管理**: GraphicsCoreリソース自体も内部データを`Option<T>`でラップし、破棄時はNoneに設定。初期化システムはNoneを検出して初期化を実行。
2. **2マーカーコンポーネント方式**: 拡張性とパフォーマンスを両立するため、静的マーカー`HasGraphicsResources`（リソース使用宣言）と動的マーカー`GraphicsNeedsInit`（初期化状態）を使用。内部データを`Option<T>`でラップし、初期化システムは`Query<&mut T, With<GraphicsNeedsInit>>`で対象を絞る。
3. **一括マーキングパターン**: GraphicsCore初期化システムが、初期化完了時に`Query<Entity, With<HasGraphicsResources>>`で全対象エンティティを取得し、`Commands::entity(entity).insert(GraphicsNeedsInit)`で一括追加。これにより新Widgetコンポーネント追加時もシステム変更不要。
4. **ECSスケジューリング最適化**: 初期化システムは`Query<&mut T, With<GraphicsNeedsInit>>`、参照システムは`Query<&T, Without<GraphicsNeedsInit>>`を使用し、読み取り専用アクセスを最大化することでBevy ECSの並列実行最適化を活用（Archetype-levelフィルタリング）。

検証テスト:
- `tests/resource_removal_detection_test.rs` - Resource削除検出の動作確認（6テスト合格）
- `tests/component_state_pattern_test.rs` - 状態管理パターン比較（6テスト合格）
- `tests/lazy_reinit_pattern_test.rs` - 遅延初期化パターン検証（5テスト合格）

## Requirements

### Requirement 1: GraphicsCore破棄検知と2マーカーコンポーネントパターン

**Objective:** Graphicsリソース管理者として、GraphicsCoreが破棄されたことを検知し、依存コンポーネントに2種類のマーカーコンポーネントを使用して初期化要求を明示したい。静的マーカー（HasGraphicsResources）でリソース使用を宣言し、動的マーカー（GraphicsNeedsInit）で初期化状態を管理することで、拡張性とパフォーマンスを両立する。

#### Acceptance Criteria (1-1: GraphicsCore破棄検知)

1. The GraphicsCoreリソースは内部データを`Option<T>`でラップしなければならない
2. When グラフィックデバイスロストなどの破棄イベントが検出される、the 破棄検出システムはGraphicsCoreの`invalidate()`を呼び出し、内部をNoneに設定しなければならない
3. When GraphicsCoreが無効化される、the システムはすべてのWindowGraphics、Visual、Surfaceコンポーネントに対して`invalidate()`を呼び出さなければならない
4. The GraphicsCore破棄検知システムは毎フレーム実行され、1フレームの遅延を許容しなければならない

#### Acceptance Criteria (1-2: 2マーカーコンポーネントパターンと一括マーキング)

1. The WindowGraphics、Visual、Surfaceコンポーネントは内部データを`Option<T>`でラップしなければならない
2. The 各コンポーネント（GraphicsCore含む）は`invalidate()`メソッドで内部をNoneに設定しなければならない
3. The 静的マーカーコンポーネント（`HasGraphicsResources`）を定義し、グラフィックスリソースを使用する全エンティティに永続的に付与しなければならない
4. The 動的マーカーコンポーネント（`GraphicsNeedsInit`）を定義し、初期化が必要な状態を示さなければならない
5. When GraphicsCore初期化システムが初期化を完了する、the システムは`Query<Entity, With<HasGraphicsResources>>`で全対象エンティティを取得し、`Commands::entity(entity).insert(GraphicsNeedsInit)`で一括マーキングしなければならない
6. When 各エンティティのグラフィックスコンポーネント初期化が完了する、the クリーンアップシステムは`GraphicsNeedsInit`マーカーを削除しなければならない
7. The 世代番号（generation）フィールドで初期化回数を追跡しなければならない
8. The 新しいWidgetタイプ（ButtonGraphics、TextGraphicsなど）追加時、spawn時に`HasGraphicsResources`マーカーを付与するだけで自動的に再初期化フローに組み込まれなければならない

### Requirement 2: GraphicsCore初期化システム

**Objective:** システム管理者として、GraphicsCoreを適切なタイミングで初期化・再初期化したい。これにより、グラフィックス機能を確立・復旧させる。

#### Acceptance Criteria

1. The init_graphics_coreシステムは毎フレーム実行され、GraphicsCoreの内部が`None`の場合に初期化を実行しなければならない
2. When GraphicsCore初期化を開始する、the システムは新しいDirectComposition、Direct2D、Direct3D11デバイスを作成しなければならない
3. If GraphicsCore作成に失敗する、then the システムはエラーログを出力し、内部をNoneのまま維持して次フレームで再試行可能にしなければならない
4. When GraphicsCore初期化が正常に完了する、the システムは`Query<Entity, With<HasGraphicsResources>>`で全エンティティを取得し、`Commands::entity(entity).insert(GraphicsNeedsInit)`で一括マーキングしなければならない
5. When 一括マーキングが完了する（apply_deferred後）、the 各コンポーネント初期化システムが順次実行されなければならない
6. The init_graphics_coreシステムはPostLayoutスケジュールの最初に実行され、すべてのコンポーネント初期化システムより前でなければならない

### Requirement 3: WindowGraphics初期化システム

**Objective:** ウィンドウ管理者として、WindowGraphicsの新規作成と再初期化を統一的に処理する専用システムを実装したい。これにより、通常の初期化フローと再初期化フローを一元管理し、ウィンドウの描画機能を効率的に復旧させる。

#### Acceptance Criteria (3-1: 統合初期化システム)

1. The `init_window_graphics`システムは新規作成と再初期化の両方を処理しなければならない
2. When WindowHandleが存在しWindowGraphicsが存在しない、the システムは新しいWindowGraphicsを作成しなければならない
3. When `GraphicsNeedsInit`マーカーが存在しWindowGraphicsが無効状態、the システムは既存WindowGraphicsを再初期化しなければならない
4. When 再初期化が完了する、the システムは新しいIDCompositionTargetとID2D1DeviceContextを生成し、世代番号をインクリメントしなければならない
5. The システムは`is_valid()`でWindowGraphicsの有効性をチェックし、有効ならスキップしなければならない
6. When 初期化が失敗する、the システムはエラーログを出力し、マーカーを保持して次フレームで再試行可能にしなければならない

#### Acceptance Criteria (3-2: 参照システムとの分離)

1. The 初期化システムは`Query<&mut WindowGraphics, With<GraphicsNeedsInit>>`で初期化が必要なエンティティのみを取得しなければならない
2. The 描画システムは`Query<&WindowGraphics, Without<GraphicsNeedsInit>>`で読み取り専用アクセスし、初期化済みエンティティのみを処理しなければならない
3. The Bevy ECSは初期化システムと描画システムを並列実行できなければならない（異なるエンティティセットを操作）
4. The `is_valid()`メソッドで内部データの有効性をチェックし、無効なら処理をスキップしなければならない

### Requirement 4: Visual/Surface初期化システム

**Objective:** 描画管理者として、VisualとSurfaceコンポーネントの新規作成と再初期化を統一的に処理する専用システムを実装したい。これにより、ビジュアルツリーと描画サーフェスを効率的に管理する。

#### Acceptance Criteria (4-1: Visual初期化システム)

1. The `init_window_visual`システムは新規作成と再初期化の両方を処理しなければならない
2. When WindowGraphicsが存在しVisualが存在しない、the システムは新しいVisualを作成しなければならない
3. When `GraphicsNeedsInit`マーカーが存在しVisualが無効状態、the システムは既存Visualを再初期化しなければならない
4. When Visual初期化が完了する、the システムは新しいIDCompositionVisual3を生成し、CompositionTargetに設定しなければならない
5. The システムは`Query<&Visual, Without<GraphicsNeedsInit>>`による読み取り専用アクセスを可能にしなければならない
6. The システムは`is_valid()`でVisualの有効性をチェックし、有効ならスキップしなければならない
7. The システムはWindowGraphicsが無効な場合、初期化をスキップしなければならない（依存関係の保証）

#### Acceptance Criteria (4-2: Surface初期化システム)

1. The `init_window_surface`システムは新規作成と再初期化の両方を処理しなければならない
2. When WindowGraphicsとVisualが存在しSurfaceが存在しない、the システムは新しいSurfaceを作成しなければならない
3. When `GraphicsNeedsInit`マーカーが存在しSurfaceが無効状態、the システムは既存Surfaceを再初期化しなければならない
4. When Surface初期化が完了する、the システムは新しいIDCompositionSurfaceを生成し、Visualに関連付けなければならない
5. The システムは`Query<&Surface, Without<GraphicsNeedsInit>>`による読み取り専用アクセスを可能にしなければならない
6. The システムは`is_valid()`でSurfaceの有効性をチェックし、有効ならスキップしなければならない
7. The システムはWindowGraphicsまたはVisualが無効な場合、初期化をスキップしなければならない（依存関係の保証）

#### Acceptance Criteria (4-3: 初期化順序制御とクリーンアップ)

1. The init_graphics_coreシステムはPostLayoutスケジュールで最初に実行され、初期化完了時に全HasGraphicsResourcesエンティティへ`GraphicsNeedsInit`を一括追加しなければならない
2. The init_window_graphicsシステムはinit_graphics_coreの後に実行されなければならない（`.after(init_graphics_core)`）
3. The init_window_visualシステムはinit_window_graphicsの後に実行されなければならない（`.after(init_window_graphics)`）
4. The init_window_surfaceシステムはinit_window_visualの後に実行されなければならない（`.after(init_window_visual)`）
5. The cleanup_graphics_needs_initシステムはinit_window_surfaceの後に実行されなければならない（`.after(init_window_surface)`）
6. The cleanup_graphics_needs_initシステムは、各エンティティのWindowGraphics、Visual、Surfaceすべてが有効な場合のみ`GraphicsNeedsInit`マーカーを削除しなければならない
7. The 描画システムはRenderスケジュールで実行され、すべての初期化システムが完了した後でなければならない

### Requirement 5: ECS整合性とChanged<T>検出

**Objective:** ECSアーキテクトとして、リソース再初期化中のECS Worldの整合性を保証し、変更検出機構を活用したい。これにより、他のシステムへの影響を最小化し、効率的な更新フローを実現する。

#### Acceptance Criteria (5-1: Changed検出とマーカー)

1. When コンポーネントの`invalidate()`が呼ばれる、the Bevy ECSの変更検出機構（`Changed<T>`）は変更を検知しなければならない
2. When 初期化システムがコンポーネントを再初期化する、the Bevy ECSの変更検出機構（`Changed<T>`）は変更を検知しなければならない
3. The 依存システムは`Query<&T, Changed<T>>`で再初期化されたコンポーネントのみを効率的に処理できなければならない
4. The マーカーコンポーネントの追加・削除も`Changed`として検出され、依存システムが反応できなければならない

#### Acceptance Criteria (5-2: ECS整合性とスケジューリング最適化)

1. While GraphicsCore再初期化が進行中、the Graphicsシステムは並行する描画リクエストを安全に処理しなければならない
2. When コンポーネントが無効状態の時（マーカーあり）、the 描画システムは`Without<GraphicsNeedsInit>`フィルタで自動的にスキップしなければならない
3. The 初期化システム（`Query<&mut T, With<GraphicsNeedsInit>>`）と参照システム（`Query<&T, Without<GraphicsNeedsInit>>`）は異なるエンティティセットを操作し、Bevy ECSが並列実行できなければならない
4. The コンポーネントの状態遷移（Valid→Invalid→Valid）はアトミックに実行されなければならない
5. The マーカーコンポーネントの追加・削除はCommandsバッファ経由で実行され、apply_deferred()後に反映されなければならない
6. The `HasGraphicsResources`マーカーはエンティティのspawn時に付与され、エンティティのライフタイム全体で永続しなければならない
7. The `GraphicsNeedsInit`マーカーは各エンティティのグラフィックスコンポーネント初期化が完了するまで保持されなければならない
8. The 新しいWidgetタイプ追加時、既存の初期化システム・クリーンアップシステムの変更なしに`HasGraphicsResources`マーカーで自動的に再初期化フローに統合されなければならない（Archetype-levelクエリ最適化活用）

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
