use bevy_ecs::prelude::*;
use wintf::ecs::calculate_surface_size_from_global_arrangement;
use wintf::ecs::layout::GlobalArrangement;
use wintf::ecs::mark_dirty_surfaces;
use wintf::ecs::world::FrameCount;
use wintf::ecs::GraphicsCommandList;
use wintf::ecs::SurfaceCreationStats;
use wintf::ecs::SurfaceGraphics;
use wintf::ecs::SurfaceGraphicsDirty;

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
    let entity = app
        .world_mut()
        .spawn((
            SurfaceGraphics::default(),
            SurfaceGraphicsDirty::default(),
            GraphicsCommandList::empty(),
        ))
        .id();

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
    let dirty = app
        .world()
        .entity(entity)
        .get::<SurfaceGraphicsDirty>()
        .unwrap();
    assert_eq!(dirty.requested_frame, 42);
}

#[test]
fn test_surface_graphics_dirty_on_surface_added() {
    // SurfaceGraphicsの追加時にSurfaceGraphicsDirtyも一緒に追加されることをテスト
    // （実際にはdeferred_surface_creation_systemが担当）
    let mut app = bevy_app::App::new();
    app.insert_resource(FrameCount(0));

    // SurfaceGraphicsとSurfaceGraphicsDirtyを一緒にspawn
    let entity = app
        .world_mut()
        .spawn((SurfaceGraphics::default(), SurfaceGraphicsDirty::default()))
        .id();

    app.update();

    // 両方のコンポーネントが存在することを確認
    assert!(app.world().entity(entity).contains::<SurfaceGraphics>());
    assert!(app
        .world()
        .entity(entity)
        .contains::<SurfaceGraphicsDirty>());
}

// ========== Task 1.1: SurfaceCreationStats テスト ==========

#[test]
fn test_surface_creation_stats_default() {
    // Req 5.3: デバッグビルド時のSurface生成統計
    let stats = SurfaceCreationStats::default();
    assert_eq!(stats.created_count, 0);
    assert_eq!(stats.skipped_count, 0);
    assert_eq!(stats.deleted_count, 0);
    assert_eq!(stats.resize_count, 0);
}

#[test]
fn test_surface_creation_stats_record_created() {
    let mut stats = SurfaceCreationStats::default();
    stats.record_created();
    assert_eq!(stats.created_count, 1);
    stats.record_created();
    assert_eq!(stats.created_count, 2);
}

#[test]
fn test_surface_creation_stats_record_skipped() {
    let mut stats = SurfaceCreationStats::default();
    stats.record_skipped();
    assert_eq!(stats.skipped_count, 1);
}

#[test]
fn test_surface_creation_stats_record_deleted() {
    let mut stats = SurfaceCreationStats::default();
    stats.record_deleted();
    assert_eq!(stats.deleted_count, 1);
}

#[test]
fn test_surface_creation_stats_record_resized() {
    let mut stats = SurfaceCreationStats::default();
    stats.record_resized();
    assert_eq!(stats.resize_count, 1);
}

#[test]
fn test_surface_creation_stats_as_resource() {
    // SurfaceCreationStatsがECSリソースとして使えることを確認
    let mut world = World::new();
    world.insert_resource(SurfaceCreationStats::default());

    // リソースが取得できることを確認
    let stats = world.resource::<SurfaceCreationStats>();
    assert_eq!(stats.created_count, 0);

    // 可変参照で更新
    world
        .resource_mut::<SurfaceCreationStats>()
        .record_created();
    assert_eq!(world.resource::<SurfaceCreationStats>().created_count, 1);
}

// ========== Task 1.2: 物理ピクセルサイズ計算ヘルパーテスト ==========

#[test]
fn test_calculate_surface_size_from_global_arrangement_normal() {
    // Req 3.1, 3.2: GlobalArrangement.boundsから物理ピクセルサイズを計算
    use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;

    let ga = GlobalArrangement {
        bounds: D2D_RECT_F {
            left: 10.0,
            top: 20.0,
            right: 110.0,
            bottom: 80.0,
        },
        ..Default::default()
    };

    let result = calculate_surface_size_from_global_arrangement(&ga);
    assert_eq!(result, Some((100, 60))); // 110-10=100, 80-20=60
}

#[test]
fn test_calculate_surface_size_from_global_arrangement_fractional() {
    // Req 3.2: 小数点以下の切り上げ
    use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;

    let ga = GlobalArrangement {
        bounds: D2D_RECT_F {
            left: 0.0,
            top: 0.0,
            right: 100.5,
            bottom: 50.3,
        },
        ..Default::default()
    };

    let result = calculate_surface_size_from_global_arrangement(&ga);
    assert_eq!(result, Some((101, 51))); // ceil(100.5)=101, ceil(50.3)=51
}

#[test]
fn test_calculate_surface_size_from_global_arrangement_zero_width() {
    // Req 3.3: サイズ0の場合はNone
    use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;

    let ga = GlobalArrangement {
        bounds: D2D_RECT_F {
            left: 50.0,
            top: 0.0,
            right: 50.0, // width = 0
            bottom: 100.0,
        },
        ..Default::default()
    };

    let result = calculate_surface_size_from_global_arrangement(&ga);
    assert_eq!(result, None);
}

#[test]
fn test_calculate_surface_size_from_global_arrangement_zero_height() {
    // Req 3.3: サイズ0の場合はNone
    use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;

    let ga = GlobalArrangement {
        bounds: D2D_RECT_F {
            left: 0.0,
            top: 50.0,
            right: 100.0,
            bottom: 50.0, // height = 0
        },
        ..Default::default()
    };

    let result = calculate_surface_size_from_global_arrangement(&ga);
    assert_eq!(result, None);
}

#[test]
fn test_calculate_surface_size_from_global_arrangement_negative() {
    // 負の幅・高さの場合もNone
    use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;

    let ga = GlobalArrangement {
        bounds: D2D_RECT_F {
            left: 100.0,
            top: 100.0,
            right: 50.0,  // negative width
            bottom: 50.0, // negative height
        },
        ..Default::default()
    };

    let result = calculate_surface_size_from_global_arrangement(&ga);
    assert_eq!(result, None);
}
