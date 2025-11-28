# Gap Analysis: surface-allocation-optimization

## 分析サマリー

- **スコープ**: Surface生成ロジックの最適化（二重経路の一本化、DPIスケール対応）
- **主要課題**: `sync_surface_from_arrangement`と`deferred_surface_creation_system`の重複
- **複雑度**: 中程度（既存パターン拡張、アーキテクチャ変更なし）
- **前提条件**: `marker-component-to-changed` ✅ 完了済み
- **推奨アプローチ**: Option A（既存コンポーネント拡張） + 部分的削除

---

## 1. 現状調査

### 1.1 関連ファイル・モジュール

| ファイル | 役割 | 変更対象 |
|----------|------|---------|
| `ecs/graphics/systems.rs` | Surface生成・描画システム | ✅ 主要変更 |
| `ecs/graphics/components.rs` | SurfaceGraphics, SurfaceGraphicsDirty | 軽微 |
| `ecs/world.rs` | スケジュール定義 | ✅ システム登録変更 |
| `ecs/layout/arrangement.rs` | GlobalArrangement定義 | 参照のみ |

### 1.2 既存アーキテクチャパターン

```
GraphicsSetup スケジュール:
  └── sync_surface_from_arrangement (Changed<Arrangement>)
       └── Surface作成 (GraphicsCommandList有無を無視)

Draw スケジュール:
  └── deferred_surface_creation_system (With<GraphicsCommandList>, Without<SurfaceGraphics>)
       └── Surface作成 (GraphicsCommandList存在時のみ)
```

**問題点**:
1. 二重経路による不要なSurface作成（VRAMの浪費）
2. サイズ取得元の不整合（`Arrangement` vs `GlobalArrangement`）
3. GraphicsCommandList削除時のSurface解放処理が未実装

### 1.3 既存コンポーネントの状態

| コンポーネント | 状態 | 備考 |
|---------------|------|------|
| `SurfaceGraphics` | ✅ 使用中 | サイズ情報を保持 |
| `SurfaceGraphicsDirty` | ✅ 使用中 | Changed方式実装済み |
| `GlobalArrangement` | ✅ 使用中 | `bounds`フィールドあり（物理ピクセル） |
| `GraphicsCommandList` | ✅ 使用中 | 描画コマンド保持 |

---

## 2. 要件充足性分析

### Requirement 1: GraphicsCommandList存在に基づくSurface生成判定

| AC | 現状 | ギャップ |
|----|------|---------|
| AC-1: CommandList追加時にSurface作成 | ✅ `deferred_surface_creation_system`で対応 | なし |
| AC-2: CommandListなしならスキップ | ❌ `sync_surface_from_arrangement`が無条件作成 | **Missing** |
| AC-3: CommandList削除時にSurface解放 | ❌ 未実装 | **Missing** |
| AC-4: 専用クリーンアップシステム | ❌ 未実装 | **Missing** |

### Requirement 2: Surface生成システムの一本化

| AC | 現状 | ギャップ |
|----|------|---------|
| AC-1: sync_surface_from_arrangement廃止 | ❌ 存在中 | **要削除** |
| AC-2: deferred_surface_creation唯一化 | △ 部分対応 | **要強化** |
| AC-3: トリガーをCommandList存在のみに | △ 部分対応 | **要確認** |
| AC-4: Arrangement変更時スキップ | ❌ 現在は作成する | **要対応** |

### Requirement 3: DPIスケール対応のSurfaceサイズ計算

| AC | 現状 | ギャップ |
|----|------|---------|
| AC-1: GlobalArrangement.boundsから計算 | ❌ Arrangement使用中 | **Missing** |
| AC-2: スケール適用後サイズ | ❌ 論理サイズ使用中 | **Missing** |
| AC-3: サイズ0ならスキップ | ✅ 実装済み | なし |
| AC-4: サイズ変更時にSurface再作成 | ❌ Arrangement基準 | **要変更** |

### Requirement 4: 既存SurfaceGraphicsとの整合性維持

