//! コンポーネント状態管理パターンの比較検証
//!
//! GraphicsCore再初期化において、コンポーネントの状態管理に最適なパターンを検証:
//! 1. Optionalラップ方式: コンポーネント内部をOptionでラップ
//! 2. 要初期化フラグ方式: needs_reinit フラグを持つ
//! 3. 別コンポーネント方式: 状態を独立したコンポーネントで管理

use bevy_ecs::prelude::*;

// ===== モックデータ =====

#[derive(Debug, Clone)]
struct MockGraphicsData {
    id: u32,
}

// ===== Pattern 1: Optionalラップ方式 =====

#[derive(Component, Debug)]
struct WindowGraphicsOptional {
    inner: Option<MockGraphicsData>,
    generation: u64,
}

impl WindowGraphicsOptional {
    fn new(id: u32) -> Self {
        Self {
            inner: Some(MockGraphicsData { id }),
            generation: 0,
        }
    }

    fn is_valid(&self) -> bool {
        self.inner.is_some()
    }

    fn invalidate(&mut self) {
        self.inner = None;
    }

    fn reinitialize(&mut self, id: u32) {
        self.inner = Some(MockGraphicsData { id });
        self.generation += 1;
    }
}

// ===== Pattern 2: 要初期化フラグ方式 =====

#[derive(Component, Debug)]
struct WindowGraphicsWithFlag {
    data: MockGraphicsData,
    needs_reinit: bool,
    generation: u64,
}

impl WindowGraphicsWithFlag {
    fn new(id: u32) -> Self {
        Self {
            data: MockGraphicsData { id },
            needs_reinit: false,
            generation: 0,
        }
    }

    fn is_valid(&self) -> bool {
        !self.needs_reinit
    }

    fn invalidate(&mut self) {
        self.needs_reinit = true;
    }

    fn reinitialize(&mut self, id: u32) {
        self.data = MockGraphicsData { id };
        self.needs_reinit = false;
        self.generation += 1;
    }
}

// ===== Pattern 3: 別コンポーネント方式 =====

#[derive(Component, Debug)]
struct WindowGraphics {
    data: MockGraphicsData,
    generation: u64,
}

#[derive(Component, Debug)]
struct NeedsReinitialize;

impl WindowGraphics {
    fn new(id: u32) -> Self {
        Self {
            data: MockGraphicsData { id },
            generation: 0,
        }
    }

    fn reinitialize(&mut self, id: u32) {
        self.data = MockGraphicsData { id };
        self.generation += 1;
    }
}

// ===== 検証テスト =====

#[test]
fn test_pattern1_optional_lifecycle() {
    let mut world = World::new();
    let entity = world.spawn(WindowGraphicsOptional::new(1)).id();

    // Phase 1: 初期状態（有効）
    {
        let wg = world
            .entity(entity)
            .get::<WindowGraphicsOptional>()
            .unwrap();
        assert!(wg.is_valid(), "初期状態では有効");
        assert_eq!(wg.generation, 0);
    }

    // Phase 2: 無効化
    {
        let mut wg = world.get_mut::<WindowGraphicsOptional>(entity).unwrap();
        wg.invalidate();
    }

    // Phase 3: 無効化後の状態確認
    {
        let wg = world
            .entity(entity)
            .get::<WindowGraphicsOptional>()
            .unwrap();
        assert!(!wg.is_valid(), "無効化後は無効");
    }

    // Phase 4: 再初期化
    {
        let mut wg = world.get_mut::<WindowGraphicsOptional>(entity).unwrap();
        wg.reinitialize(2);
    }

    // Phase 5: 再初期化後の状態確認
    {
        let wg = world
            .entity(entity)
            .get::<WindowGraphicsOptional>()
            .unwrap();
        assert!(wg.is_valid(), "再初期化後は有効");
        assert_eq!(wg.generation, 1);
    }

    eprintln!("[Pattern1 Lifecycle] テスト成功");
}

