use crate::com::d2d::*;
use crate::com::dcomp::*;
use crate::ecs::graphics::{GraphicsCore, GraphicsNeedsInit, HasGraphicsResources, Surface, Visual, WindowGraphics};
use bevy_ecs::prelude::*;
use windows::core::Result;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows::Win32::Graphics::Dxgi::Common::*;

// ========== ヘルパー関数 ==========

/// HWNDに対してWindowGraphicsリソースを作成する
fn create_window_graphics_for_hwnd(graphics: &GraphicsCore, hwnd: HWND) -> Result<WindowGraphics> {
    use windows::Win32::Graphics::Direct2D::D2D1_DEVICE_CONTEXT_OPTIONS_NONE;

    if !graphics.is_valid() {
        return Err(windows::core::Error::from(E_FAIL));
    }

    // 1. CompositionTarget作成
    let desktop = graphics.desktop().ok_or(windows::core::Error::from(E_FAIL))?;
    let target = desktop.create_target_for_hwnd(hwnd, true)?;

    // 2. DeviceContext作成
    let d2d = graphics.d2d_device().ok_or(windows::core::Error::from(E_FAIL))?;
    let device_context = d2d.create_device_context(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;

    Ok(WindowGraphics::new(target, device_context))
}

/// IDCompositionTargetに対してVisualを作成してルートに設定する
fn create_visual_for_target(
    graphics: &GraphicsCore,
    target: &IDCompositionTarget,
) -> Result<Visual> {
    if !graphics.is_valid() {
        return Err(windows::core::Error::from(E_FAIL));
    }

    // 1. ビジュアル作成
    let dcomp = graphics.dcomp().ok_or(windows::core::Error::from(E_FAIL))?;
    let visual = dcomp.create_visual()?;

    // 2. ターゲットにルートとして設定
    target.set_root(&visual)?;

    Ok(Visual::new(visual))
}

/// Surfaceを作成してVisualに設定する
fn create_surface_for_window(
    graphics: &GraphicsCore,
    visual: &Visual,
    width: u32,
    height: u32,
) -> Result<Surface> {
    if !graphics.is_valid() {
        return Err(windows::core::Error::from(E_FAIL));
    }

    // 1. IDCompositionSurface作成
    let dcomp = graphics.dcomp().ok_or(windows::core::Error::from(E_FAIL))?;
    let surface = dcomp.create_surface(
        width,
        height,
        DXGI_FORMAT_B8G8R8A8_UNORM,
        DXGI_ALPHA_MODE_PREMULTIPLIED,
    )?;

    // 2. VisualにSurfaceを設定
    let visual_ref = visual.visual().ok_or(windows::core::Error::from(E_FAIL))?;
    visual_ref.set_content(&surface)?;

    Ok(Surface::new(surface))
}

// ========== 描画システム ==========/// Surfaceへの描画（GraphicsCommandListの有無を統合処理）
pub fn render_surface(
    query: Query<
        (
            Entity,
            Option<&crate::ecs::graphics::GraphicsCommandList>,
            &Surface,
        ),
        Or<(
            Changed<crate::ecs::graphics::GraphicsCommandList>,
            Changed<Surface>,
        )>,
    >,
    _graphics_core: Option<Res<GraphicsCore>>,
) {
    use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

    for (entity, command_list, surface) in query.iter() {
        if !surface.is_valid() {
            continue;
        }

        let command_list = match command_list {
            Some(a) => a.command_list(),
            None => None,
        };
        eprintln!(
            "[render_surface] Entity={:?}, has_command_list={}",
            entity,
            command_list.is_some()
        );

        // Surface描画開始
        let surface_ref = match surface.surface() {
            Some(s) => s,
            None => continue,
        };
        let (dc, _offset) = match surface_ref.begin_draw(None) {
            Ok(result) => result,
            Err(err) => {
                eprintln!(
                    "[render_surface] Failed to begin draw for Entity={:?}: {:?}",
                    entity, err
                );
                continue;
            }
        };

        unsafe {
            // 透明色クリア（常に実行）
            dc.clear(Some(&D2D1_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }));

            // CommandListがある場合のみ描画
            if let Some(command_list) = command_list {
                dc.draw_image(command_list);
            }

            if let Err(err) = dc.EndDraw(None, None) {
                eprintln!(
                    "[render_surface] EndDraw failed for Entity={:?}: {:?}",
                    entity, err
                );
                let _ = surface_ref.end_draw();
                continue;
            }
        }

        // Surface描画終了
        if let Err(err) = surface_ref.end_draw() {
            eprintln!(
                "[render_surface] Failed to end draw for Entity={:?}: {:?}",
                entity, err
            );
            continue;
        }
    }
}

