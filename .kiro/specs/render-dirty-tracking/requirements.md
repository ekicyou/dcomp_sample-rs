# Requirements Document

## Project Description (Input)
render_surfaceが毎フレーム実行されているが、自身と子孫に変更があったときだけの描画にできないか？

## Introduction

現在の`render_surface`システムは毎フレーム全てのWindowエンティティとその子孫を無条件に再描画しています。これは、変更が発生していない要素に対しても不要な描画コストが発生し、パフォーマンスの非効率性につながります。本要件では、自身または子孫に変更があったときのみ描画を実行する差分レンダリング（Dirty Tracking）機能を実装し、描画処理を最適化します。

## Requirements

### Requirement 1: Dirty Flag管理

**Objective:** 開発者として、変更検出機構を実装することで、不要な再描画を排除し、システムパフォーマンスを向上させたい

#### Acceptance Criteria

1. When エンティティの描画に影響するコンポーネント（`GraphicsCommandList`, `GlobalArrangement`）が変更された場合、the Render Systemは該当エンティティにDirtyマーカーを設定すること
2. When 親エンティティにDirtyマーカーが設定された場合、the Render Systemは全ての祖先Windowエンティティまで伝播してDirtyマーカーを設定すること
3. When フレームの最後に`render_surface`が完了した場合、the Render SystemはDirtyマーカーをクリアすること
4. The Render SystemはDirtyマーカーとしてECSコンポーネント型（例: `NeedsRender`）を使用し、bevy_ecsのQuery機能で効率的にフィルタリングできること

### Requirement 2: 選択的レンダリング実行

**Objective:** システムとして、Dirtyマーカーが設定されたエンティティのみを再描画することで、描画コストを削減したい

#### Acceptance Criteria

1. When `render_surface`システムが実行される場合、the Render SystemはDirtyマーカーを持つWindowエンティティのみを処理対象とすること
2. While Windowエンティティを描画している間、the Render Systemは該当Windowの全子孫を階層的に描画すること（現在の動作を維持）
3. When WindowエンティティにDirtyマーカーが存在しない場合、the Render Systemは該当Windowとその子孫の描画処理をスキップすること
4. The Render SystemはSkipされたWindow数とRendered Window数をフレームごとに診断ログに出力すること

### Requirement 3: 初期化時の強制描画

**Objective:** 開発者として、グラフィックスリソースの初期化や再初期化時に確実に描画が実行されることを保証したい

#### Acceptance Criteria

1. When Windowエンティティに`GraphicsNeedsInit`マーカーが存在する場合、the Render SystemはDirtyマーカーの有無に関わらず強制的に描画を実行すること
2. When グラフィックスリソースの再初期化（デバイスロスト回復等）が完了した場合、the Render SystemはDirtyマーカーを自動的に設定すること
3. When 新規Windowが作成された場合、the Render Systemは初回フレームで必ず描画を実行すること

### Requirement 4: 既存システムとの統合

**Objective:** 開発者として、既存のECSシステム（レイアウト、描画コマンド生成）と透過的に連携できる仕組みを提供したい

#### Acceptance Criteria

1. The Render Systemは既存の`GraphicsCommandList`コンポーネントとの互換性を維持すること
2. When レイアウトシステム（`Layout`, `PostLayout`スケジュール）が`GlobalArrangement`を更新した場合、the Render Systemは自動的にDirtyマーカーを検出すること
3. When 描画コマンド生成システム（`Draw`スケジュール）が`GraphicsCommandList`を更新した場合、the Render Systemは自動的にDirtyマーカーを検出すること
4. The Render SystemはECSの`Changed<T>`フィルタと連携し、bevy_ecsのネイティブ変更検出機構を活用すること

### Requirement 5: パフォーマンス測定とデバッグ

**Objective:** 開発者として、Dirty Tracking機能の効果を測定し、問題を診断できるようにしたい

#### Acceptance Criteria

1. The Render Systemは各フレームで以下のメトリクスを診断ログに出力すること：スキップされたWindow数、描画されたWindow数、合計Window数
2. When デバッグモードが有効な場合、the Render SystemはDirtyマーカーが設定された理由（コンポーネント変更種別）をログに出力すること
3. The Render Systemは既存の`FrameCount`リソースを使用してフレーム番号をログに含めること
4. If Dirtyマーカーの伝播処理で無限ループや異常が検出された場合、then the Render Systemはエラーログを出力し、該当フレームの処理を安全に中断すること

### Requirement 6: 段階的な導入

**Objective:** 開発者として、既存のレンダリング動作を維持しながら段階的に最適化を導入できるようにしたい

#### Acceptance Criteria

1. The Render SystemはDirty Tracking機能の有効・無効を切り替えられる設定オプションを提供すること
2. When Dirty Tracking機能が無効な場合、the Render Systemは現在の動作（毎フレーム全Window描画）を維持すること
3. When Dirty Tracking機能が有効な場合、the Render Systemは選択的レンダリングを実行すること
4. The Render Systemのデフォルト設定はDirty Tracking有効とすること
