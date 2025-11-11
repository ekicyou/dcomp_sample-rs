#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::core::*;
use wintf::ecs::Window;
use wintf::*;

fn main() -> Result<()> {
    human_panic::setup_panic!();

    let mgr = WinThreadMgr::new()?;
    let world = mgr.world();

    // Windowコンポーネントを持つEntityを作成
    world.borrow_mut().world_mut().spawn(Window {
        title: "wintf - ECS Window".to_string(),
        width: 800,
        height: 600,
        x: 100,
        y: 100,
        ..Default::default()
    });

    // メッセージループを開始（システムが自動的にウィンドウを作成）
    mgr.run()?;

    Ok(())
}
