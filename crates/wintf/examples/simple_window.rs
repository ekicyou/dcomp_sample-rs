#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::core::*;
use windows::Win32::Foundation::{POINT, SIZE};
use wintf::ecs::{Window, WindowPos};
use wintf::*;

fn main() -> Result<()> {
    human_panic::setup_panic!();

    let mgr = WinThreadMgr::new()?;
    let world = mgr.world();

    // 1つ目のWindowコンポーネントを持つEntityを作成
    world.borrow_mut().world_mut().spawn((
        Window {
            title: "wintf - ECS Window 1".to_string(),
            ..Default::default()
        },
        WindowPos {
            position: Some(POINT { x: 100, y: 100 }),
            size: Some(SIZE { cx: 800, cy: 600 }),
            ..Default::default()
        },
    ));

    // 2つ目のWindowコンポーネントを持つEntityを作成
    world.borrow_mut().world_mut().spawn((
        Window {
            title: "wintf - ECS Window 2".to_string(),
            ..Default::default()
        },
        WindowPos {
            position: Some(POINT { x: 950, y: 150 }),
            size: Some(SIZE { cx: 600, cy: 400 }),
            ..Default::default()
        },
    ));

    // メッセージループを開始（システムが自動的にウィンドウを作成）
    mgr.run()?;

    Ok(())
}
