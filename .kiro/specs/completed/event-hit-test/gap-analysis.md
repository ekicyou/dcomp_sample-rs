# Gap Analysis Report

| 項目 | 内容 |
|------|------|
| **Feature** | event-hit-test |
| **Date** | 2025-12-01 |
| **Version** | 1.0 |

---

## Analysis Summary

- **スコープ**: ECSベースのヒットテストシステム（矩形判定、Z順序走査、API提供）
- **既存資産の活用度**: 高 - `GlobalArrangement.bounds`, `D2DRectExt::contains()`, `Children` 階層が利用可能
- **主要課題**: 座標変換（スクリーン→ウィンドウローカル）、ツリー走査の実装
- **推奨アプローチ**: Option B（新規コンポーネント作成）- `HitTest` を `ecs::layout` に追加
- **Effort**: M (3-5日) | **Risk**: Low

---

## 1. Current State Investigation

### 1.1 関連する既存資産

| カテゴリ | ファイル/モジュール | 内容 | 再利用可能性 |
|---------|---------------------|------|-------------|
| **座標系** | `ecs/layout/arrangement.rs` | `GlobalArrangement.bounds` (物理ピクセル) | ✅ 直接利用可能 |
| **矩形操作** | `ecs/layout/rect.rs` | `D2DRectExt::contains(x, y)` | ✅ 直接利用可能 |
| **ECS階層** | `bevy_ecs::hierarchy` | `ChildOf`, `Children` | ✅ 直接利用可能 |
| **ツリー走査** | `ecs/common/tree_system.rs` | `propagate_parent_transforms` | ⚠️ 参考（走査パターン） |
| **メッセージ** | `win_message_handler.rs` | `WM_MOUSEMOVE`, `WM_NCHITTEST` | ⚠️ 統合ポイント |
| **テスト例** | `tests/taffy_layout_integration_test.rs` | レイアウトテストパターン | ✅ テスト設計参考 |

### 1.2 既存パターンと規約

**コンポーネント設計パターン**:
```rust
// ecs/layout/arrangement.rs の例
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SomeComponent { ... }
```

**ツリー走査パターン** (`ecs/common/tree_system.rs`):
- `Children` を使用した深さ優先走査
- `propagate_parent_transforms` のジェネリック設計が参考になる
- ただし、ヒットテストは逆順走査（front-to-back）が必要

**テストパターン** (`tests/taffy_layout_integration_test.rs`):
- `World::new()` で純粋なECSテスト
- `BoxStyle`, `Arrangement`, `GlobalArrangement` を直接spawn
- `Visual` なしでレイアウトテスト可能（要件の設計決定を裏付け）

### 1.3 統合ポイント

| 統合先 | 役割 | 備考 |
|--------|------|------|
| `win_message_handler.rs` | `WM_MOUSEMOVE` 受信 | lparam から座標取得 |
| `win_message_handler.rs` | `WM_NCHITTEST` 受信 | カーソル形状決定 |
| `ecs/layout/mod.rs` | モジュール公開 | `HitTest`, `HitTestMode` を pub use |
| `ecs/mod.rs` | 上位公開 | layout の再エクスポート確認 |

---

## 2. Requirements Feasibility Analysis

### 2.1 要件 → 技術ニーズマッピング

| 要件 | 技術ニーズ | 既存資産 | ギャップ |
|------|-----------|---------|---------|
| R1: HitTestMode enum | enum定義 | - | **Missing**: 新規作成 |
| R1: HitTest component | Componentマクロ | パターン確立済 | **Missing**: 新規作成 |
| R2: 矩形ヒットテスト | bounds判定 | `D2DRectExt::contains()` | ✅ 利用可能 |
| R3: Z順序走査 | Children逆順走査 | `Children` コンポーネント | **Missing**: 走査ロジック |
| R4: ヒットテスト除外 | None モード判定 | - | **Missing**: 分岐ロジック |
| R5: 座標変換 | スクリーン→ローカル | - | **Missing**: 変換関数 |
| R6: ECS統合 | Query定義 | bevy_ecs パターン | **Missing**: システム関数 |
| R7: 呼び出しタイミング | キャッシュ機構 | - | **Missing**: キャッシュ実装 |
| R8: API | 公開関数 | - | **Missing**: API関数 |

### 2.2 ギャップ詳細

#### Missing: HitTest コンポーネント
```rust
// 新規作成が必要
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HitTest {
    pub mode: HitTestMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HitTestMode {
    None,
    #[default]
    Bounds,
}
```

#### Missing: ヒットテスト走査関数
- 深さ優先・逆順走査（front-to-back）
- `Children` の逆イテレーション
- 再帰的な子孫調査

#### Missing: 座標変換
- `ScreenToClient` (Win32 API) 相当の処理
- Window の `GlobalArrangement.bounds` オフセット適用

#### Missing: キャッシュ機構
- 前回座標とヒット結果の保持
- `ArrangementTreeChanged` によるキャッシュ無効化

### 2.3 複雑性シグナル

