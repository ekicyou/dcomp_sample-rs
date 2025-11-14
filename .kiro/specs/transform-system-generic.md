# 仕様: transform_system.rs実装のジェネリック化

**機能名**: `transform-system-generic`  
**作成日**: 2025-11-14  
**ステータス**: Phase 1 - 初期化完了

## 概要

`crates/wintf/src/ecs/transform_system.rs`の実装をジェネリック化し、汎用性と再利用性を向上させます。現在の実装は特定の型に依存しているため、これをジェネリック型パラメータを使用した柔軟な設計に変更します。

## 現状の課題

現在の`transform_system.rs`は以下の特徴を持っています：
- `Transform`, `GlobalTransform`, `TransformTreeChanged`といった固定型への依存
- `ChildOf`, `Children`という固定の親子関係コンポーネント
- 並列処理のための`WorkQueue`実装
- DirectComposition連携を前提とした実装

これらを、より汎用的な型システムで抽象化することで、異なるグラフィックスバックエンドやユースケースへの適用を容易にする必要があります。

## ステアリングコンテキストとの整合性

**プロダクト観点**:
- ECSアーキテクチャの利点を最大化する設計変更
- Windows特有の機能に依存しない汎用的な階層変換システム

**技術観点**:
- Rustの型安全性を活用したジェネリック設計
- bevy_ecsの標準パターンに準拠
- `unsafe`コードの範囲は変更せず、安全性は維持

**構造観点**:
- `/crates/wintf/src/ecs/`層での変更
- 他のECSコンポーネントへの影響を最小化

## 要件定義

### 機能要件 (FR: Functional Requirements)

#### FR1: 型パラメータの抽象化
**スコープ**: 以下の3つの型を型パラメータ化する
1. `Transform` → ローカル変換型 `L`
2. `GlobalTransform` → グローバル変換型 `G`
3. `TransformTreeChanged` → ダーティマーカー型 `M`

**型パラメータの条件**:

##### L: ローカル変換型
```rust
L: Component + Copy + Into<G>
```
- `Component`: bevy_ecsコンポーネントとして使用可能
- `Copy`: 値渡しによる効率的な処理
- `Into<G>`: グローバル変換型Gへ変換可能（`sync_simple_transforms`で必要）

**理由**: `sync_simple_transforms`内で `GlobalTransform((*transform).into())` を使用しているため、
`L` → `G` の変換が必須。

##### G: グローバル変換型
```rust
G: Component + Copy + PartialEq + Mul<L, Output = G>
```
- `Component`: bevy_ecsコンポーネントとして使用可能
- `Copy`: 値渡しによる効率的な処理
- `PartialEq`: `set_if_neq`最適化のために必要
- `Mul<L, Output = G>`: 親のグローバル変換と子のローカル変換を合成（`propagate_descendants_unchecked`で必要）

**理由**: 
- `PartialEq`は変換伝播時の不要な更新を避けるための最適化
- `Mul<L, Output = G>`は階層的な変換合成（`parent_global * child_local = child_global`）に必須

##### M: ダーティマーカー型
```rust
M: Component
```
- `Component`: bevy_ecsコンポーネントとして使用可能

**理由**: bevy_ecsの変更検出機能（`is_changed()`, `set_changed()`）を利用するための最小要件。
ゼロサイズ型（ZST）であることが望ましいが、型パラメータ制約としては不要。

#### FR2: 階層関係の抽象化（フェーズ2へ延期）

**現状**: `ChildOf`と`Children`は具体型のまま維持
**理由**: 
- bevy_ecsの標準型への依存を段階的に解決
- フェーズ1のリスクを最小化

#### FR3: 変換操作の抽象化

**TransformOpsトレイト**:
```rust
pub trait TransformOps<L, G>
where
    L: Component + Copy + Into<G>,
    G: Component + Copy + PartialEq + Mul<L, Output = G>,
{
    /// ローカル変換からグローバル変換を作成
    fn from_local(local: L) -> G;
    
    /// グローバル変換とローカル変換を合成
    fn compose(parent_global: G, child_local: L) -> G;
}
```

**設計判断**:
- `from_local`は`L::into()`のラッパー（将来の拡張性のため）
- `compose`は`parent_global * child_local`のラッパー
- トレイト境界で既に必要な制約（`Into<G>`, `Mul<L, Output = G>`）を含む

#### FR4: 並列処理の維持
- **FR4.1**: `WorkQueue`の動作を維持
  - 条件: 型パラメータ化後も並列性能を維持
  - 制約: `Entity`ベースのキューイングは変更不要

