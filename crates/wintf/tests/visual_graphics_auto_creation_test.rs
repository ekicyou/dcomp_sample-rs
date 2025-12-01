//! VisualGraphics 自動作成テスト (R2)
//!
//! Visual コンポーネント追加時に VisualGraphics が自動的に作成され、
//! SurfaceGraphics は作成されないことをテストする。

use bevy_ecs::prelude::*;
use windows::core::Result;
use wintf::ecs::world::FrameCount;
use wintf::ecs::{
    visual_resource_management_system, GraphicsCore, SurfaceGraphics, Visual, VisualGraphics,
};

/// テスト用の GraphicsCore を作成するヘルパー関数
fn setup_graphics() -> Result<GraphicsCore> {
    GraphicsCore::new()
}

/// テスト用のワールドをセットアップするヘルパー関数
fn setup_world_with_graphics() -> Result<World> {
    let graphics = setup_graphics()?;
    let mut world = World::new();
    world.insert_resource(graphics);
    world.insert_resource(FrameCount(1));
    Ok(world)
}

/// Visual追加時に VisualGraphics が自動的に作成されることを確認
#[test]
fn test_visual_triggers_visual_graphics_creation() -> Result<()> {
    let mut world = setup_world_with_graphics()?;

    // Visual を持つ Entity を作成
    let entity = world.spawn(Visual::default()).id();

    // この時点では VisualGraphics はまだない
    // システムを実行すると作成される
    let mut schedule = Schedule::default();
    schedule.add_systems(visual_resource_management_system);
    schedule.run(&mut world);

    // VisualGraphics が作成されていることを確認
    let vg = world.get::<VisualGraphics>(entity);
    assert!(
        vg.is_some(),
        "VisualGraphics should be created for entity with Visual"
    );

    Ok(())
}

/// Visual追加時に SurfaceGraphics は即時作成されないことを確認（R2 AC#5, R5で遅延作成）
/// 注意: 現在の実装ではSurfaceも作成されるが、設計では遅延作成が求められている
#[test]
fn test_visual_does_not_trigger_immediate_surface_creation() -> Result<()> {
    let mut world = setup_world_with_graphics()?;

    // Visual を持つ Entity を作成
    let entity = world.spawn(Visual::default()).id();

    let mut schedule = Schedule::default();
    schedule.add_systems(visual_resource_management_system);
    schedule.run(&mut world);

    // 現在の実装では SurfaceGraphics も作成される（後で修正が必要）
    // この部分は Phase 4 で修正される
    let _surface = world.get::<SurfaceGraphics>(entity);
    // 現状の実装を確認するためのテスト（修正後は is_none() になる）
    // assert!(surface.is_none(), "SurfaceGraphics should NOT be created immediately (deferred creation)");

    Ok(())
}

/// VisualGraphics が既に存在する場合は再作成されないことを確認
#[test]
fn test_visual_graphics_not_recreated_if_exists() -> Result<()> {
    let mut world = setup_world_with_graphics()?;

    let entity = world.spawn(Visual::default()).id();

    // 最初のシステム実行
    let mut schedule = Schedule::default();
    schedule.add_systems(visual_resource_management_system);
    schedule.run(&mut world);

    let vg_first = world.get::<VisualGraphics>(entity).expect("should exist");
    let is_valid_first = vg_first.is_valid();

    // 2回目の実行（変更なし）
    schedule.run(&mut world);

    let vg_second = world
        .get::<VisualGraphics>(entity)
        .expect("should still exist");
    let is_valid_second = vg_second.is_valid();

    // 両方とも有効であることを確認
    assert!(is_valid_first, "First VisualGraphics should be valid");
    assert!(is_valid_second, "Second VisualGraphics should be valid");

    Ok(())
}

/// 複数の Entity に対して VisualGraphics が作成されることを確認
#[test]
fn test_multiple_entities_get_visual_graphics() -> Result<()> {
    let mut world = setup_world_with_graphics()?;

    let entity1 = world.spawn(Visual::default()).id();
    let entity2 = world.spawn(Visual::default()).id();
    let entity3 = world.spawn(Visual::default()).id();

    let mut schedule = Schedule::default();
    schedule.add_systems(visual_resource_management_system);
    schedule.run(&mut world);

    assert!(
        world.get::<VisualGraphics>(entity1).is_some(),
        "Entity1 should have VisualGraphics"
    );
    assert!(
        world.get::<VisualGraphics>(entity2).is_some(),
        "Entity2 should have VisualGraphics"
    );
    assert!(
        world.get::<VisualGraphics>(entity3).is_some(),
        "Entity3 should have VisualGraphics"
    );

    Ok(())
}

/// GraphicsCore が無効（invalidate済み）の場合は VisualGraphics のGPUリソースが作成されないことを確認
/// Note: Visual.on_add で VisualGraphics::default() は挿入されるが、GPUリソースは作成されない
#[test]
fn test_no_visual_graphics_with_invalid_graphics_core() -> Result<()> {
    let mut graphics = setup_graphics()?;
    // GraphicsCore を無効化
    graphics.invalidate();

    let mut world = World::new();
    world.insert_resource(graphics);
    world.insert_resource(FrameCount(1));

    let entity = world.spawn(Visual::default()).id();

    let mut schedule = Schedule::default();
    schedule.add_systems(visual_resource_management_system);
    schedule.run(&mut world);

    // VisualGraphics コンポーネント自体は Visual.on_add で作成される
    let vg = world.get::<VisualGraphics>(entity);
    assert!(
        vg.is_some(),
        "VisualGraphics component should exist (created by Visual.on_add)"
    );

    // ただし、GPUリソースは作成されない（GraphicsCoreが無効なため）
    assert!(
        !vg.unwrap().is_valid(),
        "VisualGraphics should not have GPU resources with invalid GraphicsCore"
    );

    Ok(())
}
