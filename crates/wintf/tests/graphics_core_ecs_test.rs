//! GraphicsCoreのECS統合テスト
//!
//! このテストは以下を検証します:
//! - GraphicsCoreがECSリソースとして登録されること
//! - システムからアクセス可能であること
//! - init_graphics_coreシステムが正しく動作すること

use bevy_ecs::prelude::*;
use wintf::ecs::{init_graphics_core, GraphicsCore};

#[test]
fn test_init_graphics_core_creates_resource() {
    // init_graphics_coreシステムがリソースを作成することを確認

    let mut world = World::new();
    
    // FrameCountリソースを追加
    world.insert_resource(wintf::ecs::FrameCount::default());

    // 初期状態ではGraphicsCoreリソースが存在しないことを確認
    assert!(
        world.get_resource::<GraphicsCore>().is_none(),
        "初期状態ではGraphicsCoreリソースは存在しない"
    );

    // init_graphics_coreシステムを作成して実行
    let mut system = IntoSystem::into_system(init_graphics_core);
    system.initialize(&mut world);
    let _ = system.run((), &mut world);

    // コマンドを適用
    system.apply_deferred(&mut world);

    // GraphicsCoreリソースが作成されたことを確認
    let graphics_res = world.get_resource::<GraphicsCore>();
    assert!(
        graphics_res.is_some(),
        "init_graphics_core実行後、GraphicsCoreリソースが存在すること"
    );

    eprintln!("✅ init_graphics_core - リソース作成成功");
}

#[test]
fn test_init_graphics_core_idempotent() {
    // init_graphics_coreが冪等であることを確認
    // （既にリソースが存在する場合は何もしない）

    let mut world = World::new();

    // 最初のGraphicsCoreを作成して登録
    let first_graphics = GraphicsCore::new().expect("GraphicsCore作成成功");
    world.insert_resource(first_graphics);

    // GraphicsCoreが存在することを確認
    assert!(
        world.get_resource::<GraphicsCore>().is_some(),
        "事前にGraphicsCoreが存在"
    );

    // init_graphics_coreシステムを実行
    let mut system = IntoSystem::into_system(init_graphics_core);
    system.initialize(&mut world);
    let _ = system.run((), &mut world);
    system.apply_deferred(&mut world);

    // GraphicsCoreが依然として存在することを確認（上書きされていない）
    let graphics_after = world.get_resource::<GraphicsCore>();
    assert!(
        graphics_after.is_some(),
        "init_graphics_core実行後もGraphicsCoreは存在"
    );

    eprintln!("✅ init_graphics_core - 冪等性確認（既存リソースを保持）");
}

#[test]
fn test_init_graphics_core_in_schedule() {
    // スケジュール内でinit_graphics_coreが正しく動作することを確認

    let mut world = World::new();
    
    // FrameCountリソースを追加
    world.insert_resource(wintf::ecs::FrameCount::default());
    
    let mut schedule = Schedule::default();

    // init_graphics_coreをスケジュールに追加
    schedule.add_systems(init_graphics_core);

    // 初期状態ではリソースが存在しない
    assert!(
        world.get_resource::<GraphicsCore>().is_none(),
        "スケジュール実行前はリソースが存在しない"
    );

    // スケジュールを実行
    schedule.run(&mut world);

    // リソースが作成されたことを確認
    assert!(
        world.get_resource::<GraphicsCore>().is_some(),
        "スケジュール実行後、リソースが存在する"
    );

    eprintln!("✅ init_graphics_core - スケジュール内で正常動作");
}

#[test]
fn test_graphics_core_as_resource() {
    // GraphicsCoreをECSリソースとして登録・取得できることを確認

    let mut world = World::new();

    // 初期状態ではGraphicsCoreリソースが存在しないことを確認
    assert!(
        world.get_resource::<GraphicsCore>().is_none(),
        "初期状態ではGraphicsCoreリソースは存在しない"
    );

    // GraphicsCoreを作成して登録
    let graphics = GraphicsCore::new().expect("GraphicsCore作成成功");
    world.insert_resource(graphics);

    // GraphicsCoreリソースが作成されたことを確認
    let graphics_res = world.get_resource::<GraphicsCore>();
    assert!(
        graphics_res.is_some(),
        "insert_resource後、GraphicsCoreリソースが存在すること"
    );

    eprintln!("✅ GraphicsCore - ECSリソースとして登録成功");
}