- **FR4.2**: `ParamSet`と`par_iter_mut`の互換性
  - 条件: bevy_ecsのクエリシステムとの整合性を保つ

### 非機能要件 (NFR: Non-Functional Requirements)

#### NFR1: パフォーマンス
- **NFR1.1**: 現在の並列処理性能を維持（10%以内の性能劣化に抑える）
- **NFR1.2**: ゼロコスト抽象化の原則に従う
  - モノモーフィゼーションにより実行時のオーバーヘッドを排除
  - コンパイル時の型チェックにより安全性を確保

#### NFR2: 型安全性
- **NFR2.1**: `unsafe`コードの範囲を変更しない
  - 現在の`propagate_descendants_unchecked`の安全性保証を維持
  - 型システムで表現可能な制約はトレイト境界で実装

- **NFR2.2**: コンパイル時の型エラーメッセージの明確性
  - トレイト境界エラーが理解しやすいこと
  - 型推論が適切に機能すること

#### NFR3: 互換性
- **NFR3.1**: 既存の`transform.rs`との後方互換性を維持
  - 現在の`Transform`、`GlobalTransform`は型エイリアスとして提供
  - 既存のコードは変更なしで動作すること

- **NFR3.2**: bevy_ecs 0.17.2との互換性維持
  - クエリシステムの動作変更なし
  - コンポーネントライフタイムの扱いに変更なし

#### NFR4: 保守性
- **NFR4.1**: ドキュメントの充実
  - ジェネリック型パラメータの制約を明確に文書化
  - 使用例を提供（ドキュメントテスト）

- **NFR4.2**: コードの可読性維持
  - 過度なジェネリック化を避ける
  - 型エイリアスで複雑な型シグネチャを簡潔にする

### 制約条件

#### C1: 技術的制約
- Rust 2021 Edition以降
- bevy_ecs 0.17.2の機能範囲内
- Windows固有の`Matrix3x2`型への依存は許容（プラットフォーム固有部分）

#### C2: 実装制約
- `unsafe`コードのセマンティクスを変更しない
- 並列処理のアルゴリズムを変更しない（ジェネリック化のみ）
- テスト可能性を損なわない

#### C3: スコープ制約
- 以下は対象外:
  - 3D変換への対応（2D変換のジェネリック化に集中）
  - 階層システムの再設計（bevy_ecsの標準を使用）
  - アニメーションシステムとの統合

### 受け入れ基準

#### AC1: 基本機能
- [ ] ジェネリック版の`sync_simple_transforms`が既存の型で動作する
- [ ] ジェネリック版の`mark_dirty_trees`が既存の型で動作する
- [ ] ジェネリック版の`propagate_parent_transforms`が既存の型で動作する

#### AC2: 拡張性
- [ ] カスタム変換型を定義し、システムが動作することを確認
- [ ] 型パラメータの制約が適切にエラーを報告することを確認

#### AC3: パフォーマンス
- [ ] 既存のベンチマークで性能劣化が10%以内であることを確認
- [ ] コンパイル時間の増加が許容範囲内（20%以内）であることを確認

#### AC4: 互換性
- [ ] 既存のサンプル（`areka.rs`, `dcomp_demo.rs`）が変更なしでビルド・動作する
- [ ] 既存のテストが全て通過する

## フェーズ

### Phase 1: 仕様策定
- [x] 初期化完了
- [x] 要件定義完了
- [ ] ギャップ分析
- [ ] 設計
- [ ] タスク分解

### Phase 2: 実装
- [ ] 実装未開始

## ギャップ分析

### 既存実装の詳細

#### コード構造分析

**ファイル**: `crates/wintf/src/ecs/transform_system.rs` (360行)

**公開関数（3つ）**:
1. `sync_simple_transforms` (16-45行)
   - 階層を持たないエンティティの変換を更新
   - `ParamSet`で2つのクエリを切り替え
   - `RemovedComponents<ChildOf>`で孤立エンティティに対応
   
2. `mark_dirty_trees` (50-74行)
   - ダーティビットを親方向に伝播
   - 静的シーン最適化のための前処理
   
3. `propagate_parent_transforms` (82-140行)
   - ルートエンティティから並列で変換を伝播
   - `WorkQueue`を使った並列タスク管理

**内部関数（2つ）**:
1. `propagation_worker` (145-206行)
   - `WorkQueue`からタスクを取得して処理
   - スピンロックによる効率的なタスク取得
   
