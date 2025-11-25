use crate::com::dcomp::DCompositionDeviceExt;
use crate::ecs::graphics::{
    GraphicsCore, GraphicsNeedsInit, SurfaceGraphics, Visual, VisualGraphics, WindowGraphics,
};
use bevy_ecs::prelude::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows::Win32::Graphics::Dxgi::Common::*;

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

fn create_visual_resources(
    commands: &mut Commands,
    entity: Entity,
    visual: &Visual,
    dcomp: &IDCompositionDevice3,
) {
    // 1. Create Visual
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

            commands
                .entity(entity)
                .insert(VisualGraphics::new(v3.clone()));

            // 2. Create Surface (Requirement R2: Always create surface)
            // Use size from Visual component
            let width = visual.size.X as u32;
            let height = visual.size.Y as u32;

            // Ensure non-zero size
            let width = if width == 0 { 1 } else { width };
            let height = if height == 0 { 1 } else { height };

            eprintln!(
                "[visual_creation_system] Creating Surface for Entity={:?}, size={}x{} (from Visual.size=({}, {}))",
                entity, width, height, visual.size.X, visual.size.Y
            );

            let surface_res = dcomp.create_surface(
                width,
                height,
                DXGI_FORMAT_B8G8R8A8_UNORM,
                DXGI_ALPHA_MODE_PREMULTIPLIED,
            );

            match surface_res {
                Ok(surface) => {
                    // Set content
                    unsafe {
                        let _ = v3.SetContent(&surface);
                    }
                    commands
                        .entity(entity)
                        .insert(SurfaceGraphics::new(surface, (width, height)));
                    eprintln!(
                        "[visual_creation_system] Surface created successfully for Entity={:?}",
                        entity
                    );
                }
                Err(e) => {
                    eprintln!("Failed to create surface for Entity={:?}: {:?}", entity, e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to create visual: {:?}", e);
        }
    }
}

/// Visualコンポーネントに基づいてGPUリソースを管理するシステム
pub fn visual_resource_management_system(
    mut commands: Commands,
    graphics: Res<GraphicsCore>,
    query: Query<(Entity, &Visual), Added<Visual>>,
) {
    if !graphics.is_valid() {
        return;
    }

    let dcomp = match graphics.dcomp() {
        Some(d) => d,
        None => return,
    };

    for (entity, visual) in query.iter() {
        create_visual_resources(&mut commands, entity, visual, dcomp);
    }
}

/// Visualリソースの再初期化システム
pub fn visual_reinit_system(
    mut commands: Commands,
    graphics: Res<GraphicsCore>,
    query: Query<(Entity, &Visual), With<GraphicsNeedsInit>>,
) {
    if !graphics.is_valid() {
        return;
    }

    let dcomp = match graphics.dcomp() {
        Some(d) => d,
        None => return,
    };

    for (entity, visual) in query.iter() {
        create_visual_resources(&mut commands, entity, visual, dcomp);
    }
}

/// WindowGraphicsとVisualGraphicsを紐付けるシステム
///
/// WindowGraphicsを持つエンティティにVisualGraphicsが追加された場合、
/// そのVisualをウィンドウのルートVisualとして設定する。
pub fn window_visual_integration_system(
    query: Query<
        (Entity, &WindowGraphics, &VisualGraphics),
        Or<(Changed<WindowGraphics>, Changed<VisualGraphics>)>,
    >,
) {
    for (_entity, window_graphics, visual_graphics) in query.iter() {
        if let Some(target) = window_graphics.get_target() {
            if let Some(visual) = visual_graphics.visual() {
                unsafe {
                    let _ = target.SetRoot(visual);
                }
            }
        }
    }
}
