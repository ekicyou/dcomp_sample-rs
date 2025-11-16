# Implementation Gap Analysis

**Feature**: graphics-resource-reinitialization  
**Date**: 2025-11-16  
**Analysis Framework**: gap-analysis.md

## 分析概要

- **スコープ**: GraphicsCoreとその依存コンポーネント（WindowGraphics、Visual、Surface）の破棄検知と再初期化
- **主な課題**: ECSリソース削除の検知、コンポーネント状態管理、再初期化の順序制御
- **推奨アプローチ**: ハイブリッド（既存システム拡張 + 新規状態管理コンポーネント追加）

---

## 1. 現状調査

### 1.1 既存アセット

#### Graphicsモジュール構造
```
crates/wintf/src/ecs/graphics/
├── mod.rs              - モジュール統合
├── core.rs             - GraphicsCore リソース定義
├── components.rs       - WindowGraphics, Visual, Surface コンポーネント
├── systems.rs          - 作成システム群（create_window_graphics等）
└── command_list.rs     - 描画コマンド管理
```

#### 既存コンポーネント
- **GraphicsCore** (Resource): D3D11、DXGI、D2D、DirectWrite、DirectCompositionデバイス
- **WindowGraphics** (Component): IDCompositionTarget、ID2D1DeviceContext
- **Visual** (Component): IDCompositionVisual3
- **Surface** (Component): IDCompositionSurface

#### 既存システム
- **ensure_graphics_core**: GraphicsCore不在時に自動作成（UISetupスケジュール）
- **create_window_graphics**: WindowHandle → WindowGraphics作成（PostLayoutスケジュール）
- **create_window_visual**: WindowGraphics → Visual作成（PostLayoutスケジュール）
- **create_window_surface**: WindowGraphics + Visual → Surface作成（PostLayoutスケジュール）
- **render_surface**: Surface描画実行（Renderスケジュール）
- **commit_composition**: DirectComposition変更確定（Renderスケジュール）

#### スケジュール構造（実行順序）
```
Input → Update → PreLayout → Layout → PostLayout → UISetup → 
Draw → Render → RenderSurface → Composition → CommitComposition
```

- **UISetup**: メインスレッド固定、ensure_graphics_core実行
- **PostLayout**: マルチスレッド、Graphics系コンポーネント作成
- **Render**: マルチスレッド、描画実行

### 1.2 既存パターン

#### コンポーネントフック
- **WindowHandle**: `on_add`/`on_remove`フックでApp通知
- **Rectangle**: `on_remove`フックでリソースクリーンアップ
- bevy_ecs 0.17.2の`#[component]`マクロでライフサイクルフック実装

#### エラーハンドリング
- `windows::core::Result`を使用
- システムレベルでは`Option<Res<GraphicsCore>>`で不在チェック
- 失敗時は`eprintln!`でログ出力、処理継続（panicしない）

#### 状態管理
- 現状、コンポーネントに状態フラグなし（Valid/Invalidの概念不在）
- `Query`の`Without`フィルタで未作成エンティティを特定
- `Changed`フィルタで変更検知

### 1.3 統合ポイント

- **Schedules**: `world.resource_mut::<Schedules>()`経由でシステム登録
- **Commands**: `Commands::insert_resource`でリソース追加、`Commands::entity().insert()`でコンポーネント追加
- **Resource監視**: bevy_ecsにResourceの削除検知機構は標準装備されていない

---

## 2. 要件ギャップ分析

### 要件1: GraphicsCore破棄検知と状態管理

| 要件内容 | 現状 | ギャップ |
|---------|------|---------|
| GraphicsCore削除時に依存コンポーネントを無効化 | 削除検知機構なし | **Missing**: Resource削除イベント検知 |
| GraphicsCore不在時の処理スキップ | `Option<Res<GraphicsCore>>`で実装済み | ✅ 既存機能で対応可能 |
| コンポーネント状態追跡（Valid/Invalid） | 状態フラグなし | **Missing**: 状態管理フィールド |

**技術的制約**:
- bevy_ecsはResource削除の直接的なイベント検知をサポートしていない
- RemovedComponentsはComponentのみ対応、Resourceには非対応

**実装オプション**:
1. **ポーリング方式**: 各フレームで`Option<Res<GraphicsCore>>`をチェックし、前フレームから変化を検知
2. **明示的削除API**: GraphicsCore削除時に専用システムを呼び出し、依存コンポーネントをマーク
3. **状態フラグ追加**: コンポーネントに`is_valid: bool`フィールドを追加