2. `propagate_descendants_unchecked` (228-295行) **[unsafe]**
   - 深さ優先探索で子孫に変換を伝播
   - 反復的実装（再帰ではない）
   - `max_depth`パラメータでタスク分割を制御

**型定義**:
- `NodeQuery` (299-311行): 頻繁に使用されるクエリの型エイリアス
- `WorkQueue` (314-359行): 並列処理のためのワークキュー

#### 型依存性の分析

**直接的な型依存**:
```rust
// transform.rsからのインポート
use super::transform::{GlobalTransform, Transform, TransformTreeChanged};

// bevy_ecsからの暗黙的な依存
ChildOf      // bevy_hierarchy提供
Children     // bevy_hierarchy提供
```

**型が使用されている箇所**:

1. **Transform**: 7箇所
   - L19, L26: クエリのコンポーネント型
   - L53: `Changed<Transform>`フィルタ
   - L85: ルートクエリの型
   - L94, L35, L42: `Into<Matrix3x2>`への変換
   - L266: 変換合成での使用
   - L305: `NodeQuery`型定義

2. **GlobalTransform**: 11箇所
   - L19, L26: クエリのコンポーネント型（ミュータブル）
   - L53: `Added<GlobalTransform>`フィルタ
   - L85: ルートクエリの型（ミュータブル）
   - L94, L35, L42: `Transform`からの変換代入
   - L230: 関数パラメータ
   - L265-267: 変換合成（`a * b`）
   - L306: `NodeQuery`型定義

3. **TransformTreeChanged**: 4箇所
   - L56: クエリのコンポーネント型（ミュータブル）
   - L66: `.set_changed()`呼び出し
   - L86: ルートクエリのフィルタ
   - L257: ダーティチェック（`.is_changed()`）
   - L307: `NodeQuery`型定義

4. **ChildOf**: 9箇所
   - L22, L26: `Without<ChildOf>`フィルタ
   - L28: `RemovedComponents<ChildOf>`
   - L53: `Changed<ChildOf>`フィルタ
   - L55: `RemovedComponents<ChildOf>`
   - L56, L67: クエリのコンポーネント型
   - L86: `Without<ChildOf>`フィルタ
   - L261: `.parent()`メソッド呼び出し
   - L309: `NodeQuery`型定義（`Read<ChildOf>`）

5. **Children**: 6箇所
   - L23: `Without<Children>`フィルタ
   - L26: `Without<Children>`フィルタ
   - L85: ルートクエリのコンポーネント型
   - L231: 関数パラメータ
   - L250: `.iter()`イテレーション
   - L269-271: `Option<&Children>`のマップ
   - L309: `NodeQuery`型定義（`Option<Read<Children>>`）

#### ジェネリック化の障壁

**障壁1: bevy_hierarchyへの依存**
- 現状: `ChildOf`と`Children`はbevy_ecsが提供（bevy_hierarchy機能）
- 問題: これらの型は現在のwintfのCargo.tomlに明示的に依存していない
- 調査結果: bevy_ecs 0.17.2の`serialize`フィーチャに含まれていると推測される
- ギャップ: 型定義が外部クレートにあるため、直接ジェネリック化できない

**障壁2: 型固有のメソッド依存**
- `ChildOf::parent()` (L67, L261)
- `TransformTreeChanged::set_changed()` (L66)
- `GlobalTransform(Matrix3x2)` のnewtype構築 (L35, L42, L94)
- `Transform`から`Matrix3x2`への`Into`変換 (L94, L35, L42, L266)

**障壁3: 演算子オーバーロード**
- `GlobalTransform * Transform = GlobalTransform` (L267: `a * b`)
- 現在の実装は`transform.rs`で定義された`Mul`トレイト実装に依存
- ジェネリック化には適切なトレイト境界が必要

**障壁4: bevy_ecsのクエリシステム制約**
- `Query<...>`の型パラメータはコンパイル時に確定必要
- `Read<T>`と`Mut<T>`はbevy_ecsの内部型（lifetimeless）
- `ParamSet`は具体的な型が必要

#### 使用箇所の分析

**world.rsでの使用**:
- 検索結果: 使用箇所なし
- 意味: これらのシステムはデフォルトでは登録されていない
- 推測: ユーザーが明示的に`add_systems()`で登録する必要がある

**examplesでの使用**:
- 検索結果: 使用箇所なし
- 意味: 現在のサンプルアプリケーションは変換システムを使用していない
- 推測: DirectCompositionの変換機能を直接使用している可能性

**影響範囲**:
- 既存コードへの影響: **最小限**
- 理由: システム関数が外部から使用されていない
- 利点: ジェネリック化による破壊的変更のリスクが低い

