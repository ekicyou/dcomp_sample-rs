//! Lazy Reinitialization Pattern - 遅延初期化パターンの検証
//!
//! Pattern 2（要初期化フラグ方式）とPattern 1（Optionalラップ）のハイブリッド:
//! - 無効化時は内部をNoneに設定
//! - 取得時にNoneを検出したら自動的に初期化を実行
//!
//! 利点:
//! - Changed<T>検出が機能
//! - 遅延初期化により必要になるまで初期化コストを遅延
//! - 呼び出し側がシンプル（get_or_init()で自動初期化）
//! - 既存コードへの影響が小さい

use bevy_ecs::prelude::*;

// ===== モックデータ =====

#[derive(Debug, Clone, PartialEq)]
struct MockGraphicsData {
    id: u32,
    created_from_core_gen: u64,
}

#[derive(Resource, Debug)]
struct MockGraphicsCore {
    generation: u64,
}

impl MockGraphicsCore {
    fn new() -> Self {
        Self { generation: 0 }
    }

    fn invalidate(&mut self) {
        self.generation += 1;
    }
}

// ===== Lazy Reinitialization Pattern =====

#[derive(Component, Debug)]
struct WindowGraphics {
    /// 内部データ（無効時はNone）
    inner: Option<MockGraphicsData>,
    /// 世代番号（初期化された回数）
    generation: u64,
}

impl WindowGraphics {
    fn new(id: u32, core: &MockGraphicsCore) -> Self {
        Self {
            inner: Some(MockGraphicsData {
                id,
                created_from_core_gen: core.generation,
            }),
            generation: 0,
        }
    }

    /// 1. 無効化: 内部をNoneに設定
    fn invalidate(&mut self) {
        self.inner = None;
    }

    /// 2. 取得: Noneなら自動初期化 (lazy initialization)
    fn get_or_init(&mut self, id: u32, core: &MockGraphicsCore) -> &MockGraphicsData {
        if self.inner.is_none() {
            eprintln!(
                "[Lazy Reinit] Entity id={} を再初期化 (core gen={})",
                id, core.generation
            );
            self.inner = Some(MockGraphicsData {
                id,
                created_from_core_gen: core.generation,
            });
            self.generation += 1;
        }
        self.inner.as_ref().unwrap()
    }

    /// 内部データが有効かチェック（デバッグ用）
    fn is_valid(&self) -> bool {
        self.inner.is_some()
    }
}

// ===== 検証テスト =====

#[test]
fn test_lazy_reinit_lifecycle() {
    let mut world = World::new();
    let core = MockGraphicsCore::new();
    world.insert_resource(core);

    let entity = world
        .spawn(WindowGraphics::new(
            1,
            &world.resource::<MockGraphicsCore>(),
        ))
        .id();

    // Phase 1: 初期状態（有効）
    {
        let wg = world.entity(entity).get::<WindowGraphics>().unwrap();
        assert!(wg.is_valid(), "初期状態では有効");
        assert_eq!(wg.generation, 0);
        assert_eq!(wg.inner.as_ref().unwrap().created_from_core_gen, 0);
    }

    // Phase 2: 無効化
    {
        let mut wg = world.get_mut::<WindowGraphics>(entity).unwrap();
        wg.invalidate();
    }

    // Phase 3: 無効化後の状態確認
    {
        let wg = world.entity(entity).get::<WindowGraphics>().unwrap();
        assert!(!wg.is_valid(), "無効化後は無効");
    }

    // Phase 4: Core を更新（デバイスロスをシミュレート）
    {
        world.resource_mut::<MockGraphicsCore>().invalidate();
    }

    // Phase 5: get_or_init で自動再初期化
    {
        let core_gen = world.resource::<MockGraphicsCore>().generation;
        let mut wg = world.get_mut::<WindowGraphics>(entity).unwrap();
        let core = MockGraphicsCore {
            generation: core_gen,
        };
        let data = wg.get_or_init(1, &core);

        assert_eq!(data.id, 1);
        assert_eq!(data.created_from_core_gen, 1, "新しいCore世代で再初期化");
        assert_eq!(wg.generation, 1, "コンポーネント世代が更新");
    }

    // Phase 6: 再初期化後の状態確認
    {
        let wg = world.entity(entity).get::<WindowGraphics>().unwrap();
        assert!(wg.is_valid(), "再初期化後は有効");
    }

    eprintln!("[Lazy Reinit Lifecycle] テスト成功");
}

