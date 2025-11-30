//! BitmapSource 画像読み込み統合テスト
//!
//! Task 7.2: 画像読み込みテストの実装
//! - test_8x8_rgba.png（αチャネル付き）の正常読み込み確認
//! - test_8x8_rgb.bmp（αなし）のPBGRA32変換確認
//! - invalid.binのエラーハンドリング確認

use std::path::PathBuf;
use windows::Win32::Graphics::Imaging::IWICBitmapSource;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};
use windows_core::Interface;
use wintf::ecs::widget::bitmap_source::systems::load_bitmap_source;
use wintf::ecs::widget::bitmap_source::WicCore;

/// テストアセットディレクトリのパスを取得
fn test_asset_path(filename: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("tests")
        .join("assets")
        .join(filename)
}

/// COMを初期化するヘルパー
fn with_com_initialized<T, F: FnOnce() -> T>(f: F) -> T {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
    let result = f();
    unsafe {
        CoUninitialize();
    }
    result
}

// ============================================================
// PNG画像読み込みテスト（αチャネル付き）
// ============================================================

#[test]
fn test_load_png_with_alpha() {
    with_com_initialized(|| {
        let wic_core = WicCore::new().expect("WicCore creation failed");
        let path = test_asset_path("test_8x8_rgba.png");

        // 画像読み込み
        let result = load_bitmap_source(wic_core.factory(), &path);
        assert!(
            result.is_ok(),
            "PNG loading should succeed: {:?}",
            result.err()
        );

        let source = result.unwrap();

        // 画像サイズの確認（8x8）
        let (width, height) = get_bitmap_size(&source);
        assert_eq!(width, 8, "width should be 8");
        assert_eq!(height, 8, "height should be 8");
    });
}

#[test]
fn test_load_png_16x16() {
    with_com_initialized(|| {
        let wic_core = WicCore::new().expect("WicCore creation failed");
        let path = test_asset_path("test_16x16_rgba.png");

        // 画像読み込み
        let result = load_bitmap_source(wic_core.factory(), &path);
        assert!(
            result.is_ok(),
            "16x16 PNG loading should succeed: {:?}",
            result.err()
        );

        let source = result.unwrap();

        // 画像サイズの確認（16x16）
        let (width, height) = get_bitmap_size(&source);
        assert_eq!(width, 16, "width should be 16");
        assert_eq!(height, 16, "height should be 16");
    });
}

// ============================================================
// BMP画像読み込みテスト（αなし→PBGRA32変換）
// ============================================================

#[test]
fn test_load_bmp_without_alpha() {
    with_com_initialized(|| {
        let wic_core = WicCore::new().expect("WicCore creation failed");
        let path = test_asset_path("test_8x8_rgb.bmp");

        // 画像読み込み（PBGRA32に変換される）
        let result = load_bitmap_source(wic_core.factory(), &path);
        assert!(
            result.is_ok(),
            "BMP loading should succeed (PBGRA32 conversion): {:?}",
            result.err()
        );

        let source = result.unwrap();

        // 画像サイズの確認（8x8）
        let (width, height) = get_bitmap_size(&source);
        assert_eq!(width, 8, "width should be 8");
        assert_eq!(height, 8, "height should be 8");

        // PBGRA32形式に変換されていることを確認
        let format = get_pixel_format(&source);
        // GUID_WICPixelFormat32bppPBGRA = 6fddc324-4e03-4bfe-b185-3d77768dc910
        let expected_format =
            windows::core::GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc910);
        assert_eq!(format, expected_format, "pixel format should be PBGRA32");
    });
}

// ============================================================
// 無効ファイルのエラーハンドリングテスト
// ============================================================

#[test]
fn test_load_invalid_file() {
    with_com_initialized(|| {
        let wic_core = WicCore::new().expect("WicCore creation failed");
        let path = test_asset_path("invalid.bin");

        // 無効なファイルの読み込みはエラーになるはず
        let result = load_bitmap_source(wic_core.factory(), &path);
        assert!(result.is_err(), "Invalid file should fail to load");
    });
}

#[test]
fn test_load_nonexistent_file() {
    with_com_initialized(|| {
        let wic_core = WicCore::new().expect("WicCore creation failed");
        let path = test_asset_path("nonexistent_file.png");

        // 存在しないファイルの読み込みはエラーになるはず
        let result = load_bitmap_source(wic_core.factory(), &path);
        assert!(result.is_err(), "Nonexistent file should fail to load");
    });
}

// ============================================================
// ヘルパー関数
// ============================================================

/// IWICBitmapSourceから画像サイズを取得
fn get_bitmap_size(source: &IWICBitmapSource) -> (u32, u32) {
    unsafe {
        let mut width = 0u32;
        let mut height = 0u32;
        source
            .GetSize(&mut width, &mut height)
            .expect("GetSize failed");
        (width, height)
    }
}

/// IWICBitmapSourceからピクセル形式を取得
fn get_pixel_format(source: &IWICBitmapSource) -> windows::core::GUID {
    unsafe { source.GetPixelFormat().expect("GetPixelFormat failed") }
}

// ============================================================
// WicCoreの基本テスト
// ============================================================

#[test]
fn test_wic_core_creation_in_integration() {
    with_com_initialized(|| {
        let result = WicCore::new();
        assert!(
            result.is_ok(),
            "WicCore creation should succeed in integration test"
        );

        let wic_core = result.unwrap();
        assert!(
            !wic_core.factory().as_raw().is_null(),
            "factory should not be null"
        );
    });
}

#[test]
fn test_wic_core_clone_in_integration() {
    with_com_initialized(|| {
        let wic_core = WicCore::new().expect("WicCore creation failed");
        let cloned = wic_core.clone();

        // 両方のファクトリが有効であることを確認
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