### ギャップの詳細

#### GAP1: 階層コンポーネントの抽象化
**現状**: `ChildOf`と`Children`は暗黙的にbevy_ecsから提供
**要件**: カスタム階層構造への対応（FR2.1）
**ギャップ**: 
- 型の出所が不明確（bevy_hierarchyの可能性）
- メソッド（`.parent()`, `.iter()`）の抽象化が必要
- `RemovedComponents<T>`のジェネリック対応が必要
**解決策の方向性**:
- トレイトで階層関係を抽象化（`HierarchyParent`, `HierarchyChildren`など）
- bevy_ecsの標準型をデフォルト実装として提供

#### GAP2: 変換型の抽象化
**現状**: `Transform`, `GlobalTransform`は`transform.rs`で定義
**要件**: 任意の変換型への対応（FR1.1, FR1.2）
**ギャップ**:
- `Into<Matrix3x2>`への変換が必要
- `Mul`トレイトでの合成が必要
- newtype構築パターン（`GlobalTransform(matrix)`）の抽象化
**解決策の方向性**:
- ジェネリック型パラメータ`L: LocalTransform`, `G: GlobalTransform`
- トレイト境界で必要な操作を制約
- 型エイリアスで既存の型を維持

#### GAP3: マーカーコンポーネントの抽象化
**現状**: `TransformTreeChanged`は具体的な型
**要件**: 独自のダーティトラッキング（FR1.3）
**ギャップ**:
- `Component`トレイトの実装が必要
- `.set_changed()`, `.is_changed()`の使用
- ゼロサイズ型（ZST）であることが望ましい
**解決策の方向性**:
- ジェネリック型パラメータ`M: Component`
- トレイトは不要（bevy_ecsの変更検出機能を直接使用）

#### GAP4: クエリシステムの型安全性
**現状**: `NodeQuery`は具体的な型のエイリアス
**要件**: ジェネリック化後も型推論が機能すること（NFR2.2）
**ギャップ**:
- 複雑なネストした型パラメータ
- `Read<T>`と`Mut<T>`のライフタイム管理
- コンパイルエラーメッセージの可読性
**解決策の方向性**:
- 型エイリアスを多用して型シグネチャを簡潔に
- ドキュメントで型パラメータの制約を明確に説明

#### GAP5: パフォーマンス検証
**現状**: ベンチマークが存在しない
**要件**: 性能劣化10%以内（NFR1.1）
**ギャップ**:
- ベースライン測定がない
- ジェネリック化後の比較ができない
**解決策の方向性**:
- 簡易的なベンチマークを作成
- コンパイル時の最適化（モノモーフィゼーション）を確認

#### GAP6: 既存コードの互換性
**現状**: システム関数は未使用
**要件**: 既存コードが変更なしで動作（NFR3.1）
**ギャップ**:
- 実際の使用パターンが不明
- 破壊的変更のリスク評価が困難
**解決策の方向性**:
- 型エイリアスで既存の関数シグネチャを維持
- 新しいジェネリック版を別名で提供し、段階的移行を可能に

### 推奨アプローチ

#### アプローチA: 完全なジェネリック化（理想）
**メリット**: 最大の柔軟性、要件を完全に満たす
**デメリット**: 
- 複雑な型シグネチャ
- bevy_hierarchyへの依存問題
- 大規模な変更
**実現可能性**: 中程度

#### アプローチB: 段階的ジェネリック化（推奨）
**フェーズ1**: 変換型のみジェネリック化（FR1.1, FR1.2, FR1.3）
- `Transform`, `GlobalTransform`, `TransformTreeChanged`を型パラメータ化
- `ChildOf`, `Children`は具体型のまま維持
**フェーズ2**: 階層型のジェネリック化（FR2.1, FR2.2）
- トレイトで階層関係を抽象化
- bevy_hierarchyへの依存を明確化
**メリット**: リスク分散、段階的検証が可能
**デメリット**: 2段階の実装が必要
**実現可能性**: 高い

#### アプローチC: 限定的ジェネリック化（最小）
**範囲**: `Transform`と`GlobalTransform`のみ
**不変**: `ChildOf`, `Children`, `TransformTreeChanged`
**メリット**: 最小限の変更、低リスク
**デメリット**: FR2, FR1.3を満たさない
**実現可能性**: 非常に高い

### 選択基準