| AC | 現状 | ギャップ |
|----|------|---------|
| AC-1: BeginDraw/EndDrawサイクル維持 | ✅ 問題なし | なし |
| AC-2: 子Visual階層維持 | ✅ 問題なし | なし |
| AC-3: VisualGraphics独立性維持 | ✅ 問題なし | なし |
| AC-4: デバイスロスト対応 | △ 既存実装あり | **要検証** |

### Requirement 5: 診断とデバッグ支援

| AC | 現状 | ギャップ |
|----|------|---------|
| AC-1: スキップ理由ログ | ✅ 部分実装 | 要強化 |
| AC-2: 作成ログ（物理サイズ） | ❌ 論理サイズ表示 | **要修正** |
| AC-3: デバッグ統計 | ❌ 未実装 | **Missing** |

---

## 3. 実装アプローチ選択肢

### Option A: 既存コンポーネント拡張（推奨）

**概要**: `deferred_surface_creation_system`を拡張し、`sync_surface_from_arrangement`を廃止

**変更対象**:
1. `sync_surface_from_arrangement` → 削除
2. `deferred_surface_creation_system` → `GlobalArrangement`使用に変更
3. 新システム `cleanup_surface_on_commandlist_removed` 追加
4. `world.rs` スケジュール更新

**トレードオフ**:
- ✅ 最小限のファイル変更
- ✅ 既存パターンを踏襲
- ✅ テスト影響範囲が限定的
- ❌ 既存コードの削除が必要

### Option B: 新統合システム作成

**概要**: Surface管理を統括する新システム `surface_lifecycle_system` を作成

**変更対象**:
1. 新ファイル `ecs/graphics/surface_system.rs` 作成
2. 既存2システムを廃止
3. Surface生成/更新/削除を一元管理

**トレードオフ**:
- ✅ 明確な責務分離
- ✅ 将来の拡張性
- ❌ 新ファイル追加
- ❌ より大きなコード変更

### Option C: ハイブリッド（段階的移行）

**概要**: Phase 1で`sync_surface_from_arrangement`無効化、Phase 2で統合

**トレードオフ**:
- ✅ リスク分散
- ✅ 段階的検証可能
- ❌ 一時的な複雑化
- ❌ 移行期間中の不整合リスク

---

## 4. 実装複雑度と リスク

### 工数見積もり: **M (3-7日)**

**根拠**:
- 既存パターンの拡張（新規パターン導入なし）
- Surface生成ロジックの集約（中程度の複雑さ）
- RemovedComponents検出の追加（bevy_ecs標準機能）
- テスト更新が必要

### リスク評価: **Medium**

| リスク項目 | 評価 | 対策 |
|-----------|------|------|
| Surface生成タイミング問題 | 中 | 既存テストで検証 |
| DPIスケール計算ミス | 中 | GlobalArrangementの仕様確認 |
| デバイスロスト対応漏れ | 低 | 既存パターン踏襲 |
| パフォーマンス退化 | 低 | Changed検出は最適化済み |

---

## 5. 設計フェーズへの推奨事項

### 採用アプローチ: **Option A（既存コンポーネント拡張）**

**理由**:
- 変更範囲が最小限で予測可能
- 既存の`Changed<T>`パターンを活用
- marker-component-to-changed完了により前提条件クリア

### 設計フェーズで決定すべき事項

1. **Surface削除システムの実行タイミング**
   - GraphicsSetup? Draw? どちらに配置するか

2. **GlobalArrangement.boundsのサイズ計算**
   - `right - left`, `bottom - top` で物理ピクセルサイズ取得
   - 浮動小数点→整数変換の丸め方法（ceil? round?）

3. **既存テストの更新範囲**
   - `sync_surface_from_arrangement`に依存するテストの特定

### Research Needed（設計フェーズで調査）

- [ ] `RemovedComponents<GraphicsCommandList>`の検出タイミング
- [ ] Surface削除時のVisual.SetContent(null)の必要性
- [ ] デバイスロスト時のSurface再作成フロー確認

---

## 6. 次のステップ

1. **要件承認**: requirements.mdの内容を確認・承認
2. **設計生成**: `/kiro-spec-design surface-allocation-optimization`
3. **設計レビュー**: 上記の決定事項を設計ドキュメントで明確化
