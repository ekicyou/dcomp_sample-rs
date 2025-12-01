# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | event-hit-test 要件定義書 |
| **Version** | 0.1 (Draft) |
| **Date** | 2025-12-01 |
| **Parent Spec** | wintf-P0-event-system |
| **Author** | AI-DLC System |

---

## Introduction

本仕様書は wintf フレームワークにおけるヒットテストシステムの要件を定義する。親仕様「wintf-P0-event-system」の Requirement 1（ヒットテストシステム）と Requirement 2（ヒット領域定義）を実装する。画面座標からエンティティを特定し、ヒットテストタイプに基づいた判定を行う。

### 背景

デスクトップマスコットアプリケーションでは、キャラクター画像のクリック判定が必要である。透明部分をクリックしてもヒットしない、部位ごとに異なる反応をする等の要件があり、柔軟なヒットテストシステムが必要となる。

### スコープ

**含まれるもの**:
- ヒットテストタイプの定義（Enum）
- 矩形領域によるヒットテスト（GlobalArrangementのboundsを使用）
- 座標変換（スクリーン座標 → ウィンドウ座標 → ローカル座標）
- Z順序に基づく最前面エンティティの特定
- ECSシステムとしての実装

**含まれないもの（孫仕様で対応）**:
- αマスクによるピクセル単位ヒットテスト → `event-hit-test-alpha-mask`
- 多角形・カスタム形状によるヒットテスト → 将来の孫仕様
- 名前付きヒット領域 → 将来の孫仕様

### 技術的調査結果

**GlobalArrangementからの座標取得**:
- `GlobalArrangement.bounds`: `D2D_RECT_F` 型、仮想デスクトップ座標系（LayoutRoot基準）
- **単位: 物理ピクセル**（DIP × scale で計算済み）
- LayoutRoot の `Arrangement.scale` にはDPIスケールファクターが設定される
- 子エンティティの `GlobalArrangement` は親からの累積変換により物理ピクセル座標に変換済み

**座標系の階層**:
```
LayoutRoot (仮想デスクトップ: 物理ピクセル)
  └── Monitor (各モニター: 物理ピクセル)
        └── Window (ウィンドウ: 物理ピクセル)
              └── Visual (ウィジェット: 物理ピクセル)
```

**ヒットテストの座標変換**:
- Win32 `WM_NCHITTEST` 等のlparamはスクリーン座標（物理ピクセル）
- `GlobalArrangement.bounds` は既に物理ピクセルなので直接比較可能
- DIP→物理ピクセル変換は不要（レイアウトシステムで変換済み）

---

## Requirements

### Requirement 1: ヒットテストタイプ定義

**Objective:** 開発者として、Visualのヒットテスト動作を設定したい。それによりウィジェットごとに適切なヒット判定を行える。

#### Acceptance Criteria

1. The HitTest System shall `HitTestMode` enumを提供し、以下のバリアントを含む：
   - `None`: ヒットテスト対象外（透過）
   - `Bounds`: 矩形領域（GlobalArrangement.bounds）でヒットテスト
2. The HitTest System shall 将来の拡張に備え、`HitTestMode` enumに新しいバリアントを追加可能な設計とする
3. The `Visual` component shall `hit_test_mode: HitTestMode` フィールドを持つ
4. When `Visual` componentが作成された時, the HitTest System shall デフォルト値として `HitTestMode::Bounds` を設定する

---

### Requirement 2: 矩形ヒットテスト

**Objective:** 開発者として、ウィジェットの矩形領域でヒット判定を行いたい。それにより基本的なクリック検出が可能になる。

#### Acceptance Criteria

1. When `HitTestMode::Bounds` が設定されている時, the HitTest System shall `GlobalArrangement.bounds` を使用してヒット判定を行う
2. The HitTest System shall スクリーン座標（物理ピクセル）を入力として受け付ける
3. The HitTest System shall スクリーン座標をウィンドウローカル座標（論理ピクセル）に変換する
4. The HitTest System shall ウィンドウローカル座標が `GlobalArrangement.bounds` 内にあるかを判定する
5. When ヒット判定が成功した時, the HitTest System shall ヒットしたEntityを返す

---

### Requirement 3: Z順序によるヒット優先度

**Objective:** 開発者として、重なったウィジェットのうち最前面のものを取得したい。それにより正しいインタラクション対象を特定できる。

#### Acceptance Criteria

1. The HitTest System shall DirectCompositionビジュアルツリーのZ順序ルールに従う
2. The HitTest System shall ECS階層構造（Parent/Children）を使用してZ順序を決定する
3. The HitTest System shall Children配列の順序 = 描画順序 = Z順序として扱う（後の要素が前面）
4. When 親エンティティと子エンティティの両方がヒットした時, the HitTest System shall 子エンティティを優先して返す
5. The HitTest System shall 深さ優先・逆順走査（front-to-back）でヒットテストを行う

#### ヒットテスト走査アルゴリズム