| 基準 | アプローチA | アプローチB | アプローチC |
|------|------------|------------|------------|
| 要件充足度 | ★★★★★ | ★★★★☆ | ★★☆☆☆ |
| 実装コスト | ★★☆☆☆ | ★★★★☆ | ★★★★★ |
| リスク管理 | ★★☆☆☆ | ★★★★☆ | ★★★★★ |
| 保守性 | ★★★☆☆ | ★★★★☆ | ★★★★☆ |
| **総合評価** | 65% | **85%** | 70% |

**推奨**: **アプローチB（段階的ジェネリック化）**

理由:
1. 要件の大部分を満たしつつ、リスクを管理できる
2. フェーズ1で効果を検証してからフェーズ2に進める
3. 既存コードへの影響を最小化できる
4. bevy_hierarchyへの依存問題を段階的に解決できる

## 設計

### 設計方針

**採用アプローチ**: アプローチB（段階的ジェネリック化）のフェーズ1

**スコープ**: 変換型のみをジェネリック化
- 対象: `Transform`, `GlobalTransform`, `TransformTreeChanged`
- 非対象: `ChildOf`, `Children`（bevy_ecsの標準型を継続使用）

**設計原則**:
1. **ゼロコスト抽象化**: モノモーフィゼーションによる実行時オーバーヘッドゼロ
2. **後方互換性**: 型エイリアスで既存APIを維持
3. **型安全性**: トレイト境界で制約を明確化
4. **段階的移行**: 既存関数と並存、deprecation警告で移行を促進

### アーキテクチャ設計

#### 層構造

```
┌─────────────────────────────────────────────────┐
│  既存API（後方互換性レイヤー）                    │
│  - sync_simple_transforms (型エイリアス使用)     │
│  - mark_dirty_trees (型エイリアス使用)           │
│  - propagate_parent_transforms (型エイリアス使用)│
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│  ジェネリックAPI（新規実装）                      │
│  - sync_simple_transforms_generic<L, G, M>      │
│  - mark_dirty_trees_generic<L, G, M>            │
│  - propagate_parent_transforms_generic<L, G, M> │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│  トレイト定義                                     │
│  - TransformOps<L, G> (変換操作の抽象化)         │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│  bevy_ecs層                                      │
│  - Component, Query, ParamSet, etc.             │
└─────────────────────────────────────────────────┘
```

### トレイト設計

#### TransformOps トレイト

```rust
/// 変換操作を抽象化するトレイト
/// 
/// # 型パラメータ
/// - `L`: ローカル変換型（例: `Transform`）
/// - `G`: グローバル変換型（例: `GlobalTransform`）
pub trait TransformOps<L, G>
where
    L: Component + Copy,
    G: Component + Copy + PartialEq,
{
    /// ローカル変換からグローバル変換を作成
    fn from_local(local: L) -> G;
    
    /// グローバル変換とローカル変換を合成
    /// 
    /// # 動作
    /// `parent_global * child_local = child_global`
    fn compose(parent_global: G, child_local: L) -> G;
}
```

**設計判断**:
- トレイト境界を最小限に（`Component + Copy + PartialEq`のみ）
- `Into`や`Mul`トレイトへの依存を避け、独自のメソッドで抽象化
- `PartialEq`は`set_if_neq`最適化のために必要

#### デフォルト実装

```rust
/// wintfの標準変換型に対する実装
impl TransformOps<Transform, GlobalTransform> for () {
    fn from_local(local: Transform) -> GlobalTransform {
        GlobalTransform(local.into())
    }
    
    fn compose(parent_global: GlobalTransform, child_local: Transform) -> GlobalTransform {
        parent_global * child_local
    }
}
```

**設計判断**:
- ユニット型`()`をマーカーとして使用
- 既存の`Mul`実装を活用
- 追加の型定義を避けシンプルに

### 型パラメータ設計

#### ジェネリック関数シグネチャ

```rust
pub fn sync_simple_transforms_generic<L, G, M, Ops>(
    mut query: ParamSet<(
        Query<
            (&L, &mut G),
            (
                Or<(Changed<L>, Added<G>)>,
                Without<ChildOf>,
                Without<Children>,
            ),
        >,
        Query<(Ref<L>, &mut G), (Without<ChildOf>, Without<Children>)>,
    )>,
    mut orphaned: RemovedComponents<ChildOf>,
) where
    L: Component + Copy,
    G: Component + Copy + PartialEq,
    M: Component,
    Ops: TransformOps<L, G>,
```

**型パラメータ**:
- `L`: Local transform（ローカル変換）
- `G`: Global transform（グローバル変換）
- `M`: Marker component（ダーティマーカー）
- `Ops`: Transform operations（変換操作）