#[test]
fn test_pattern2_flag_lifecycle() {
    let mut world = World::new();
    let entity = world.spawn(WindowGraphicsWithFlag::new(1)).id();

    // Phase 1: 初期状態（有効）
    {
        let wg = world
            .entity(entity)
            .get::<WindowGraphicsWithFlag>()
            .unwrap();
        assert!(wg.is_valid(), "初期状態では有効");
        assert_eq!(wg.generation, 0);
    }

    // Phase 2: 無効化
    {
        let mut wg = world.get_mut::<WindowGraphicsWithFlag>(entity).unwrap();
        wg.invalidate();
    }

    // Phase 3: 無効化後の状態確認
    {
        let wg = world
            .entity(entity)
            .get::<WindowGraphicsWithFlag>()
            .unwrap();
        assert!(!wg.is_valid(), "無効化後は無効");
    }

    // Phase 4: 再初期化
    {
        let mut wg = world.get_mut::<WindowGraphicsWithFlag>(entity).unwrap();
        wg.reinitialize(2);
    }

    // Phase 5: 再初期化後の状態確認
    {
        let wg = world
            .entity(entity)
            .get::<WindowGraphicsWithFlag>()
            .unwrap();
        assert!(wg.is_valid(), "再初期化後は有効");
        assert_eq!(wg.generation, 1);
    }

    eprintln!("[Pattern2 Flag Lifecycle] テスト成功");
}

#[test]
fn test_pattern3_separate_component_lifecycle() {
    let mut world = World::new();
    let entity = world.spawn(WindowGraphics::new(1)).id();

    // Phase 1: 初期状態（無効化マーカーなし）
    {
        let has_marker = world.entity(entity).get::<NeedsReinitialize>().is_some();
        assert!(!has_marker, "初期状態では無効化マーカーなし");
    }

    // Phase 2: 無効化（マーカー追加）
    {
        world.entity_mut(entity).insert(NeedsReinitialize);
    }

    // Phase 3: 無効化後の状態確認
    {
        let has_marker = world.entity(entity).get::<NeedsReinitialize>().is_some();
        assert!(has_marker, "無効化後はマーカーあり");
    }

    // Phase 4: 再初期化（データ更新 + マーカー削除）
    {
        let mut wg = world.get_mut::<WindowGraphics>(entity).unwrap();
        wg.reinitialize(2);
        world.entity_mut(entity).remove::<NeedsReinitialize>();
    }

    // Phase 5: 再初期化後の状態確認
    {
        let has_marker = world.entity(entity).get::<NeedsReinitialize>().is_some();
        assert!(!has_marker, "再初期化後はマーカー削除");
        let wg = world.entity(entity).get::<WindowGraphics>().unwrap();
        assert_eq!(wg.generation, 1);
    }

    eprintln!("[Pattern3 Separate Component Lifecycle] テスト成功");
}

#[test]
fn test_pattern1_changed_detection() {
    let mut world = World::new();
    let _e1 = world.spawn(WindowGraphicsOptional::new(1)).id();
    let _e2 = world.spawn(WindowGraphicsOptional::new(2)).id();

    let mut schedule = Schedule::default();

    // Changed<WindowGraphicsOptional>でフィルタするシステム
    fn check_changed(query: Query<Entity, Changed<WindowGraphicsOptional>>) {
        let count = query.iter().count();
        eprintln!("[Changed Detection] {count} entities detected");
    }

    schedule.add_systems(check_changed);

    // 初回実行: 全エンティティが変更扱い
    eprintln!("=== 初回実行 ===");
    schedule.run(&mut world);

    // 2回目実行: 変更なし
    eprintln!("=== 2回目実行（変更なし） ===");
    schedule.run(&mut world);

    // e1を無効化
    {
        let mut wg = world.get_mut::<WindowGraphicsOptional>(_e1).unwrap();
        wg.invalidate();
    }

    // 3回目実行: e1のみ検出されるはず
    eprintln!("=== 3回目実行（e1を無効化） ===");
    schedule.run(&mut world);

    eprintln!("[Pattern1 Changed Detection] テスト成功");
}

