# セッションコンテキスト: surface-allocation-optimization

## 最終更新

- **日時**: 2025-11-28 (要件承認)
- **フェーズ**: requirements-approved（要件承認済み）
- **次のステップ**: 設計生成 (`/kiro-spec-design`)
- **前提条件**: marker-component-to-changed ✅ 完了済み

## 現在の状況

### 完了済み

1. **要件定義** ✅ 承認済み
   - R1: GraphicsCommandList存在に基づくSurface生成判定
   - R2: Surface生成システムの一本化
   - R3: DPIスケール対応のSurfaceサイズ計算
   - R4: 既存SurfaceGraphicsとの整合性維持
   - R5: 診断とデバッグ支援

2. **ギャップ分析** ✅ 完了
   - 推奨アプローチ: Option A（既存コンポーネント拡張）
   - 工数: M (3-7日)
   - リスク: Medium

### 未完了

1. **設計生成** ⏳
2. **タスク生成** ⏳
3. **実装** ⏳

## 重要な議論経緯

### 1. Changed方式の採用決定

セッション中に以下が決定された：
- マーカーコンポーネントの`With<>` + `remove()`パターンは高コスト（アーキタイプ変更）
- `Changed<T>`パターンを採用（自動リセット、低オーバーヘッド）
- この方針は別仕様`marker-component-to-changed`として切り出し済み

### 2. R2 AC-3 vs R3 AC-4 の整理

- **R2 AC-3**: Surface「生成」トリガー = `GraphicsCommandList`の存在
- **R3 AC-4**: Surface「再作成」トリガー = `GlobalArrangement`サイズ変更

両者は同じ作成処理だが、トリガー条件が異なる。設計フェーズで統一的に扱う。

### 3. DirectComposition Surfaceの制約

- Surfaceはリサイズ不可、再作成のみ
- `insert()`で上書きすればCOMオブジェクトは自動回収される

## 関連仕様

- **marker-component-to-changed**: ✅ **完了済み・アーカイブ** (2025-11-27)
  - `Changed<SurfaceGraphicsDirty>` パターン実装済み
  - `HasGraphicsResources` 世代番号方式実装済み
  - 本仕様の前提条件が満たされた

## 現在のコードベース状況（2025-11-28確認）

### Surface生成の二重経路（解消対象）
1. **`sync_surface_from_arrangement`**: `Changed<Arrangement>` で発火
   - GraphicsCommandList有無を確認せずSurface作成 → **廃止対象**
2. **`deferred_surface_creation_system`**: GraphicsCommandList存在時のみ発火
   - 正しい方式 → **唯一のシステムとする**

### DPIスケール問題
- 現在: `Arrangement`（論理サイズ）を使用
- 目標: `GlobalArrangement.bounds`（物理ピクセルサイズ）を使用

## 再開時の手順

1. このファイルを確認
2. `/kiro-spec-status surface-allocation-optimization`で状態確認
3. `/kiro-spec-design surface-allocation-optimization`で設計生成

## 議題（要件承認時に検討）

1. ~~**marker-component-to-changedとの依存関係**~~ ✅ 解決済み
   - Changed方式は実装完了
   - 先行実装の必要なし

2. **R2とR3の統合可能性**
   - 生成と再作成を同一システムで処理
   - 検出システムは分離するか統合するか

3. **要件定義の妥当性確認**
   - R2: `sync_surface_from_arrangement`廃止は現コードと整合
   - R3: `GlobalArrangement`使用への変更が必要

## ファイル構成

```text
.kiro/specs/surface-allocation-optimization/
├── spec.json              # 仕様メタデータ（requirements-generated）
├── requirements.md        # 要件定義（R1-R5）
├── gap-analysis.md        # ギャップ分析（2025-11-28生成）
└── SESSION_CONTEXT.md     # 本ファイル
```