**トレイト境界の根拠**:
- `Component`: bevy_ecsの必須要件
- `Copy`: 値渡しによる効率的な処理
- `PartialEq` (Gのみ): `set_if_neq`最適化

#### NodeQuery のジェネリック化

```rust
type NodeQuery<'w, 's, L, G, M> = Query<
    'w,
    's,
    (
        Entity,
        (
            Ref<'static, L>,
            Mut<'static, G>,
            Ref<'static, M>,
        ),
        (Option<Read<Children>>, Read<ChildOf>),
    ),
>;
```

**設計判断**:
- 階層コンポーネント（`ChildOf`, `Children`）は固定型のまま
- ライフタイム`'static`は`lifetimeless`モジュールの要件

### 後方互換性設計

#### 型エイリアスによる既存API維持

```rust
/// 既存APIの型エイリアス（後方互換性のため）
pub fn sync_simple_transforms(
    query: ParamSet<(
        Query<
            (&Transform, &mut GlobalTransform),
            (
                Or<(Changed<Transform>, Added<GlobalTransform>)>,
                Without<ChildOf>,
                Without<Children>,
            ),
        >,
        Query<(Ref<Transform>, &mut GlobalTransform), (Without<ChildOf>, Without<Children>)>,
    )>,
    orphaned: RemovedComponents<ChildOf>,
) {
    sync_simple_transforms_generic::<Transform, GlobalTransform, TransformTreeChanged, ()>(
        query,
        orphaned,
    )
}
```

**移行戦略**:
1. 既存関数を残し、内部でジェネリック版を呼び出す
2. `#[deprecated]`属性は初期段階では付与しない
3. ドキュメントでジェネリック版の使用を推奨

### WorkQueue 設計

**判断**: ジェネリック化不要

**理由**:
- `Entity`ベースのキューは変換型に依存しない
- 並列処理ロジックは型に依存しない
- 変更はコストに見合わない

```rust
/// 変換伝播のためにスレッド間で共有されるキュー。
/// （変更なし）
pub struct WorkQueue {
    busy_threads: AtomicI32,
    sender: Sender<Vec<Entity>>,
    receiver: Arc<Mutex<Receiver<Vec<Entity>>>>,
    local_queue: Parallel<Vec<Entity>>,
}
```

### unsafeコード設計

**方針**: セマンティクスを変更しない

#### propagate_descendants_unchecked のジェネリック化

```rust
#[inline]
#[expect(unsafe_code, reason = "Mutating disjoint entities in parallel")]
unsafe fn propagate_descendants_unchecked<L, G, M, Ops>(
    parent: Entity,
    p_global_transform: Mut<G>,
    p_children: &Children,
    nodes: &NodeQuery<L, G, M>,
    outbox: &mut Vec<Entity>,
    queue: &WorkQueue,
    max_depth: usize,
) where
    L: Component + Copy,
    G: Component + Copy + PartialEq,
    M: Component,
    Ops: TransformOps<L, G>,
{
    // 既存のロジックをそのまま維持
    // Ops::from_local(), Ops::compose()を使用
}
```

**安全性保証**:
- 既存のSAFETYコメントをそのまま維持
- 型パラメータの追加は健全性に影響しない
- `Entity`の一意性に基づく並列性は変更なし

### パフォーマンス設計

#### モノモーフィゼーション戦略

```rust
// コンパイラは使用される型ごとに専用コードを生成
sync_simple_transforms_generic::<Transform, GlobalTransform, TransformTreeChanged, ()>
// ↓ コンパイル時に展開
sync_simple_transforms_generic_Transform_GlobalTransform_TransformTreeChanged_unit
```

**最適化ポイント**:
1. `#[inline]`属性の適切な配置
2. `Copy`トレイト境界によるメモリコピー最小化
3. `PartialEq`による早期リターン（`set_if_neq`）

#### ベンチマーク設計

```rust
// tests/benches/transform_system_bench.rs（新規作成）
#[cfg(test)]
mod benches {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn bench_propagate_existing(c: &mut Criterion) {
        // 既存の実装（型エイリアス経由）
    }
    
    fn bench_propagate_generic(c: &mut Criterion) {
        // ジェネリック版の直接呼び出し
    }
}
```

**測定指標**:
- 実行時間（マイクロ秒単位）
- コンパイル時間（秒単位）
- バイナリサイズ（バイト単位）

### ドキュメント設計

#### モジュールレベルドキュメント