/// DirectCompositionのすべての変更を確定する
pub fn commit_composition(graphics: Option<Res<GraphicsCore>>) {
    let Some(graphics) = graphics else {
        return;
    };

    if !graphics.is_valid() {
        return;
    }

    let dcomp = match graphics.dcomp() {
        Some(d) => d,
        None => return,
    };

    if let Err(e) = dcomp.commit() {
        eprintln!("[commit_composition] Commit失敗: HRESULT {:?}", e);
    }
}

// ========== 新しい再初期化システム ==========

/// GraphicsCore初期化・再初期化・一括マーキング
pub fn init_graphics_core(
    graphics: Option<ResMut<GraphicsCore>>,
    query: Query<Entity, With<HasGraphicsResources>>,
    mut commands: Commands,
) {
    match graphics {
        Some(mut gc) => {
            if gc.is_valid() {
                return;
            }
            
            eprintln!("[init_graphics_core] GraphicsCore再初期化を開始");
            match GraphicsCore::new() {
                Ok(new_gc) => {
                    *gc = new_gc;
                    eprintln!("[init_graphics_core] GraphicsCore再初期化完了");
                    
                    let count = query.iter().count();
                    eprintln!("[init_graphics_core] {}個のエンティティにGraphicsNeedsInitマーカーを追加", count);
                    for entity in query.iter() {
                        commands.entity(entity).insert(GraphicsNeedsInit);
                    }
                }
                Err(e) => {
                    eprintln!("[init_graphics_core] GraphicsCore再初期化失敗: {:?}", e);
                }
            }
        }
        None => {
            eprintln!("[init_graphics_core] GraphicsCore初期化を開始");
            match GraphicsCore::new() {
                Ok(gc) => {
                    eprintln!("[init_graphics_core] GraphicsCore初期化完了");
                    commands.insert_resource(gc);
                    
                    let count = query.iter().count();
                    eprintln!("[init_graphics_core] {}個のエンティティにGraphicsNeedsInitマーカーを追加", count);
                    for entity in query.iter() {
                        commands.entity(entity).insert(GraphicsNeedsInit);
                    }
                }
                Err(e) => {
                    eprintln!("[init_graphics_core] GraphicsCore初期化失敗: {:?}", e);
                }
            }
        }
    }
}

/// WindowGraphics初期化・再初期化
pub fn init_window_graphics(
    graphics: Res<GraphicsCore>,
    mut query: Query<
        (Entity, &crate::ecs::window::WindowHandle, Option<&mut WindowGraphics>),
        Or<(
            Without<WindowGraphics>,
            With<GraphicsNeedsInit>,
        )>
    >,
    mut commands: Commands,
) {
    if !graphics.is_valid() {
        return;
    }

    for (entity, handle, window_graphics) in query.iter_mut() {
        match window_graphics {
            None => {
                eprintln!("[init_window_graphics] WindowGraphics新規作成 (Entity: {:?})", entity);
                match create_window_graphics_for_hwnd(&graphics, handle.hwnd) {
                    Ok(wg) => {
                        eprintln!("[init_window_graphics] WindowGraphics作成完了 (Entity: {:?})", entity);
                        commands.entity(entity).insert(wg);
                    }
                    Err(e) => {
                        eprintln!("[init_window_graphics] エラー: Entity {:?}, HRESULT {:?}", entity, e);
                    }
                }
            }
            Some(mut wg) => {
                if !wg.is_valid() {
                    eprintln!("[init_window_graphics] WindowGraphics再初期化 (Entity: {:?})", entity);
                    let old_generation = wg.generation();
                    match create_window_graphics_for_hwnd(&graphics, handle.hwnd) {
                        Ok(new_wg) => {
                            // 古いgenerationを引き継いでインクリメント
                            let new_generation = old_generation.wrapping_add(1);
                            *wg = new_wg;
                            // generation を手動で設定（newのデフォルトは0なので）
                            while wg.generation() < new_generation {
                                wg.increment_generation();
                            }
                            eprintln!("[init_window_graphics] WindowGraphics再初期化完了 (Entity: {:?}, generation: {} -> {})", 
                                entity, old_generation, wg.generation());
                        }
                        Err(e) => {
                            eprintln!("[init_window_graphics] 再初期化エラー: Entity {:?}, HRESULT {:?}", entity, e);
                        }
                    }
                }
            }
        }
    }
}

