//! テスト画像生成ユーティリティ
//!
//! `cargo run --example generate_test_image` で実行

use image::{Rgba, RgbaImage};
use std::path::Path;

fn main() {
    let assets_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("assets");

    // 64x64 白(半透明)/透明 チェッカーパターン（α確認用）
    generate_checker_pattern(
        &assets_dir.join("demo_checker_64x64.png"),
        64,
        8,
        Rgba([255, 255, 255, 200]), // 白（半透明 α≈78%）
        Rgba([0, 0, 0, 0]),         // 完全透明
    );

    println!("Generated: demo_checker_64x64.png (white=semi-transparent, other=fully transparent)");
}

fn generate_checker_pattern(
    path: &Path,
    size: u32,
    block_size: u32,
    color1: Rgba<u8>,
    color2: Rgba<u8>,
) {
    let mut img = RgbaImage::new(size, size);

    for y in 0..size {
        for x in 0..size {
            let block_x = x / block_size;
            let block_y = y / block_size;
            let color = if (block_x + block_y) % 2 == 0 {
                color1
            } else {
                color2
            };
            img.put_pixel(x, y, color);
        }
    }

    img.save(path).expect("Failed to save image");
}