| 側面 | 評価 | 理由 |
|------|------|------|
| アルゴリズム | 中 | 再帰ツリー走査は単純だが、逆順・クリッピングなし考慮が必要 |
| 外部統合 | 低 | Win32 座標のみ（外部サービスなし） |
| ECS統合 | 低 | 既存パターンに従う |
| テスト | 低 | レイアウトテストパターンが確立済 |

---

## 3. Implementation Approach Options

### Option A: 既存モジュール拡張

**アプローチ**: `arrangement.rs` に `HitTest` 関連を追加

**変更対象**:
- `ecs/layout/arrangement.rs`: HitTest, HitTestMode 追加
- `ecs/layout/mod.rs`: pub use 追加

**Trade-offs**:
- ✅ ファイル数最小化
- ✅ GlobalArrangement との近接性
- ❌ arrangement.rs が肥大化（現在220行 → 400行以上）
- ❌ 責務混在（配置 + ヒットテスト）

**推奨度**: ⚠️ 非推奨

---

### Option B: 新規モジュール作成 【推奨】

**アプローチ**: `ecs/layout/hit_test.rs` を新規作成

**変更対象**:
- `ecs/layout/hit_test.rs`: 新規（HitTest, HitTestMode, API関数）
- `ecs/layout/mod.rs`: pub mod hit_test, pub use 追加
- `tests/hit_test_integration_test.rs`: 新規テスト

**ディレクトリ構造**:
```
ecs/layout/
├── arrangement.rs      # 既存（変更なし）
├── hit_test.rs         # 新規
├── mod.rs              # pub mod/use 追加
├── rect.rs             # 既存（変更なし）
├── systems.rs          # 既存（変更なし）
└── ...
```

**Trade-offs**:
- ✅ 単一責任の原則遵守
- ✅ テスト容易性（独立してテスト可能）
- ✅ 将来の拡張性（AlphaMask等を別ファイルに追加可能）
- ❌ ファイル数増加（1ファイル）

**推奨度**: ✅ 推奨

---

### Option C: ハイブリッド（段階的）

**アプローチ**: 
1. Phase 1: `hit_test.rs` にコンポーネントとAPI
2. Phase 2: `hit_test_cache.rs` にキャッシュ機構
3. Phase 3: メッセージハンドラ統合

**Trade-offs**:
- ✅ 段階的リスク軽減
- ❌ 今回の要件規模では過剰

**推奨度**: ⚠️ 今回は不要（Option B で十分）

---

## 4. Effort and Risk Assessment

### Effort: M (3-5日)

| タスク | 見積もり |
|--------|---------|
| HitTest コンポーネント定義 | 0.5日 |
| ヒットテスト走査関数 | 1日 |
| 座標変換ユーティリティ | 0.5日 |
| キャッシュ機構 | 1日 |
| API設計・実装 | 0.5日 |
| テスト（ユニット + 統合） | 1日 |
| **合計** | **4.5日** |

### Risk: Low

| リスク要因 | 評価 | 緩和策 |
|-----------|------|--------|
| 技術不確実性 | 低 | 既存パターンに従う |
| 外部依存 | 低 | Win32 APIのみ |
| アーキテクチャ影響 | 低 | 独立コンポーネント |
| パフォーマンス | 低 | O(n)で数百エンティティは問題なし |

---

## 5. Recommendations for Design Phase

### 推奨アプローチ

**Option B: 新規モジュール作成**

### キー設計決定

1. **モジュール配置**: `ecs/layout/hit_test.rs`
2. **公開API**:
   - `hit_test(world: &World, point: PhysicalPoint) -> Option<Entity>`
   - `hit_test_in_window(world: &World, window: Entity, point: PhysicalPoint) -> Option<Entity>`
3. **キャッシュ**: `HitTestCache` リソースとして実装

### Research Items（設計フェーズで調査）

| 項目 | 内容 |
|------|------|
| **R1** | `ScreenToClient` のRust実装方法（windows crate経由） |
| **R2** | `Children` の逆イテレーション最適化（.iter().rev() のコスト） |
| **R3** | キャッシュ無効化の最適タイミング（ArrangementTreeChanged との連携） |

### 次のステップ

1. 要件定義書の承認
2. `/kiro-spec-design event-hit-test` で設計書生成
3. 設計レビュー後、タスク分解

---

## Appendix: 既存コード参照

### D2DRectExt::contains() 実装

```rust
// ecs/layout/rect.rs:140
fn contains(&self, x: f32, y: f32) -> bool {
    x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
}
```

### Children 階層パターン

```rust
// bevy_ecs::hierarchy 使用例（ecs/layout/systems.rs）
use bevy_ecs::hierarchy::{ChildOf, Children};

// Children からの走査
for child in children.iter() { ... }
```

### テストパターン

```rust
// tests/taffy_layout_integration_test.rs
#[test]
fn test_example() {
    let mut world = World::new();
    let entity = world.spawn((
        BoxStyle { ... },
        Arrangement::default(),
    )).id();
    // Visual なしでテスト可能
}
```