```rust
//! # Transform System (Generic)
//!
//! 階層構造を持つエンティティの変換を効率的に伝播するシステム。
//!
//! ## 使用方法
//!
//! ### 標準的な使用（wintfの型）
//!
//! ```ignore
//! world.add_systems(Update, sync_simple_transforms);
//! world.add_systems(Update, mark_dirty_trees);
//! world.add_systems(Update, propagate_parent_transforms);
//! ```
//!
//! ### カスタム変換型での使用
//!
//! ```ignore
//! // 1. カスタム型を定義
//! #[derive(Component, Clone, Copy)]
//! struct MyTransform { /* ... */ }
//!
//! #[derive(Component, Clone, Copy, PartialEq)]
//! struct MyGlobalTransform { /* ... */ }
//!
//! // 2. TransformOps を実装
//! struct MyOps;
//! impl TransformOps<MyTransform, MyGlobalTransform> for MyOps {
//!     fn from_local(local: MyTransform) -> MyGlobalTransform { /* ... */ }
//!     fn compose(parent: MyGlobalTransform, child: MyTransform) -> MyGlobalTransform { /* ... */ }
//! }
//!
//! // 3. ジェネリック版を使用
//! world.add_systems(Update, 
//!     sync_simple_transforms_generic::<MyTransform, MyGlobalTransform, MyMarker, MyOps>
//! );
//! ```
```

#### 関数レベルドキュメント

```rust
/// 階層を持たないエンティティの変換を更新する（ジェネリック版）。
///
/// # 型パラメータ
///
/// - `L`: ローカル変換コンポーネント（例: `Transform`）
/// - `G`: グローバル変換コンポーネント（例: `GlobalTransform`）
/// - `M`: ダーティトラッキング用マーカー（例: `TransformTreeChanged`）
/// - `Ops`: 変換操作を提供する型（例: `()`）
///
/// # トレイト境界
///
/// - `L: Component + Copy` - ECSコンポーネントで値渡し可能
/// - `G: Component + Copy + PartialEq` - 等価比較による最適化が可能
/// - `M: Component` - ダーティトラッキング用マーカー
/// - `Ops: TransformOps<L, G>` - 変換操作を提供
///
/// # 使用例
///
/// ```ignore
/// sync_simple_transforms_generic::<Transform, GlobalTransform, TransformTreeChanged, ()>(
///     query,
///     orphaned,
/// );
/// ```
pub fn sync_simple_transforms_generic<L, G, M, Ops>(/* ... */) { /* ... */ }
```

### エラーハンドリング設計

**方針**: コンパイル時エラーを優先

#### トレイト境界違反

```rust
// コンパイルエラー例
struct NonCopyTransform { data: Vec<f32> }

sync_simple_transforms_generic::<NonCopyTransform, GlobalTransform, TransformTreeChanged, ()>
// ↓
// error[E0277]: the trait bound `NonCopyTransform: Copy` is not satisfied
```

#### 実行時エラー

```rust
// パニック: 既存の動作を維持
assert_eq!(child_of.parent(), parent);
// ↓
// thread 'main' panicked at 'assertion failed: child_of.parent() == parent'
```

### テスト設計

#### 単体テスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transform_ops_default() {
        let local = Transform::default();
        let global = <()>::from_local(local);
        assert_eq!(global, GlobalTransform::default());
    }
    
    #[test]
    fn test_compose() {
        let parent = GlobalTransform(Matrix3x2::translation(10.0, 20.0));
        let child = Transform {
            translate: Translate::new(5.0, 5.0),
            ..Default::default()
        };
        let result = <()>::compose(parent, child);
        // 期待値の検証
    }
}
```

#### 統合テスト

```rust
// tests/transform_system_integration.rs
#[test]
fn test_generic_transform_system_with_hierarchy() {
    let mut world = World::new();
    
    // エンティティ階層を構築
    let parent = world.spawn((
        Transform::default(),
        GlobalTransform::default(),
        TransformTreeChanged,
    )).id();
    
    let child = world.spawn((
        Transform { translate: Translate::new(10.0, 10.0), ..Default::default() },
        GlobalTransform::default(),
        TransformTreeChanged,
        ChildOf::new(parent),
    )).id();
    
    // システムを実行
    world.run_system_once(sync_simple_transforms_generic::<_, _, _, ()>);
    world.run_system_once(mark_dirty_trees_generic::<_, _, _, ()>);
    world.run_system_once(propagate_parent_transforms_generic::<_, _, _, ()>);
    
    // 結果を検証
    let child_transform = world.get::<GlobalTransform>(child).unwrap();
    // アサーション
}
```

### 実装順序