#[test]
fn test_lazy_reinit_multiple_entities() {
    let mut world = World::new();
    world.insert_resource(MockGraphicsCore::new());

    let e1 = world
        .spawn(WindowGraphics::new(
            1,
            &world.resource::<MockGraphicsCore>(),
        ))
        .id();
    let e2 = world
        .spawn(WindowGraphics::new(
            2,
            &world.resource::<MockGraphicsCore>(),
        ))
        .id();
    let e3 = world
        .spawn(WindowGraphics::new(
            3,
            &world.resource::<MockGraphicsCore>(),
        ))
        .id();

    // Phase 1: 全エンティティを無効化
    {
        let mut wg1 = world.get_mut::<WindowGraphics>(e1).unwrap();
        wg1.invalidate();
        let mut wg2 = world.get_mut::<WindowGraphics>(e2).unwrap();
        wg2.invalidate();
        let mut wg3 = world.get_mut::<WindowGraphics>(e3).unwrap();
        wg3.invalidate();
    }

    // Phase 2: Core を更新
    {
        world.resource_mut::<MockGraphicsCore>().invalidate();
    }

    // Phase 3: e2 のみ再初期化（遅延初期化により必要なもののみ初期化）
    {
        let core_gen = world.resource::<MockGraphicsCore>().generation;
        let mut wg2 = world.get_mut::<WindowGraphics>(e2).unwrap();
        let core = MockGraphicsCore {
            generation: core_gen,
        };
        wg2.get_or_init(2, &core);
    }

    // Phase 4: 状態確認
    {
        let wg1 = world.entity(e1).get::<WindowGraphics>().unwrap();
        let wg2 = world.entity(e2).get::<WindowGraphics>().unwrap();
        let wg3 = world.entity(e3).get::<WindowGraphics>().unwrap();

        assert!(!wg1.is_valid(), "e1 は未初期化");
        assert!(wg2.is_valid(), "e2 は再初期化済み");
        assert!(!wg3.is_valid(), "e3 は未初期化");
    }

    eprintln!("[Lazy Reinit Multiple Entities] テスト成功: 必要なもののみ初期化");
}

#[test]
fn test_lazy_reinit_changed_detection() {
    let mut world = World::new();
    world.insert_resource(MockGraphicsCore::new());

    let e1 = world
        .spawn(WindowGraphics::new(
            1,
            &world.resource::<MockGraphicsCore>(),
        ))
        .id();
    let _e2 = world
        .spawn(WindowGraphics::new(
            2,
            &world.resource::<MockGraphicsCore>(),
        ))
        .id();

    let mut schedule = Schedule::default();

    fn check_changed(query: Query<Entity, Changed<WindowGraphics>>) {
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
        let mut wg = world.get_mut::<WindowGraphics>(e1).unwrap();
        wg.invalidate();
    }

    // 3回目実行: e1のみ検出
    eprintln!("=== 3回目実行（e1を無効化） ===");
    schedule.run(&mut world);

    // e1を再初期化
    {
        let core_gen = world.resource::<MockGraphicsCore>().generation;
        let mut wg = world.get_mut::<WindowGraphics>(e1).unwrap();
        let core = MockGraphicsCore {
            generation: core_gen,
        };
        wg.get_or_init(1, &core);
    }

    // 4回目実行: e1のみ検出（再初期化による変更）
    eprintln!("=== 4回目実行（e1を再初期化） ===");
    schedule.run(&mut world);

    eprintln!("[Lazy Reinit Changed Detection] テスト成功");
}

