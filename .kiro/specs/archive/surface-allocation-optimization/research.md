# Research & Design Decisions: surface-allocation-optimization

## Summary
- **Feature**: `surface-allocation-optimization`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  - `sync_surface_from_arrangement`と`deferred_surface_creation_system`の二重経路が存在
  - `RemovedComponents<T>`パターンは既存コードベースで使用実績あり
  - `GlobalArrangement.bounds`は物理ピクセルサイズ（DPIスケール適用済み）

## Research Log

### Surface生成システムの現状分析
- **Context**: 二重経路による不要なSurface作成の問題を調査
- **Sources Consulted**: 
  - `crates/wintf/src/ecs/graphics/systems.rs`
  - `crates/wintf/src/ecs/world.rs`（スケジュール定義）
- **Findings**:
  - `sync_surface_from_arrangement`: `Changed<Arrangement>`で発火、GraphicsCommandList有無を確認せず
  - `deferred_surface_creation_system`: `With<GraphicsCommandList>, Without<SurfaceGraphics>`で発火
  - 両システムが独立して動作し、重複作成の可能性あり
- **Implications**: `sync_surface_from_arrangement`を廃止し、`deferred_surface_creation_system`に一本化

### RemovedComponents検出パターン
- **Context**: GraphicsCommandList削除時のSurface解放処理の実装方法を調査
- **Sources Consulted**:
  - `crates/wintf/src/ecs/common/tree_system.rs`
  - `crates/wintf/src/ecs/layout/systems.rs`
  - bevy_ecs公式ドキュメント
- **Findings**:
  - `RemovedComponents<ChildOf>`パターンが既存コードで使用されている
  - `orphaned.read()`でイテレート可能
  - 同一スケジュール内で検出・処理可能
- **Implications**: `RemovedComponents<GraphicsCommandList>`で削除検出が可能

### GlobalArrangement.boundsのサイズ単位
- **Context**: Surfaceサイズ計算の単位（論理ピクセル vs 物理ピクセル）を確認
- **Sources Consulted**:
  - `crates/wintf/src/ecs/layout/arrangement.rs`
  - ユーザー確認（2025-11-28）
- **Findings**:
  - `Arrangement.scale`にDPIスケールが含まれる（WindowレベルでDPIを受け取り伝播）
  - `GlobalArrangement.bounds`はスケール適用後 = 物理ピクセルサイズ
  - `bounds.right - bounds.left`, `bounds.bottom - bounds.top`でサイズ取得
- **Implications**: R3のAC-1,AC-2は現状の設計で対応可能

### DirectComposition Surfaceの制約
- **Context**: Surface更新・削除時の制約を調査
- **Sources Consulted**:
  - Microsoft DirectComposition公式ドキュメント
  - 既存コードの`SetContent`呼び出し
- **Findings**:
  - Surfaceはリサイズ不可、再作成のみ
  - `insert()`で上書きすればCOMオブジェクトは自動回収（Dropトレイト）
  - Surface削除時は`SetContent(null)`でVisualからコンテンツを解除可能
- **Implications**: Surface再作成は`commands.entity(entity).insert(new_surface)`で対応

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| **A: 既存拡張** | `deferred_surface_creation_system`を拡張、`sync_surface_from_arrangement`廃止 | 最小変更、既存パターン踏襲 | 既存テストへの影響 | **推奨** |
| B: 新統合システム | `surface_lifecycle_system`新規作成 | 明確な責務分離 | 新ファイル追加、変更規模大 | 過剰 |
| C: ハイブリッド | 段階的移行 | リスク分散 | 一時的複雑化 | 不要 |

## Design Decisions

### Decision: `sync_surface_from_arrangement`廃止
- **Context**: 二重経路によるVRAM浪費の解消
- **Alternatives Considered**:
  1. 両システム維持（GraphicsCommandList有無チェック追加）
  2. `sync_surface_from_arrangement`廃止、`deferred_surface_creation_system`一本化
- **Selected Approach**: Option 2（一本化）
- **Rationale**: 
  - Surface生成トリガーを`GraphicsCommandList`存在のみに統一
  - 保守性向上（ロジック集約）
- **Trade-offs**: 既存テストの更新が必要
- **Follow-up**: スケジュール定義（`world.rs`）の更新

### Decision: Surface削除システムの新設
- **Context**: GraphicsCommandList削除時のSurface解放（R1 AC-3,AC-4）
- **Alternatives Considered**:
  1. on_removeフックで対応
  2. 専用システム`cleanup_surface_on_commandlist_removed`新設
- **Selected Approach**: Option 2（専用システム）
- **Rationale**:
  - `RemovedComponents<T>`パターンが既存で使用されている
  - システムとしてスケジュール管理可能
  - ログ出力・デバッグが容易
- **Trade-offs**: システム追加によるスケジュール複雑化
- **Follow-up**: Drawスケジュールに配置

### Decision: サイズ計算を`GlobalArrangement.bounds`に変更
- **Context**: DPIスケール対応（R3）
- **Alternatives Considered**:
  1. `Arrangement.size`（論理サイズ）+ 手動スケール計算
  2. `GlobalArrangement.bounds`（物理ピクセル）直接使用
- **Selected Approach**: Option 2
- **Rationale**:
  - `bounds`は既にスケール適用済み
  - 計算ロジックの重複回避
- **Trade-offs**: `GlobalArrangement`への依存追加
- **Follow-up**: `deferred_surface_creation_system`のクエリ変更

## Risks & Mitigations
- **Surface生成タイミングの変化** — 既存サンプル（areka.rs）での動作確認
- **DPIスケール計算ミス** — `ceil()`で切り上げ、境界ケーステスト
- **デバイスロスト対応漏れ** — 既存パターン（`HasGraphicsResources`）踏襲

## References
- [bevy_ecs RemovedComponents](https://docs.rs/bevy_ecs/latest/bevy_ecs/removal_detection/struct.RemovedComponents.html) — 削除検出API
- [DirectComposition](https://learn.microsoft.com/en-us/windows/win32/directcomp/directcomposition-portal) — Surface制約