```rust
fn hit_test_recursive(entity: Entity, point: Point) -> Option<Entity> {
    // 1. 子を逆順で調査（前面から背面へ）
    //    注意: 親の bounds に関係なく子を調査（クリッピングなしのため）
    if let Some(children) = get_children(entity) {
        for child in children.iter().rev() {
            if let Some(hit) = hit_test_recursive(*child, point) {
                return Some(hit);
            }
        }
    }
    // 2. 子でヒットしなければ、自身をチェック
    if should_hit_test(entity) && is_point_in_bounds(entity, point) {
        return Some(entity);
    }
    None
}
```

#### 設計上の注意

- **クリッピングなし**: 現在の実装ではクリッピングが存在しないため、子エンティティが親の bounds 外に存在する可能性がある
- 親の bounds 外でも子の調査をスキップしてはならない
- 将来クリッピングが導入された場合、クリップ設定のあるエンティティでは最適化（早期スキップ）が可能になる

---

### Requirement 4: ヒットテスト除外

**Objective:** 開発者として、特定のウィジェットをヒットテスト対象外にしたい。それにより装飾用要素やオーバーレイを透過させられる。

#### Acceptance Criteria

1. When `HitTestMode::None` が設定されている時, the HitTest System shall そのエンティティをヒット対象から除外する
2. When 親エンティティが `HitTestMode::None` の時, the HitTest System shall 子エンティティのヒットテストには影響を与えない
3. The HitTest System shall `HitTestMode::None` のエンティティを完全にスキップし、その下のエンティティをヒット対象とする

---

### Requirement 5: 座標変換

**Objective:** 開発者として、各種座標系間の変換を行いたい。それによりスクリーン座標からローカル座標への正確な変換が可能になる。

#### Acceptance Criteria

1. The HitTest System shall スクリーン座標（物理ピクセル）からウィンドウクライアント座標（物理ピクセル）への変換を提供する
2. The HitTest System shall ウィンドウクライアント座標（物理ピクセル）と `GlobalArrangement.bounds`（物理ピクセル）を直接比較できる
3. The HitTest System shall 物理ピクセル座標からエンティティローカル座標への変換を提供する（`GlobalArrangement.transform`の逆変換）
4. The HitTest System shall `GlobalArrangement.bounds` が物理ピクセル座標であることを前提として座標判定を行う

---

### Requirement 6: ECSシステム統合

**Objective:** 開発者として、ヒットテストをECSシステムとして利用したい。それにより既存のwintfアーキテクチャと一貫性を保てる。

#### Acceptance Criteria

1. The HitTest System shall ヒットテストクエリを実行するための関数を提供する
2. The HitTest System shall `GlobalArrangement` コンポーネントを持つエンティティのみをヒットテスト対象とする
3. The HitTest System shall `Visual` コンポーネントを持つエンティティのみをヒットテスト対象とする
4. When エンティティに `GlobalArrangement` が存在しない時, the HitTest System shall そのエンティティをスキップする

---

### Requirement 7: ヒットテスト呼び出しタイミング

**Objective:** 開発者として、適切なタイミングでヒットテストを実行したい。それによりホバー状態やさわり反応などのインタラクションを実現できる。

#### Acceptance Criteria

1. The HitTest System shall `WM_MOUSEMOVE` メッセージ受信時にヒットテストを実行する
2. The HitTest System shall マウス座標が変化していない場合、前回のヒット結果をキャッシュから返す
3. The HitTest System shall `WM_LBUTTONDOWN`、`WM_RBUTTONDOWN` 等のボタンイベント時にもヒットテストを実行する
4. The HitTest System shall `WM_NCHITTEST` でカーソル形状決定に使用できる

#### キャッシュ戦略

- 前回のマウス座標とヒット結果を保持
- 座標が同一の場合は再計算をスキップ
- レイアウト変更時（`ArrangementTreeChanged`）にキャッシュを無効化

---

### Requirement 8: ヒットテストAPI

**Objective:** 開発者として、関数呼び出しでヒットテストを実行したい。それによりユニットテストやデモでの検証が可能になる。

#### Acceptance Criteria

1. The HitTest System shall 同期関数としてヒットテストAPIを提供する
2. The HitTest System shall `hit_test(world: &World, point: Point) -> Option<Entity>` シグネチャの関数を提供する
3. The HitTest System shall ヒット結果として `Entity` を返す（ヒットなしの場合は `None`）
4. The HitTest System shall Windowエンティティを指定してスコープを限定できるオーバーロードを提供する

#### API設計

```rust
/// 指定座標でヒットテストを実行（グローバル座標）
pub fn hit_test(world: &World, point: PhysicalPoint) -> Option<Entity>;

/// 指定Window内でヒットテストを実行
pub fn hit_test_in_window(world: &World, window: Entity, point: PhysicalPoint) -> Option<Entity>;

/// ヒットテスト結果（詳細情報付き）
pub struct HitTestResult {
    pub entity: Entity,
    pub local_point: Point,  // エンティティローカル座標
}

pub fn hit_test_detailed(world: &World, point: PhysicalPoint) -> Option<HitTestResult>;
```

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

