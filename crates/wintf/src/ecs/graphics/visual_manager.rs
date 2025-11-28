use crate::com::dcomp::DCompositionDeviceExt;
use crate::ecs::graphics::{
    GraphicsCore, HasGraphicsResources, SurfaceGraphics, SurfaceGraphicsDirty, Visual,
    VisualGraphics, WindowGraphics,
};
use bevy_ecs::name::Name;
use bevy_ecs::prelude::*;
use windows::Win32::Graphics::DirectComposition::*;

use super::format_entity_name;

/// デフォルトのVisualコンポーネントをEntityに挿入する (R3)
///
/// ウィジェットの on_add フックから呼び出すことを想定したヘルパー関数。
/// Entity が存在しない場合は何もしない。
///
/// # Arguments
/// * `world` - ECS World への可変参照
/// * `entity` - Visual を挿入する対象の Entity
pub fn insert_visual(world: &mut World, entity: Entity) {
    insert_visual_with(world, entity, Visual::default());
}

/// カスタム Visual コンポーネントを Entity に挿入する (R3)
///
/// ウィジェットの on_add フックから呼び出すことを想定したヘルパー関数。
/// Entity が存在しない場合は何もしない。
///
/// # Arguments
/// * `world` - ECS World への可変参照
/// * `entity` - Visual を挿入する対象の Entity
/// * `visual` - 挿入する Visual コンポーネント
pub fn insert_visual_with(world: &mut World, entity: Entity, visual: Visual) {
    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.insert(visual);
    }
}

/// Visualリソースのみを作成する（Surfaceは作成しない）
///
/// Phase 6リファクタリング: Visual作成をPreLayoutに移動し、
/// Surface作成はDrawスケジュールでCommandList存在時に遅延実行する。
///
/// 同時に空のSurfaceGraphicsとSurfaceGraphicsDirtyを配置し、
/// 後で直接更新できるようにする（commands.insert問題の回避）。
fn create_visual_only(
    commands: &mut Commands,
    entity: Entity,
    _visual: &Visual,
    dcomp: &IDCompositionDevice3,
) {
    let visual_res = dcomp.create_visual();
    match visual_res {
        Ok(v3) => {
            // FIXME: SetOpacity and SetVisible not found on IDCompositionVisual in windows 0.62?
            // if let Ok(visual_base) = v3.cast::<IDCompositionVisual>() {
            //     unsafe {
            //         let _ = visual_base.SetOpacity(visual.opacity);
            //         let _ = visual_base.SetVisible(visual.is_visible.into());
            //     }
            // }

            // VisualGraphicsと同時に空のSurfaceGraphics/SurfaceGraphicsDirtyも配置
            // これにより後で直接更新でき、commands.insert()の遅延問題を回避
            commands.entity(entity).insert((
                VisualGraphics::new(v3.clone()),
                SurfaceGraphics::default(),
                SurfaceGraphicsDirty::default(),
            ));

            eprintln!(
                "[visual_creation_system] Visual created for Entity={:?} (Surface deferred, SurfaceGraphics pre-allocated)",
                entity
            );
        }
        Err(e) => {
            eprintln!("Failed to create visual: {:?}", e);
        }
    }
}

/// Visualコンポーネントに基づいてGPUリソースを管理するシステム
///
/// Phase 6: Visualのみを作成し、Surfaceは作成しない。
/// SurfaceはDrawスケジュールでCommandList存在時に遅延作成される。
pub fn visual_resource_management_system(
    mut commands: Commands,
    graphics: Res<GraphicsCore>,
    query: Query<(Entity, &Visual, Option<&Name>), Added<Visual>>,
    frame_count: Res<crate::ecs::world::FrameCount>,
) {
    if !graphics.is_valid() {
        return;
    }

    let dcomp = match graphics.dcomp() {
        Some(d) => d,
        None => return,
    };

    for (entity, visual, name) in query.iter() {
        let entity_name = format_entity_name(entity, name);
        eprintln!(
            "[Frame {}] [visual_resource_management] VisualGraphics作成開始 (Entity: {})",
            frame_count.0, entity_name
        );
        create_visual_only(&mut commands, entity, visual, dcomp);
        eprintln!(
            "[Frame {}] [visual_resource_management] VisualGraphics作成完了 (Entity: {})",
            frame_count.0, entity_name
        );
    }
}

/// Visualリソースの再初期化システム
///
/// Phase 6: Visualのみを作成し、Surfaceは作成しない。
/// Changed: GraphicsNeedsInitマーカーから、Changed<HasGraphicsResources> + needs_init()に移行
pub fn visual_reinit_system(
    mut commands: Commands,
    graphics: Res<GraphicsCore>,
    query: Query<(Entity, &Visual, &HasGraphicsResources), Changed<HasGraphicsResources>>,
) {
    if !graphics.is_valid() {
        return;
    }

    let dcomp = match graphics.dcomp() {
        Some(d) => d,
        None => return,
    };

    for (entity, visual, res) in query.iter() {
        if res.needs_init() {
            create_visual_only(&mut commands, entity, visual, dcomp);
        }
    }
}

/// WindowGraphicsとVisualGraphicsを紐付けるシステム
///
/// WindowGraphicsを持つエンティティにVisualGraphicsが追加された場合、
/// そのVisualをウィンドウのルートVisualとして設定する。
pub fn window_visual_integration_system(
    query: Query<
        (Entity, &WindowGraphics, &VisualGraphics, Option<&Name>),
        Or<(Changed<WindowGraphics>, Changed<VisualGraphics>)>,
    >,
    frame_count: Res<crate::ecs::world::FrameCount>,
) {
    for (entity, window_graphics, visual_graphics, name) in query.iter() {
        if let Some(target) = window_graphics.get_target() {
            if let Some(visual) = visual_graphics.visual() {
                let entity_name = format_entity_name(entity, name);
                eprintln!(
                    "[Frame {}] [window_visual_integration] SetRoot実行 (Entity: {})",
                    frame_count.0, entity_name
                );
                unsafe {
                    let _ = target.SetRoot(visual);
                }
            }
        }
    }
}