### 要件2: GraphicsCore再初期化トリガー

| 要件内容 | 現状 | ギャップ |
|---------|------|---------|
| 再初期化要求受付 | `ensure_graphics_core`が自動作成 | **Gap**: 手動トリガー機構なし |
| 初期化失敗時の再試行 | 失敗時はpanic | **Gap**: エラー記録と再試行ロジック不在 |
| 再作成後の再初期化フロー開始 | 既存システムが自動実行 | ✅ 部分的に対応済み |

**制約**:
- 現在の`ensure_graphics_core`は失敗時にpanicする（アプリ終了）

**実装オプション**:
1. **Resource状態管理**: GraphicsCore再作成要求を示すResourceを導入
2. **エラーログResource**: 失敗情報を記録し、次フレームで再試行

### 要件3: WindowGraphics再初期化

| 要件内容 | 現状 | ギャップ |
|---------|------|---------|
| 無効コンポーネント特定 | 機構なし | **Missing**: 無効状態フラグ |
| WindowGraphics再作成 | 初回作成のみ対応 | **Gap**: 既存コンポーネント更新ロジック不在 |
| 再作成成功時の状態更新 | 機構なし | **Missing**: 状態フラグ更新 |
| 再初期化中のアクセス制御 | 機構なし | **Missing**: 状態遷移管理 |

**制約**:
- `create_window_graphics`は`Without<WindowGraphics>`でフィルタ（既存コンポーネントを無視）
- コンポーネント更新には削除→再挿入、または既存コンポーネント変更が必要

**実装オプション**:
1. **削除→再作成**: 無効コンポーネントを削除し、既存システムで再作成
2. **in-place更新**: コンポーネント内部フィールドを更新（WindowGraphicsを`pub`フィールドで公開）
3. **状態フラグ付き再作成**: 状態コンポーネントと組み合わせ、条件付きで再作成

### 要件4: Visual/Surface再初期化

| 要件内容 | 現状 | ギャップ |
|---------|------|---------|
| WindowGraphics再作成時のVisual無効化 | 連鎖機構なし | **Missing**: 依存関係追跡 |
| 無効Visual再作成 | 初回作成のみ | **Gap**: 既存更新ロジック不在 |
| 再初期化順序保証 | システムチェーンで部分対応 | **Gap**: 状態ベースの順序制御不在 |

**制約**:
- システム実行順序は`.after()`で制御済み
- 状態ベースの条件付き実行機構なし

### 要件5: ECS整合性の保証

| 要件内容 | 現状 | ギャップ |
|---------|------|---------|
| 再初期化中の並行処理 | マルチスレッドスケジュール対応 | ✅ 既存アーキテクチャで対応可能 |
| 無効コンポーネント使用のスキップ | 機構なし | **Missing**: 状態チェック |
| アトミックな状態遷移 | 機構なし | **Missing**: 状態管理システム |
| 状態変更通知 | 機構なし | **Gap**: イベントシステム不在 |

**制約**:
- bevy_ecsのクエリは自然にマルチスレッド対応
- 状態管理は追加実装が必要

### 要件6: エラーハンドリングとログ

| 要件内容 | 現状 | ギャップ |
|---------|------|---------|
| 詳細エラーログ | `eprintln!`で基本実装 | **Gap**: 構造化ログなし |
| ステップログ記録 | 部分的に実装 | **Gap**: 統一ログフォーマットなし |
| 統計情報記録 | 機構なし | **Missing**: メトリクスResource |

**実装オプション**:
1. **既存パターン拡張**: `eprintln!`マクロ継続使用、フォーマット統一
2. **ログResource導入**: 構造化ログをResourceに蓄積

### 要件7: テスト可能性

| 要件内容 | 現状 | ギャップ |
|---------|------|---------|
| GraphicsCore模擬 | `GraphicsCore::new()`は実デバイス作成 | **Research Needed**: モックデバイス作成可能性 |
| 状態遷移検証 | 既存テストあり（`graphics_core_ecs_test.rs`） | **Gap**: 状態管理機能追加後のテスト拡張 |
| 失敗シナリオ再現 | 機構なし | **Research Needed**: DXGI/D3D失敗シミュレーション |

