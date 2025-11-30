//! BitmapSource モジュールのユニットテスト

use super::*;
use windows_core::Interface;

// ============================================================
// Task 1.1: WicCore Tests
// ============================================================

// COMを初期化するヘルパー
fn with_com_initialized<F: FnOnce()>(f: F) {
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};
    // COINIT_MULTITHREADED for WIC free-threaded factory
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
    f();
    unsafe {
        CoUninitialize();
    }
}

#[test]
fn test_wic_core_creation() {
    // WicCoreが正常に作成できることを確認
    with_com_initialized(|| {
        let result = WicCore::new();
        assert!(result.is_ok(), "WicCore creation should succeed");
    });
}

#[test]
fn test_wic_core_factory_access() {
    // factory()アクセサが有効な参照を返すことを確認
    with_com_initialized(|| {
        let wic_core = WicCore::new().expect("WicCore creation failed");
        let factory = wic_core.factory();
        // factory が存在することを確認（nullではない）
        assert!(!factory.as_raw().is_null(), "factory should not be null");
    });
}

#[test]
fn test_wic_core_clone() {
    // Cloneトレイトが正しく実装されていることを確認
    with_com_initialized(|| {
        let wic_core = WicCore::new().expect("WicCore creation failed");
        let cloned = wic_core.clone();
        // 両方のfactoryが有効であることを確認
        assert!(
            !wic_core.factory().as_raw().is_null(),
            "original factory should be valid"
        );
        assert!(
            !cloned.factory().as_raw().is_null(),
            "cloned factory should be valid"
        );
    });
}

#[test]
fn test_wic_core_send_sync() {
    // Send + Syncトレイトが実装されていることをコンパイル時に確認
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<WicCore>();
}

// ============================================================
// Task 1.2: WintfTaskPool Tests
// ============================================================

#[test]
fn test_wintf_task_pool_creation() {
    // WintfTaskPoolが正常に作成できることを確認
    let task_pool = WintfTaskPool::new();
    // 作成直後はコマンドキューが空
    assert!(
        task_pool.is_empty(),
        "new task pool should have empty queue"
    );
}

#[test]
fn test_wintf_task_pool_drain_empty() {
    // 空のプールでdrain_and_applyが安全に動作することを確認
    use bevy_ecs::prelude::*;
    let task_pool = WintfTaskPool::new();
    let mut world = World::new();
    // パニックしないことを確認
    task_pool.drain_and_apply(&mut world);
}

#[test]
fn test_wintf_task_pool_command_send_receive() {
    // spawnで送信したコマンドがdrain_and_applyで実行されることを確認
    use bevy_ecs::prelude::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let task_pool = WintfTaskPool::new();
    let executed = Arc::new(AtomicBool::new(false));
    let executed_clone = executed.clone();

    // クロージャベースのコマンドを送信（BoxedCommandはFnOnce）
    task_pool.send_command(Box::new(move |_world: &mut World| {
        executed_clone.store(true, Ordering::SeqCst);
    }));

    // drain_and_apply実行
    let mut world = World::new();
    task_pool.drain_and_apply(&mut world);

    assert!(
        executed.load(Ordering::SeqCst),
        "command should be executed"
    );
}

// ============================================================
// Task 2.1: BitmapSource Component Tests
// ============================================================

#[test]
fn test_bitmap_source_creation() {
    // BitmapSourceが正しくパスを保持することを確認
    let bitmap_source = BitmapSource::new("test/path.png");
    assert_eq!(bitmap_source.path, "test/path.png");
}

#[test]
fn test_bitmap_source_from_string() {
    // String型からも作成できることを確認
    let path = String::from("assets/image.png");
    let bitmap_source = BitmapSource::new(path);
    assert_eq!(bitmap_source.path, "assets/image.png");
}

// ============================================================
// Task 2.2: BitmapSourceResource Tests
// ============================================================

#[test]
fn test_bitmap_source_resource_send_sync() {
    // Send + Syncトレイトが実装されていることをコンパイル時に確認
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<BitmapSourceResource>();
}

// ============================================================
// Task 2.3: BitmapSourceGraphics Tests
// ============================================================

#[test]
fn test_bitmap_source_graphics_new() {
    // 空のBitmapSourceGraphicsが作成できることを確認
    let graphics = BitmapSourceGraphics::new();
    assert!(!graphics.is_valid(), "new graphics should not be valid");
    assert!(graphics.bitmap().is_none(), "bitmap should be None");
}

#[test]
fn test_bitmap_source_graphics_invalidate() {
    // invalidate()でbitmap がNoneになることを確認
    let mut graphics = BitmapSourceGraphics::new();
    graphics.invalidate();
    assert!(
        !graphics.is_valid(),
        "invalidated graphics should not be valid"
    );
}

#[test]
fn test_bitmap_source_graphics_send_sync() {
    // Send + Syncトレイトが実装されていることをコンパイル時に確認
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<BitmapSourceGraphics>();
}