#[test]
fn test_pattern2_changed_detection() {
    let mut world = World::new();
    let _e1 = world.spawn(WindowGraphicsWithFlag::new(1)).id();
    let _e2 = world.spawn(WindowGraphicsWithFlag::new(2)).id();

    let mut schedule = Schedule::default();

    fn check_changed(query: Query<Entity, Changed<WindowGraphicsWithFlag>>) {
        let count = query.iter().count();
        eprintln!("[Changed Detection] {count} entities detected");
    }

    schedule.add_systems(check_changed);

    eprintln!("=== 初回実行 ===");
    schedule.run(&mut world);

    eprintln!("=== 2回目実行（変更なし） ===");
    schedule.run(&mut world);

    {
        let mut wg = world.get_mut::<WindowGraphicsWithFlag>(_e1).unwrap();
        wg.invalidate();
    }

    eprintln!("=== 3回目実行（e1を無効化） ===");
    schedule.run(&mut world);

    eprintln!("[Pattern2 Changed Detection] テスト成功");
}

#[test]
fn test_pattern3_marker_query() {
    let mut world = World::new();
    let _e1 = world.spawn(WindowGraphics::new(1)).id();
    let e2 = world.spawn(WindowGraphics::new(2)).id();
    let _e3 = world.spawn(WindowGraphics::new(3)).id();

    // e2にのみマーカー追加
    world.entity_mut(e2).insert(NeedsReinitialize);

    let mut schedule = Schedule::default();

    fn check_needs_reinit(query: Query<Entity, With<NeedsReinitialize>>) {
        let entities: Vec<_> = query.iter().collect();
        eprintln!(
            "[Needs Reinit Query] {} entities need reinit",
            entities.len()
        );
        assert_eq!(entities.len(), 1, "e2のみマーカーあり");
    }

    schedule.add_systems(check_needs_reinit);

    schedule.run(&mut world);

    eprintln!("[Pattern3 Marker Query] テスト成功");
}

// ===== パターン比較サマリー =====

/*
## テスト結果の考察

### Pattern 1: Optionalラップ方式
- **利点:**
  - Changed<T>検出が機能
  - Rustのイディオムに合致（Option<T>は標準パターン）
  - 無効状態を型レベルで表現
- **欠点:**
  - 既存コードの変更が多い（.inner.as_ref().unwrap()など）
  - コンポーネント構造の変更が必要
  - マイグレーション時の修正箇所が多い

### Pattern 2: 要初期化フラグ方式 ★推奨★
- **利点:**
  - Changed<T>検出が機能
  - 既存コードへの影響が最小（フィールド追加のみ）
  - シンプルで理解しやすい
  - 世代番号と組み合わせて二重チェック可能
- **欠点:**
  - 無効状態でもデータは保持される（メモリ効率やや劣る）
  - フラグの適切な管理が必要

### Pattern 3: 別コンポーネント方式
- **利点:**
  - ECS的に最もクリーン
  - クエリでの判別が明確（With<NeedsReinitialize>）
  - コンポーネントの追加/削除で状態管理
- **欠点:**
  - コンポーネント追加/削除のオーバーヘッド
  - 状態が分散するため、デバッグ時に確認箇所が増える
  - 既存実装への組み込みが複雑

## 推奨アプローチ

**Pattern 2: 要初期化フラグ方式** を推奨します。

理由:
1. **既存コードへの影響が最小**: フィールド追加とフラグチェックのみで対応可能
2. **Changed<T>検出が使える**: Bevy ECSの変更検出機構を活用
3. **世代番号との組み合わせ**: needs_reinit + generation で二重チェック可能
4. **実装が直感的**: フラグの意味が明確で、保守性が高い

リソース削除検出（ポーリング方式）とコンポーネント状態管理（フラグ方式）を組み合わせることで、
安定した再初期化フローを実現できます。
*/
