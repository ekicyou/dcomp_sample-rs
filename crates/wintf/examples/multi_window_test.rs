#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::core::*;
use windows::Win32::Foundation::{HWND, POINT, SIZE};
use wintf::ecs::{Window, WindowHandle, WindowPos};
use wintf::*;

/// タスク7.2: 複数ウィンドウでのグラフィックス初期化テスト
fn main() -> Result<()> {
    println!("\n========== Multi-Window Graphics Test ==========\n");
    
    human_panic::setup_panic!();
    
    let mgr = WinThreadMgr::new()?;
    let world = mgr.world();
    
    // 3つのウィンドウを作成
    println!("[Test] Creating 3 windows...");
    
    world.borrow_mut().world_mut().spawn((
        Window {
            title: "Test Window 1".to_string(),
            ..Default::default()
        },
        WindowPos {
            position: Some(POINT { x: 100, y: 100 }),
            size: Some(SIZE { cx: 400, cy: 300 }),
            ..Default::default()
        },
    ));
    
    world.borrow_mut().world_mut().spawn((
        Window {
            title: "Test Window 2".to_string(),
            ..Default::default()
        },
        WindowPos {
            position: Some(POINT { x: 550, y: 100 }),
            size: Some(SIZE { cx: 400, cy: 300 }),
            ..Default::default()
        },
    ));
    
    world.borrow_mut().world_mut().spawn((
        Window {
            title: "Test Window 3".to_string(),
            ..Default::default()
        },
        WindowPos {
            position: Some(POINT { x: 325, y: 450 }),
            size: Some(SIZE { cx: 400, cy: 300 }),
            ..Default::default()
        },
    ));
    
    println!("[Test] Windows spawned. Running schedules...\n");
    
    // スケジュールを1回実行（ウィンドウ作成とグラフィックス初期化）
    world.borrow_mut().try_tick_world();
    
    println!("\n[Test] Verifying all windows have graphics components...\n");
    
    // 検証: 3つのエンティティがすべてのコンポーネントを持つことを確認
    use bevy_ecs::prelude::*;
    use wintf::ecs::{WindowGraphics, Visual};
    
    let entities: Vec<(Entity, HWND)> = {
        let world_ref = world.borrow();
        let world_inner = world_ref.world();
        let mut query = world_inner.query::<(Entity, &WindowHandle, &WindowGraphics, &Visual)>();
        query.iter(world_inner).map(|(e, h, _g, _v)| (e, h.hwnd)).collect()
    };
    
    if entities.len() != 3 {
        println!("[TEST FAIL] Expected 3 entities, found {}", entities.len());
        println!("Press Enter to exit...");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        return Ok(());
    }
    
    println!("[TEST PASS] Found 3 entities with all graphics components");
    
    // 各エンティティの情報を表示
    for (entity, hwnd) in entities {
        println!("\n[Test] Entity {:?} (HWND: {:?}):", entity, hwnd);
        println!("  [PASS] WindowHandle + WindowGraphics + Visual present");
    }
    
    println!("\n========================================");
    println!("✅ [TEST SUCCESS] All windows initialized correctly!");
    println!("========================================\n");
    
    println!("Press Enter to exit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    
    Ok(())
}