#### Phase 1-1: 基盤実装（最優先）
1. `TransformOps`トレイト定義
2. デフォルト実装（`()`型）
3. 型エイリアスの準備

#### Phase 1-2: システム関数のジェネリック化
1. `sync_simple_transforms_generic`
2. `mark_dirty_trees_generic`
3. `propagate_parent_transforms_generic`
4. `propagation_worker_generic`（内部関数）
5. `propagate_descendants_unchecked_generic`（内部関数）

#### Phase 1-3: 後方互換性レイヤー
1. 既存関数を型エイリアス経由に変更
2. ドキュメント更新
3. 使用例の追加

#### Phase 1-4: テスト実装
1. 単体テスト（トレイト実装）
2. 統合テスト（システム動作）
3. ベンチマーク（パフォーマンス検証）

### リスクと緩和策

| リスク | 影響度 | 緩和策 |
|--------|--------|--------|
| 複雑な型エラーメッセージ | 中 | ドキュメントで明確な例を提供、型エイリアスで簡潔化 |
| コンパイル時間の増加 | 低 | ベンチマークで測定、20%以内を目標 |
| bevy_ecs APIの変更 | 低 | 0.17.2に固定、破壊的変更に備えた抽象化 |
| 既存コードの破壊 | 極低 | 型エイリアスで完全な後方互換性を保証 |

### 成果物

#### 新規ファイル
- `tests/benches/transform_system_bench.rs` - ベンチマーク
- `tests/transform_system_integration.rs` - 統合テスト

#### 変更ファイル
- `crates/wintf/src/ecs/transform_system.rs` - ジェネリック実装追加
- `crates/wintf/src/ecs/mod.rs` - エクスポート調整（必要に応じて）

#### ドキュメント
- モジュールドキュメント（`transform_system.rs`冒頭）
- 関数ドキュメント（各関数）
- 使用例（doctest）

## タスク分解（簡素化版）

### 全体概要

**1つのタスク**: 既存の3つの関数に型パラメータを追加するだけ

### Task 1: 型パラメータの追加

**ファイル**: `crates/wintf/src/ecs/transform_system.rs`

#### 変更内容

**1. 関数シグネチャに型パラメータを追加**

```rust
// Before
pub fn sync_simple_transforms(
    mut query: ParamSet<(
        Query<(&Transform, &mut GlobalTransform), ...>,
        ...
    )>,
    mut orphaned: RemovedComponents<ChildOf>,
)

// After
pub fn sync_simple_transforms<L, G, M>(
    mut query: ParamSet<(
        Query<(&L, &mut G), ...>,
        ...
    )>,
    mut orphaned: RemovedComponents<ChildOf>,
) where
    L: Component + Copy + Into<G>,
    G: Component + Copy + PartialEq + Mul<L, Output = G>,
    M: Component,
```

**2. 同様に他の関数も型パラメータ化**

- `mark_dirty_trees<L, G, M>(...)`
- `propagate_parent_transforms<L, G, M>(...)`
- `propagation_worker<L, G, M>(...)`（内部関数）
- `propagate_descendants_unchecked<L, G, M>(...)`（内部関数）

**3. 型エイリアスの更新**

```rust
// Before
type NodeQuery<'w, 's> = Query<...>;

// After
type NodeQuery<'w, 's, L, G, M> = Query<...>;
```

**4. 変換処理の微調整**

```rust
// Before
*global_transform = GlobalTransform((*transform).into());

// After  
*global_transform = (*transform).into();
```

#### 成果物

- 型パラメータ化された5つの関数
- 更新された型エイリアス
- 動作する既存のコード

#### 工数

**2-3時間**

#### 検証

1. `cargo build` が通る
2. 既存のサンプルが動作する（現状は未使用だが）
3. 型推論が正しく機能する

### それだけ！

複雑なトレイトやラッパーは不要。既存のRustの型システム（`Into`, `Mul`）で十分対応可能。

## フェーズ

### Phase 1: 仕様策定
- [x] 初期化完了
- [x] 要件定義完了（簡素化版）
- [x] ギャップ分析完了
- [x] 設計完了（簡素化版）
- [x] タスク分解完了（簡素化版）

### Phase 2: 実装
- [ ] 型パラメータの追加（2-3時間）

## 次のステップ

実装を開始:
- 既存の3つの関数に型パラメータ `<L, G, M>` を追加
- トレイト境界を追加
- 型の置換（`Transform` → `L`, `GlobalTransform` → `G`, `TransformTreeChanged` → `M`）
- ビルド確認

---
_最終更新: 2025-11-14_