**制約**:
- Windows COMオブジェクトの完全なモック化は困難
- 実デバイス依存のテストが主体

---

## 3. 実装アプローチオプション

### Option A: 既存システム拡張（最小変更）

**戦略**:
- 既存の`create_*`システムに状態チェック機能を追加
- コンポーネントに`is_valid`フィールドを追加し、無効時に再作成ロジックを実行

**変更対象ファイル**:
- `crates/wintf/src/ecs/graphics/components.rs`: 状態フィールド追加
- `crates/wintf/src/ecs/graphics/systems.rs`: 再初期化ロジック追加
- `crates/wintf/src/ecs/graphics/core.rs`: エラーハンドリング改善

**互換性**:
- ✅ 既存APIは維持（状態フィールドはデフォルトtrue）
- ✅ 既存テストは継続動作
- ❌ コンポーネント構造の変更は既存コードへ影響

**複雑性とメンテナンス性**:
- ✅ 新規ファイル不要、学習コスト低
- ❌ `systems.rs`のロジック増加（現状286行 → 400行程度）
- ❌ 状態管理ロジックが各システムに散在

**トレードオフ**:
- ✅ 実装速度が速い
- ✅ 既存パターンとの一貫性
- ❌ 将来的な拡張性が限定的（状態種類追加時にコンポーネント変更必要）
- ❌ 単一責任原則の軽微な違反（コンポーネントが状態とデータを両方保持）

---

### Option B: 新規状態管理コンポーネント作成

**戦略**:
- `GraphicsResourceState`コンポーネントを新規作成（Valid/Invalid/Reinitializing）
- 状態監視システムと再初期化システムを分離
- 既存コンポーネントは不変、状態管理を別レイヤーで実装

**新規ファイル**:
- `crates/wintf/src/ecs/graphics/state.rs`: 状態コンポーネントと遷移ロジック
- `crates/wintf/src/ecs/graphics/reinit_systems.rs`: 再初期化専用システム

**統合ポイント**:
- `Query<(&WindowGraphics, &GraphicsResourceState)>`で状態を組み合わせ取得
- `Commands::entity().insert(GraphicsResourceState::Invalid)`で状態変更
- PostLayoutスケジュールに再初期化システムを追加

**責務境界**:
- **既存コンポーネント**: 純粋なデータ保持（COMインターフェース）
- **GraphicsResourceState**: 有効性フラグと世代番号
- **reinit_systems**: 状態遷移と再作成ロジック

**トレードオフ**:
- ✅ 明確な責務分離（状態管理が独立）
- ✅ 既存コンポーネントは不変（後方互換性高）
- ✅ テストが容易（状態管理のみを単独テスト可能）
- ❌ ファイル数増加（+2ファイル）
- ❌ クエリが複雑化（状態コンポーネント追加で`Query`が長くなる）
- ⚠️ 状態とデータの同期を保証する仕組みが必要

---

### Option C: ハイブリッドアプローチ（推奨）

**戦略**:
- **Phase 1**: 状態管理コンポーネント追加（Option B）+ GraphicsCore削除検知システム新規作成
- **Phase 2**: 既存`create_*`システムを拡張し、無効状態のコンポーネント再作成に対応
- **Phase 3**: テストとログ強化

**Phase 1 - 基盤構築**:
```rust
// 新規: state.rs
pub enum ResourceState { Valid, Invalid, Reinitializing }

#[derive(Component)]
pub struct GraphicsResourceState {
    pub state: ResourceState,
    pub generation: u64,  // GraphicsCore世代番号
}

// 新規システム: detect_graphics_core_removal
// 前フレームのGraphicsCore存在状態を記録し、削除を検知
```

**Phase 2 - 既存システム拡張**:
```rust
// systems.rs拡張
pub fn create_or_reinit_window_graphics(
    query: Query<
        (Entity, &WindowHandle, Option<&WindowGraphics>, Option<&GraphicsResourceState>)
    >,
    // ...
) {
    // 既存: WindowGraphicsがない → 作成
    // 新規: GraphicsResourceStateがInvalid → 削除して再作成
}
```

**Phase 3 - 強化**:
- テストケース追加: `tests/graphics_reinit_test.rs`
- 構造化ログ: `GraphicsMetrics` Resource導入