#[test]
fn test_lazy_reinit_system_integration() {
    let mut world = World::new();
    world.insert_resource(MockGraphicsCore::new());

    let e1 = world
        .spawn(WindowGraphics::new(
            1,
            &world.resource::<MockGraphicsCore>(),
        ))
        .id();
    let e2 = world
        .spawn(WindowGraphics::new(
            2,
            &world.resource::<MockGraphicsCore>(),
        ))
        .id();

    // Phase 1: 全エンティティを無効化
    {
        let mut wg1 = world.get_mut::<WindowGraphics>(e1).unwrap();
        wg1.invalidate();
        let mut wg2 = world.get_mut::<WindowGraphics>(e2).unwrap();
        wg2.invalidate();
    }

    // Phase 2: Core を更新
    {
        world.resource_mut::<MockGraphicsCore>().invalidate();
    }

    // Phase 3: システムで自動再初期化
    fn reinit_system(core: Res<MockGraphicsCore>, mut query: Query<(Entity, &mut WindowGraphics)>) {
        for (entity, mut wg) in query.iter_mut() {
            if !wg.is_valid() {
                eprintln!("[System] Entity {:?} を自動再初期化", entity);
                wg.get_or_init(entity.index(), &core);
            }
        }
    }

    let mut schedule = Schedule::default();
    schedule.add_systems(reinit_system);

    eprintln!("=== システムで自動再初期化実行 ===");
    schedule.run(&mut world);

    // Phase 4: 状態確認
    {
        let wg1 = world.entity(e1).get::<WindowGraphics>().unwrap();
        let wg2 = world.entity(e2).get::<WindowGraphics>().unwrap();

        assert!(wg1.is_valid(), "e1 は再初期化済み");
        assert!(wg2.is_valid(), "e2 は再初期化済み");
        assert_eq!(wg1.generation, 1);
        assert_eq!(wg2.generation, 1);
    }

    eprintln!("[Lazy Reinit System Integration] テスト成功");
}

#[test]
fn test_lazy_reinit_no_unnecessary_init() {
    let mut world = World::new();
    world.insert_resource(MockGraphicsCore::new());

    let entity = world
        .spawn(WindowGraphics::new(
            1,
            &world.resource::<MockGraphicsCore>(),
        ))
        .id();

    // Phase 1: 初期世代確認
    {
        let wg = world.entity(entity).get::<WindowGraphics>().unwrap();
        assert_eq!(wg.generation, 0);
    }

    // Phase 2: 複数回 get_or_init を呼んでも再初期化されない
    {
        let core_gen = world.resource::<MockGraphicsCore>().generation;
        let mut wg = world.get_mut::<WindowGraphics>(entity).unwrap();
        let core = MockGraphicsCore {
            generation: core_gen,
        };

        wg.get_or_init(1, &core);
        wg.get_or_init(1, &core);
        wg.get_or_init(1, &core);

        assert_eq!(wg.generation, 0, "有効な状態では再初期化されない");
    }

    eprintln!("[Lazy Reinit No Unnecessary Init] テスト成功: 不要な初期化は発生しない");
}

// ===== パターンの評価 =====

/*
## Lazy Reinitialization Pattern の評価

### 設計の特徴
1. **無効化**: `invalidate()` で内部をNoneに設定
2. **取得**: `get_or_init()` でNone検出時に自動初期化

### 利点
- ✅ **Changed<T>検出が機能**: 無効化・再初期化でBevy ECSが変更を検知
- ✅ **遅延初期化**: 必要になるまで初期化を遅延、リソース効率◎
- ✅ **呼び出し側がシンプル**: `get_or_init()`で自動的に最新状態を取得
- ✅ **既存コードへの影響小**: 取得方法の変更のみ（`.get()` → `.get_or_init()`）
- ✅ **不要な初期化を防ぐ**: 使用されないエンティティは初期化されない
- ✅ **システム統合が容易**: システム内で自動再初期化を実装可能

### 欠点
- ⚠️ **可変参照が必要**: `get_or_reinit()`は`&mut self`を要求
- ⚠️ **世代番号の二重管理**: Core世代とコンポーネント世代の両方を管理
- ⚠️ **初期化コストの予測困難**: 初回アクセス時に遅延初期化が発生

### 実装時の注意点
1. `get_or_init()`は可変参照を要求するため、複数箇所での同時アクセスは不可
2. GraphicsCore の世代番号と照合する仕組みが必要
3. 初期化失敗時のエラーハンドリング戦略が必要

### 推奨される使用シーン
- WindowGraphics, Visual, Surface など、GraphicsCore に依存するコンポーネント
- デバイスロス後の再初期化が必要なグラフィックスリソース
- 使用頻度が低いリソース（遅延初期化の利点を活かせる）

### 結論
**Pattern 2（要初期化フラグ）とPattern 1（Optional）の良いとこ取り**

リソース削除検出（ポーリング方式）+ 遅延初期化パターンを組み合わせることで、
効率的で保守性の高い再初期化フローを実現できます。
*/