/// Visual初期化・再初期化
pub fn init_window_visual(
    graphics: Res<GraphicsCore>,
    mut query: Query<
        (Entity, &WindowGraphics, Option<&mut Visual>),
        Or<(
            Without<Visual>,
            With<GraphicsNeedsInit>,
        )>
    >,
    mut commands: Commands,
) {
    if !graphics.is_valid() {
        return;
    }

    for (entity, window_graphics, visual) in query.iter_mut() {
        if !window_graphics.is_valid() {
            continue;
        }

        let target = match window_graphics.target() {
            Some(t) => t,
            None => continue,
        };

        match visual {
            None => {
                eprintln!("[init_window_visual] Visual新規作成 (Entity: {:?})", entity);
                match create_visual_for_target(&graphics, target) {
                    Ok(v) => {
                        eprintln!("[init_window_visual] Visual作成完了 (Entity: {:?})", entity);
                        commands.entity(entity).insert(v);
                    }
                    Err(e) => {
                        eprintln!("[init_window_visual] エラー: Entity {:?}, HRESULT {:?}", entity, e);
                    }
                }
            }
            Some(mut v) => {
                if !v.is_valid() {
                    eprintln!("[init_window_visual] Visual再初期化 (Entity: {:?})", entity);
                    match create_visual_for_target(&graphics, target) {
                        Ok(new_v) => {
                            *v = new_v;
                            eprintln!("[init_window_visual] Visual再初期化完了 (Entity: {:?})", entity);
                        }
                        Err(e) => {
                            eprintln!("[init_window_visual] 再初期化エラー: Entity {:?}, HRESULT {:?}", entity, e);
                        }
                    }
                }
            }
        }
    }
}

/// Surface初期化・再初期化
pub fn init_window_surface(
    graphics: Res<GraphicsCore>,
    mut query: Query<
        (Entity, &WindowGraphics, &Visual, Option<&mut Surface>, Option<&crate::ecs::window::WindowPos>),
        Or<(
            Without<Surface>,
            With<GraphicsNeedsInit>,
        )>
    >,
    mut commands: Commands,
) {
    if !graphics.is_valid() {
        return;
    }

    for (entity, window_graphics, visual, surface, window_pos) in query.iter_mut() {
        if !window_graphics.is_valid() || !visual.is_valid() {
            continue;
        }

        let (width, height) = window_pos
            .and_then(|pos| pos.size.map(|s| (s.cx as u32, s.cy as u32)))
            .unwrap_or((800, 600));

        match surface {
            None => {
                eprintln!("[init_window_surface] Surface新規作成 (Entity: {:?}, Size: {}x{})", entity, width, height);
                match create_surface_for_window(&graphics, visual, width, height) {
                    Ok(s) => {
                        eprintln!("[init_window_surface] Surface作成完了 (Entity: {:?})", entity);
                        commands.entity(entity).insert(s);
                    }
                    Err(e) => {
                        eprintln!("[init_window_surface] エラー: Entity {:?}, HRESULT {:?}", entity, e);
                    }
                }
            }
            Some(mut s) => {
                if !s.is_valid() {
                    eprintln!("[init_window_surface] Surface再初期化 (Entity: {:?}, Size: {}x{})", entity, width, height);
                    match create_surface_for_window(&graphics, visual, width, height) {
                        Ok(new_s) => {
                            *s = new_s;
                            eprintln!("[init_window_surface] Surface再初期化完了 (Entity: {:?})", entity);
                        }
                        Err(e) => {
                            eprintln!("[init_window_surface] 再初期化エラー: Entity {:?}, HRESULT {:?}", entity, e);
                        }
                    }
                }
            }
        }
    }
}

/// GraphicsNeedsInitマーカー削除・初期化完了判定
pub fn cleanup_graphics_needs_init(
    query: Query<
        (Entity, &WindowGraphics, &Visual, &Surface),
        With<GraphicsNeedsInit>
    >,
    mut commands: Commands,
) {
    for (entity, window_graphics, visual, surface) in query.iter() {
        if window_graphics.is_valid() && visual.is_valid() && surface.is_valid() {
            eprintln!("[cleanup_graphics_needs_init] GraphicsNeedsInitマーカー削除 (Entity: {:?})", entity);
            commands.entity(entity).remove::<GraphicsNeedsInit>();
        }
    }
}

/// GraphicsNeedsInit時に古いGraphicsCommandListを削除
pub fn cleanup_command_list_on_reinit(
    query: Query<Entity, (With<GraphicsNeedsInit>, With<crate::ecs::graphics::GraphicsCommandList>)>,
    mut commands: Commands,
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<crate::ecs::graphics::GraphicsCommandList>();
        eprintln!("[cleanup_command_list_on_reinit] GraphicsCommandList削除 (Entity: {:?})", entity);
    }
}

/// 依存コンポーネント無効化
pub fn invalidate_dependent_components(
    graphics: Option<Res<GraphicsCore>>,
    mut window_graphics_query: Query<&mut WindowGraphics>,
    mut visual_query: Query<&mut Visual>,
    mut surface_query: Query<&mut Surface>,
) {
    if let Some(gc) = graphics {
        if !gc.is_valid() {
            eprintln!("[invalidate_dependent_components] GraphicsCore無効 - 全依存コンポーネントを無効化");
            
            for mut wg in window_graphics_query.iter_mut() {
                wg.invalidate();
            }
            for mut v in visual_query.iter_mut() {
                v.invalidate();
            }
            for mut s in surface_query.iter_mut() {
                s.invalidate();
            }
        }
    }
}