**統合戦略**:
1. 状態管理を独立レイヤーとして追加（後方互換）
2. 既存システムは段階的に拡張（一度に全システム変更しない）
3. フィーチャーフラグで新機能を段階的有効化（optional）

**リスク緩和**:
- ✅ フェーズ分割により段階的デリバリー可能
- ✅ Phase 1完了時点でテスト・検証可能
- ✅ 問題発生時のロールバックが容易

**トレードオフ**:
- ✅ 段階的実装でリスク分散
- ✅ 新規機能と既存機能の共存
- ✅ 将来的な拡張性確保（状態種類追加が容易）
- ⚠️ 計画・管理の複雑性増加
- ⚠️ Phase 2完了まで完全な機能提供不可

---

## 4. 実装複雑性とリスク評価

### 工数見積もり: **M (Medium, 3-7日)**

**根拠**:
- Phase 1（基盤構築）: 2-3日
  - 状態コンポーネント設計・実装
  - GraphicsCore削除検知システム
  - 基本テスト作成
- Phase 2（既存システム拡張）: 2-3日
  - 4システムの拡張（create_window_graphics, create_window_visual, create_window_surface, render_surface）
  - 統合テスト
- Phase 3（強化）: 1日
  - ログ改善
  - 追加テストケース

**前提**:
- bevy_ecs 0.17.2の知識あり
- Windows COM APIの理解あり
- 既存コードベースに精通

### リスク: **Medium**

**技術リスク**:
- **Medium**: Resource削除検知の確実性
  - bevy_ecsの標準機能外のため、ポーリング方式の信頼性確認が必要
  - 対策: 早期プロトタイプで検証
  
- **Low**: 状態同期の複雑性
  - GraphicsResourceStateとコンポーネントの同期
  - 対策: 明確な状態遷移ルールと検証テスト

**統合リスク**:
- **Low**: 既存システムとの競合
  - 既存`create_*`システムは無効状態を無視するため、並行実行可能
  - 対策: システム実行順序を`.after()`で明示

**パフォーマンスリスク**:
- **Low**: 毎フレームの状態チェックオーバーヘッド
  - Queryフィルタは効率的、影響は最小
  - 対策: 必要に応じて`Changed`フィルタでスキップ

---

## 5. 設計フェーズへの推奨事項

### 優先実装アプローチ
**Option C: ハイブリッドアプローチ**を推奨

**理由**:
1. 段階的実装によるリスク低減
2. 既存コードとの共存が容易
3. 将来的な拡張性確保
4. テストとデバッグが段階的に実施可能

### 重点研究項目

#### 1. Resource削除検知の実装パターン（優先度: 高）
- **調査内容**: bevy_ecsでのResourceライフサイクル監視パターン
- **アプローチ案**:
  - 前フレーム状態記録Resource: `PrevGraphicsCoreState(bool)`
  - 毎フレーム比較: `Option<Res<GraphicsCore>>`の存在チェック
- **検証項目**: 削除→再作成の1フレーム遅延の影響範囲

#### 2. 状態遷移の詳細設計（優先度: 高）
- **状態定義**: Valid / Invalid / Reinitializing / Failed
- **遷移ルール**: どのイベントでどの状態に遷移するか
- **排他制御**: 再初期化中の並行アクセス防止

#### 3. テスト戦略（優先度: 中）
- **モック化範囲**: GraphicsCore作成のみモック vs 全COM APIモック
- **実デバイステスト**: CI環境でのWindows GPU利用可否
- **失敗シミュレーション**: デバイスロスト等の再現方法

#### 4. エラーリカバリーポリシー（優先度: 中）
- **再試行回数**: 何回まで再初期化を試行するか
- **ユーザー通知**: エラー時のUI表示方法（将来対応）
- **フォールバック**: 全失敗時の動作（アプリ終了 or 機能制限モード）

### 既知の制約と考慮事項

1. **bevy_ecs制約**: RemovedComponents<T>はResourceに非対応
2. **Windows COM**: デバイス再作成時のスレッド制約（一部APIはSTAスレッド固定）
3. **スケジュール順序**: UISetupはシングルスレッド、それ以外はマルチスレッド
4. **後方互換性**: 既存examplesが動作し続けることを保証

### 次ステップ
1. `/kiro-spec-design graphics-resource-reinitialization`で詳細設計フェーズへ
2. Phase 1のプロトタイプ実装で技術検証
3. 設計レビュー後、Phase 2以降を段階的に実装
