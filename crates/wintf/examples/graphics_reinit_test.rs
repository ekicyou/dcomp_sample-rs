#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy_ecs::prelude::*;
use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use windows::core::Result;
use windows::Win32::Foundation::{POINT, SIZE};
use wintf::ecs::widget::shapes::{colors, Rectangle};
use wintf::ecs::Window;
use wintf::ecs::{GraphicsCore, GraphicsNeedsInit, HasGraphicsResources, Surface, Visual, WindowGraphics, WindowHandle, WindowPos};
use wintf::*;

/// GraphicsCore再初期化システムの統合テスト
/// 
/// このテストは以下を検証します:
/// - GraphicsCore初期化とコンポーネントの自動初期化
/// - GraphicsCore無効化による依存コンポーネントの自動無効化
/// - GraphicsCore再初期化と全依存コンポーネントの再初期化
/// - 複数ウィンドウでの同時再初期化

type WorldCommand = Box<dyn FnOnce(&mut World) + Send>;

fn main() -> Result<()> {
    println!("\n========== GraphicsCore Reinitialization Test ==========\n");
    
    human_panic::setup_panic!();

    let mgr = WinThreadMgr::new()?;
    let world = mgr.world();

    let (tx, rx) = channel::<WorldCommand>();
    let rx = Mutex::new(rx);

    // テストシナリオスレッド
    thread::spawn(move || {
        // 0秒: ウィンドウを2つ作成
        println!("[Timer] 0s: Creating two windows");
        let _ = tx.send(Box::new(|world: &mut World| {
            world.spawn((
                Window {
                    title: "Test Window 1 (Red)".to_string(),
                    ..Default::default()
                },
                WindowPos {
                    position: Some(POINT { x: 100, y: 100 }),
                    size: Some(SIZE { cx: 600, cy: 400 }),
                    ..Default::default()
                },
                Rectangle {
                    x: 50.0,
                    y: 50.0,
                    width: 200.0,
                    height: 150.0,
                    color: colors::RED,
                },
            ));

            world.spawn((
                Window {
                    title: "Test Window 2 (Blue)".to_string(),
                    ..Default::default()
                },
                WindowPos {
                    position: Some(POINT { x: 750, y: 100 }),
                    size: Some(SIZE { cx: 600, cy: 400 }),
                    ..Default::default()
                },
                Rectangle {
                    x: 50.0,
                    y: 50.0,
                    width: 200.0,
                    height: 150.0,
                    color: colors::BLUE,
                },
            ));

            println!("[Test] Two windows spawned");
        }));

        // 2秒: グラフィックスコンポーネントの初期状態を検証
        thread::sleep(Duration::from_secs(2));
        println!("\n[Timer] 2s: Verifying initial graphics state");
        let _ = tx.send(Box::new(|world: &mut World| {
            let mut query = world.query::<(Entity, &WindowHandle, &WindowGraphics, &Visual, &Surface, &HasGraphicsResources)>();
            let count = query.iter(world).count();
            
            println!("[Test Phase 1] Initial State Verification:");
            println!("  - Windows with all graphics components: {}", count);
            
            if count == 2 {
                println!("  ✅ [PASS] Both windows have all graphics components");
                
                // 各ウィンドウのgeneration番号を確認
                for (entity, handle, wg, _v, _s, _) in query.iter(world) {
                    let is_valid = wg.is_valid();
                    let generation = wg.generation();
                    println!("  - Entity {:?} (HWND {:?}): valid={}, generation={}", 
                        entity, handle.hwnd, is_valid, generation);
                }
            } else {
                println!("  ❌ [FAIL] Expected 2 windows, found {}", count);
            }
        }));

        // 4秒: GraphicsCoreを無効化（デバイスロストをシミュレート）
        thread::sleep(Duration::from_secs(2));
        println!("\n[Timer] 4s: Simulating device loss (invalidating GraphicsCore)");
        let _ = tx.send(Box::new(|world: &mut World| {
            if let Some(mut graphics) = world.get_resource_mut::<GraphicsCore>() {
                println!("[Test Phase 2] Device Loss Simulation:");
                println!("  - Calling GraphicsCore.invalidate()");
                graphics.invalidate();
                println!("  - GraphicsCore invalidated, is_valid={}", graphics.is_valid());
                println!("  ✅ [PASS] GraphicsCore invalidation succeeded");
            } else {
                println!("  ❌ [FAIL] GraphicsCore resource not found");
            }
        }));

        // 5秒: 依存コンポーネントが無効化されたことを確認
        thread::sleep(Duration::from_secs(1));
        println!("\n[Timer] 5s: Verifying dependent components invalidation");
        let _ = tx.send(Box::new(|world: &mut World| {
            let mut query = world.query::<(Entity, &WindowHandle, &WindowGraphics, &Visual, &Surface)>();
            
            println!("[Test Phase 3] Dependent Components Invalidation:");
            let mut all_invalid = true;
            
            for (entity, handle, wg, v, s) in query.iter(world) {
                let wg_valid = wg.is_valid();
                let v_valid = v.is_valid();
                let s_valid = s.is_valid();
                
                println!("  - Entity {:?} (HWND {:?}):", entity, handle.hwnd);
                println!("    WindowGraphics.is_valid() = {}", wg_valid);
                println!("    Visual.is_valid() = {}", v_valid);
                println!("    Surface.is_valid() = {}", s_valid);
                
                if wg_valid || v_valid || s_valid {
                    all_invalid = false;
                }
            }
            
            if all_invalid {
                println!("  ✅ [PASS] All dependent components invalidated");
            } else {
                println!("  ❌ [FAIL] Some components still valid");
            }
        }));

        // 6秒: GraphicsNeedsInitマーカーが追加されたことを確認
        thread::sleep(Duration::from_secs(1));
        println!("\n[Timer] 6s: Verifying GraphicsNeedsInit marker");
        let _ = tx.send(Box::new(|world: &mut World| {
            let graphics_valid = world.get_resource::<GraphicsCore>()
                .map(|g| g.is_valid())
                .unwrap_or(false);
            
            let mut query = world.query::<(Entity, &WindowHandle, &GraphicsNeedsInit)>();
            let marked_count = query.iter(world).count();
            
            println!("[Test Phase 4] Reinitialization Marker:");
            println!("  - GraphicsCore.is_valid() = {}", graphics_valid);
            println!("  - Entities with GraphicsNeedsInit: {}", marked_count);
            
            if graphics_valid && marked_count == 2 {
                println!("  ✅ [PASS] GraphicsCore reinitialized and all entities marked");
            } else if !graphics_valid {
                println!("  ⏳ [WAIT] GraphicsCore not yet reinitialized");
            } else {
                println!("  ❌ [FAIL] Expected 2 marked entities, found {}", marked_count);
            }
        }));

        // 8秒: 再初期化完了を確認
        thread::sleep(Duration::from_secs(2));
        println!("\n[Timer] 8s: Verifying reinitialization completion");
        let _ = tx.send(Box::new(|world: &mut World| {
            let graphics_valid = world.get_resource::<GraphicsCore>()
                .map(|g| g.is_valid())
                .unwrap_or(false);
            
            let mut all_query = world.query::<(Entity, &WindowHandle, &WindowGraphics, &Visual, &Surface)>();
            let mut marker_query = world.query::<(Entity, &GraphicsNeedsInit)>();
            
            let marker_count = marker_query.iter(world).count();
            
            println!("[Test Phase 5] Reinitialization Completion:");
            println!("  - GraphicsCore.is_valid() = {}", graphics_valid);
            println!("  - Entities still with GraphicsNeedsInit: {}", marker_count);
            
            let mut all_valid = true;
            let mut generation_incremented = true;
            
            for (entity, handle, wg, v, s) in all_query.iter(world) {
                let wg_valid = wg.is_valid();
                let v_valid = v.is_valid();
                let s_valid = s.is_valid();
                let generation = wg.generation();
                
                println!("  - Entity {:?} (HWND {:?}):", entity, handle.hwnd);
                println!("    WindowGraphics: valid={}, generation={}", wg_valid, generation);
                println!("    Visual.is_valid() = {}", v_valid);
                println!("    Surface.is_valid() = {}", s_valid);
                
                if !wg_valid || !v_valid || !s_valid {
                    all_valid = false;
                }
                
                if generation == 0 {
                    generation_incremented = false;
                }
            }
            
            if graphics_valid && all_valid && marker_count == 0 && generation_incremented {
                println!("\n  ✅✅✅ [TEST SUCCESS] Complete reinitialization verified!");
                println!("  - GraphicsCore reinitialized");
                println!("  - All components reinitialized");
                println!("  - All markers cleaned up");
                println!("  - Generation numbers incremented");
            } else {
                if !graphics_valid {
                    println!("  ❌ GraphicsCore not valid");
                }
                if !all_valid {
                    println!("  ❌ Some components not valid");
                }
                if marker_count > 0 {
                    println!("  ⏳ {} entities still being initialized", marker_count);
                }
                if !generation_incremented {
                    println!("  ❌ Generation numbers not incremented");
                }
            }
        }));

        // 12秒: 最終状態確認と終了
        thread::sleep(Duration::from_secs(4));
        println!("\n[Timer] 12s: Final state check and closing windows");
        let _ = tx.send(Box::new(|world: &mut World| {
            let mut query = world.query::<(Entity, &WindowHandle, &WindowGraphics)>();
            let entities: Vec<_> = query.iter(world).collect();
            
            println!("\n[Test Phase 6] Final State:");
            println!("  - Total windows: {}", entities.len());
            
            for (entity, handle, wg) in entities {
                println!("  - Entity {:?} (HWND {:?}): generation={}", 
                    entity, handle.hwnd, wg.generation());
            }
            
            // 全ウィンドウを閉じてテスト終了
            let mut despawn_query = world.query::<(Entity, &WindowHandle)>();
            let to_despawn: Vec<_> = despawn_query.iter(world).map(|(e, _)| e).collect();
            
            println!("\n  Closing all windows to end test...");
            for entity in to_despawn {
                world.despawn(entity);
            }
        }));
    });

    println!("[Test] Test scenario started");
    println!("\nTest Phases:");
    println!("  Phase 1 (2s):  Verify initial graphics state");
    println!("  Phase 2 (4s):  Simulate device loss (invalidate GraphicsCore)");
    println!("  Phase 3 (5s):  Verify dependent components invalidation");
    println!("  Phase 4 (6s):  Verify GraphicsNeedsInit marker");
    println!("  Phase 5 (8s):  Verify reinitialization completion");
    println!("  Phase 6 (12s): Final state check and exit");
    println!("\n========================================\n");

    // コマンド実行システム
    world
        .borrow_mut()
        .add_systems(wintf::ecs::world::Update, move |world: &mut World| {
            let Ok(rx_guard) = rx.lock() else {
                return;
            };

            for command in rx_guard.try_iter() {
                command(world);
            }
        });

    mgr.run()?;

    Ok(())
}
