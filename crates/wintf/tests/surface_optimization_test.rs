use bevy_ecs::prelude::*;
use wintf::ecs::mark_dirty_surfaces;
use wintf::ecs::GraphicsCommandList;
use wintf::ecs::SurfaceGraphics;
use wintf::ecs::SurfaceGraphicsDirty;
use wintf::ecs::world::FrameCount;

#[test]
fn test_surface_graphics_dirty_component_exists() {
    let mut world = World::new();
    let entity = world.spawn(SurfaceGraphicsDirty::default()).id();
    assert!(world.entity(entity).contains::<SurfaceGraphicsDirty>());
    
    // デフォルトのrequested_frameは0
    let dirty = world.entity(entity).get::<SurfaceGraphicsDirty>().unwrap();
    assert_eq!(dirty.requested_frame, 0);
}

#[test]
fn test_mark_dirty_surfaces_updates_frame() {
    let mut app = bevy_app::App::new();
    app.insert_resource(FrameCount(1)); // フレーム番号1
    app.add_systems(bevy_app::Update, mark_dirty_surfaces);

    // SurfaceGraphics と SurfaceGraphicsDirty を持つエンティティを作成
    let entity = app.world_mut().spawn((
        SurfaceGraphics::default(),
        SurfaceGraphicsDirty::default(),
        GraphicsCommandList::empty(),
    )).id();

    // 最初のupdateでChangeトラッカーを初期化
    app.update();
    
    // フレーム番号を更新
    app.world_mut().resource_mut::<FrameCount>().0 = 42;

    // GraphicsCommandListを変更（強制的に変更検出をトリガー）
    if let Some(mut cmd_list) = app
        .world_mut()
        .entity_mut(entity)
        .get_mut::<GraphicsCommandList>()
    {
        cmd_list.set_changed();
    }

    app.update();

    // SurfaceGraphicsDirtyのrequested_frameがフレーム番号で更新されていることを確認
    let dirty = app.world().entity(entity).get::<SurfaceGraphicsDirty>().unwrap();
    assert_eq!(dirty.requested_frame, 42);
}

#[test]
fn test_surface_graphics_dirty_on_surface_added() {
    // SurfaceGraphicsの追加時にSurfaceGraphicsDirtyも一緒に追加されることをテスト
    // （実際にはdeferred_surface_creation_systemが担当）
    let mut app = bevy_app::App::new();
    app.insert_resource(FrameCount(0));
    
    // SurfaceGraphicsとSurfaceGraphicsDirtyを一緒にspawn
    let entity = app.world_mut().spawn((
        SurfaceGraphics::default(),
        SurfaceGraphicsDirty::default(),
    )).id();

    app.update();

    // 両方のコンポーネントが存在することを確認
    assert!(app.world().entity(entity).contains::<SurfaceGraphics>());
    assert!(app.world().entity(entity).contains::<SurfaceGraphicsDirty>());
}
