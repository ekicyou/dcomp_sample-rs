/// GraphicsCore再初期化システムの単体テスト
///
/// このテストはGraphicsCore、WindowGraphics、Visual、Surfaceの
/// Option<T>ラップと状態遷移機能をテストします。
use wintf::ecs::{GraphicsCore, SurfaceGraphics, VisualGraphics};

#[test]
fn test_graphics_core_invalidate_and_is_valid() {
    let mut graphics = GraphicsCore::new().expect("GraphicsCore作成失敗");

    // 初期状態: 有効
    assert!(
        graphics.is_valid(),
        "GraphicsCore should be valid after creation"
    );

    // 無効化
    graphics.invalidate();
    assert!(
        !graphics.is_valid(),
        "GraphicsCore should be invalid after invalidate()"
    );

    // アクセサメソッドがNoneを返す
    assert!(
        graphics.d2d_factory().is_none(),
        "d2d_factory() should return None after invalidate()"
    );
    assert!(
        graphics.d2d_device().is_none(),
        "d2d_device() should return None after invalidate()"
    );
    assert!(
        graphics.dcomp().is_none(),
        "dcomp() should return None after invalidate()"
    );
    assert!(
        graphics.desktop().is_none(),
        "desktop() should return None after invalidate()"
    );
    assert!(
        graphics.dwrite_factory().is_none(),
        "dwrite_factory() should return None after invalidate()"
    );
}

#[test]
fn test_visual_new_and_invalidate() {
    use wintf::com::dcomp::DCompositionDeviceExt;

    let graphics = GraphicsCore::new().expect("GraphicsCore作成失敗");
    let dcomp = graphics.dcomp().expect("dcomp取得失敗");

    let visual_raw = dcomp.create_visual().expect("Visual作成失敗");
    let mut visual = VisualGraphics::new(visual_raw);

    // 初期状態: 有効
    assert!(visual.is_valid(), "Visual should be valid after creation");
    assert!(
        visual.visual().is_some(),
        "visual() should return Some after creation"
    );

    // 無効化
    visual.invalidate();
    assert!(
        !visual.is_valid(),
        "Visual should be invalid after invalidate()"
    );
    assert!(
        visual.visual().is_none(),
        "visual() should return None after invalidate()"
    );
}

#[test]
fn test_surface_new_and_invalidate() {
    use windows::Win32::Graphics::Dxgi::Common::*;
    use wintf::com::dcomp::DCompositionDeviceExt;

    let graphics = GraphicsCore::new().expect("GraphicsCore作成失敗");
    let dcomp = graphics.dcomp().expect("dcomp取得失敗");

    let surface_raw = dcomp
        .create_surface(
            800,
            600,
            DXGI_FORMAT_B8G8R8A8_UNORM,
            DXGI_ALPHA_MODE_PREMULTIPLIED,
        )
        .expect("Surface作成失敗");

    let mut surface = SurfaceGraphics::new(surface_raw, (800, 600));

    // 初期状態: 有効
    assert!(surface.is_valid(), "Surface should be valid after creation");
    assert!(
        surface.surface().is_some(),
        "surface() should return Some after creation"
    );

    // 無効化
    surface.invalidate();
    assert!(
        !surface.is_valid(),
        "Surface should be invalid after invalidate()"
    );
    assert!(
        surface.surface().is_none(),
        "surface() should return None after invalidate()"
    );
}

#[test]
fn test_graphics_core_reinitialize() {
    let mut graphics = GraphicsCore::new().expect("GraphicsCore作成失敗");

    // 初期状態
    assert!(graphics.is_valid());
    let initial_d2d = graphics.d2d_device().unwrap() as *const _;

    // 無効化
    graphics.invalidate();
    assert!(!graphics.is_valid());

    // 再初期化
    let new_graphics = GraphicsCore::new().expect("GraphicsCore再初期化失敗");
    assert!(new_graphics.is_valid());

    // 新しいインスタンスは異なるポインタを持つ
    let new_d2d = new_graphics.d2d_device().unwrap() as *const _;
    assert_ne!(
        initial_d2d, new_d2d,
        "Reinitialized GraphicsCore should have different device pointers"
    );
}