- ヒットテスト: 1ms以内で完了（数百エンティティまで）
- 初期実装: O(n) 線形ツリー走査（数百エンティティ規模では十分な性能）
- 将来的な最適化として空間分割（Quadtree等）を検討可能な設計とする
- 検索アルゴリズムを差し替え可能な trait ベースの設計とする
- キャッシュにより連続した同一座標でのヒットテストを最適化

### NFR-2: 精度

- 座標変換の誤差: 0.5ピクセル以内
- DPI対応: 96, 120, 144, 192 DPI で正確に動作

### NFR-3: 拡張性

- `HitTestMode` enumは将来のバリアント追加を容易にする
- 孫仕様（`event-hit-test-alpha-mask`等）との統合を考慮した設計

### NFR-4: テスト容易性

- ヒットテストAPIは関数呼び出しで直接実行可能
- デモ・テストコードから任意のタイミングでヒットテストを実行できる
- ヒット結果をログ出力して検証可能

---

## Glossary

| 用語 | 説明 |
|------|------|
| ヒットテスト | 画面座標からエンティティを特定する処理 |
| 物理ピクセル | 実際のディスプレイピクセル（DPI × DIP） |
| DIP | Device Independent Pixels（論理ピクセル、96 DPI基準） |
| GlobalArrangement | 仮想デスクトップ座標系での累積変換とバウンディングボックス（物理ピクセル） |
| Z順序 | 描画の前後関係（前面/背面） |
| LayoutRoot | 仮想デスクトップ全体を表すルートエンティティ |

---

## Appendix

### A. 関連ドキュメント

- 親仕様: `.kiro/specs/wintf-P0-event-system/requirements.md`
- 孫仕様: `.kiro/specs/event-hit-test-alpha-mask/requirements.md`
- レイアウトシステム: `doc/spec/04-layout-system.md`

### B. 座標変換フロー

```
スクリーン座標（物理px）
    ↓ ScreenToClient(hwnd, point)
ウィンドウクライアント座標（物理px）
    ↓ + Window.GlobalArrangement.bounds.left/top
仮想デスクトップ座標（物理px）
    ↓ GlobalArrangement.bounds との比較
ヒットテスト判定

（オプション）エンティティローカル座標への変換：
仮想デスクトップ座標（物理px）
    ↓ GlobalArrangement.transform.inverse()
エンティティローカル座標（物理px）
```

### C. HitTestMode enum設計

```rust
/// ヒットテストの動作モード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HitTestMode {
    /// ヒットテスト対象外（マウスイベントを透過）
    None,
    /// バウンディングボックス（GlobalArrangement.bounds）でヒットテスト
    #[default]
    Bounds,
    // 将来の拡張用バリアント:
    // AlphaMask,     // αマスクによるピクセル単位判定（event-hit-test-alpha-mask）
    // Polygon(..),   // 多角形によるヒットテスト
    // Custom(..),    // カスタムヒットテスト関数
}
```

### D. ヒットテスト戦略 trait 設計（将来の最適化用）

```rust
/// ヒットテスト戦略（将来の検索アルゴリズム差し替え用）
pub trait HitTestStrategy {
    fn find_hit(&self, world: &World, point: Point) -> Option<Entity>;
}

/// デフォルト実装: 線形ツリー走査（O(n)）
pub struct TreeTraversalHitTest;

impl HitTestStrategy for TreeTraversalHitTest {
    fn find_hit(&self, world: &World, point: Point) -> Option<Entity> {
        // 深さ優先・逆順走査
    }
}

// 将来の拡張例:
// pub struct SpatialIndexHitTest { quadtree: Quadtree }
// pub struct CachedHitTest { cache: HitTestCache, inner: Box<dyn HitTestStrategy> }
```

### E. テスト方法（taffy_flex_demo）

```rust
// デモ内でのヒットテスト検証例
// 表示1秒後、6秒後にヒットテストAPIを呼び出し、結果をログ出力

fn test_hit_test_delayed(world: &mut World) {
    // 1秒後に実行
    schedule_delayed(Duration::from_secs(1), move |world| {
        let test_point = PhysicalPoint { x: 500.0, y: 400.0 };
        if let Some(entity) = hit_test(world, test_point) {
            if let Some(name) = world.get::<Name>(entity) {
                info!("[HitTest @1s] Hit: {:?} at {:?}", name.as_str(), test_point);
            } else {
                info!("[HitTest @1s] Hit: Entity {:?} (no name) at {:?}", entity, test_point);
            }
        } else {
            info!("[HitTest @1s] No hit at {:?}", test_point);
        }
    });

    // 6秒後に実行
    schedule_delayed(Duration::from_secs(6), move |world| {
        // 同様のテスト
    });
}
```