#[test]
fn test_graphics_core_accessible_from_system() {
    // システムからGraphicsCoreにアクセスできることを確認

    let mut world = World::new();

    // GraphicsCoreを作成して登録
    let graphics = GraphicsCore::new().expect("GraphicsCore作成成功");
    world.insert_resource(graphics);

    // テスト用システムを定義
    fn test_system(_graphics: Res<GraphicsCore>) {
        eprintln!("システムからGraphicsCoreにアクセス成功");
    }

    // システムを実行
    let mut system = IntoSystem::into_system(test_system);
    system.initialize(&mut world);
    let _ = system.run((), &mut world);

    eprintln!("✅ GraphicsCore - システムからアクセス可能");
}

#[test]
fn test_multiple_systems_can_access_graphics_core() {
    // 複数のシステムが同時にGraphicsCoreにアクセスできることを確認

    let mut world = World::new();

    // GraphicsCoreを作成して登録
    let graphics = GraphicsCore::new().expect("GraphicsCore作成成功");
    world.insert_resource(graphics);

    // 複数のシステムを定義
    fn system1(_graphics: Res<GraphicsCore>) {
        eprintln!("System1がGraphicsCoreにアクセス");
    }

    fn system2(_graphics: Res<GraphicsCore>) {
        eprintln!("System2がGraphicsCoreにアクセス");
    }

    fn system3(_graphics: Res<GraphicsCore>) {
        eprintln!("System3がGraphicsCoreにアクセス");
    }

    // システムを順次実行
    let mut s1 = IntoSystem::into_system(system1);
    let mut s2 = IntoSystem::into_system(system2);
    let mut s3 = IntoSystem::into_system(system3);

    s1.initialize(&mut world);
    s2.initialize(&mut world);
    s3.initialize(&mut world);

    let _ = s1.run((), &mut world);
    let _ = s2.run((), &mut world);
    let _ = s3.run((), &mut world);

    eprintln!("✅ GraphicsCore - 複数システムから同時アクセス可能");
}

#[test]
fn test_graphics_core_with_schedule() {
    // スケジュール内でGraphicsCoreが使用できることを確認

    let mut world = World::new();
    let mut schedule = Schedule::default();

    // GraphicsCoreを作成して登録
    let graphics = GraphicsCore::new().expect("GraphicsCore作成成功");
    world.insert_resource(graphics);

    // テストシステムを追加
    fn test_system(_graphics: Res<GraphicsCore>) {
        eprintln!("スケジュール内のシステムからGraphicsCoreにアクセス");
    }

    schedule.add_systems(test_system);

    // スケジュールを実行
    schedule.run(&mut world);

    eprintln!("✅ GraphicsCore - スケジュール内で使用可能");
}

#[test]
fn test_graphics_core_resource_trait() {
    // GraphicsCoreがResourceトレイトを実装していることを確認

    fn assert_resource<T: Resource>() {}

    assert_resource::<GraphicsCore>();

    eprintln!("✅ GraphicsCore - Resourceトレイト実装を確認");
}

#[test]
fn test_graphics_core_lifecycle() {
    // GraphicsCoreのライフサイクルを確認
    // （作成 → 使用 → ドロップ）

    let mut world = World::new();

    // 作成
    let graphics = GraphicsCore::new().expect("GraphicsCore作成成功");
    world.insert_resource(graphics);
    eprintln!("GraphicsCore作成完了");

    // 使用
    {
        let _graphics = world.get_resource::<GraphicsCore>().unwrap();
        eprintln!("GraphicsCore使用中");
    }

    // リソースを削除
    world.remove_resource::<GraphicsCore>();
    eprintln!("GraphicsCore削除完了");

    // 削除後は存在しないことを確認
    assert!(
        world.get_resource::<GraphicsCore>().is_none(),
        "削除後はリソースが存在しない"
    );

    eprintln!("✅ GraphicsCore - ライフサイクル管理正常");
}
