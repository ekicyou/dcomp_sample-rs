use bevy_ecs::prelude::*;
use wintf::ecs::mark_dirty_surfaces;
use wintf::ecs::GraphicsCommandList;
use wintf::ecs::SurfaceGraphics;
use wintf::ecs::SurfaceUpdateRequested;

#[test]
fn test_surface_update_requested_component_exists() {
    let mut world = World::new();
    let entity = world.spawn(SurfaceUpdateRequested).id();
    assert!(world.entity(entity).contains::<SurfaceUpdateRequested>());
}

#[test]
fn test_mark_dirty_surfaces_propagation() {
    let mut app = bevy_app::App::new();
    app.add_systems(bevy_app::Update, mark_dirty_surfaces);

    // Setup hierarchy: Surface -> Child
    let surface_entity = app.world_mut().spawn(SurfaceGraphics::default()).id();

    let child_entity = app.world_mut().spawn(GraphicsCommandList::empty()).id();

    app.world_mut()
        .entity_mut(surface_entity)
        .add_child(child_entity);

    // Run once to clear initial change trackers
    app.update();

    // Mutate child
    if let Some(mut cmd_list) = app
        .world_mut()
        .entity_mut(child_entity)
        .get_mut::<GraphicsCommandList>()
    {
        // Force change detection
        cmd_list.set_changed();
    }

    app.update();

    assert!(app
        .world()
        .entity(surface_entity)
        .contains::<SurfaceUpdateRequested>());
}

#[test]
fn test_surface_update_requested_on_add_hook() {
    // Appを使ってテストするのが確実。
    let mut app = bevy_app::App::new();
    let entity = app.world_mut().spawn(SurfaceGraphics::default()).id();

    // コマンドを適用させるために update を呼ぶか、
    // 単に spawn しただけではフック内のコマンドはまだ適用されていない可能性がある。

    // 1回 update を回す
    app.update();

    assert!(app
        .world()
        .entity(entity)
        .contains::<SurfaceUpdateRequested>());
}
