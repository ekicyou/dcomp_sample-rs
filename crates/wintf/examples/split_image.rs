//! 画像を64x64のタイルに分割するユーティリティ

use image::{GenericImageView, ImageFormat};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/assets/seikatu.webp");
    let output_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/assets");

    println!("Loading: {}", input_path);
    let img = image::open(input_path)?;
    let (width, height) = img.dimensions();
    println!("Image size: {}x{}", width, height);

    let tile_size = 64u32;
    let cols = width / tile_size;
    let rows = height / tile_size;
    println!(
        "Splitting into {}x{} tiles ({} total)",
        cols,
        rows,
        cols * rows
    );

    for row in 0..rows {
        for col in 0..cols {
            let x = col * tile_size;
            let y = row * tile_size;
            let tile = img.crop_imm(x, y, tile_size, tile_size);

            let output_name = format!("seikatu_{}_{}.webp", row, col);
            let output_path = Path::new(output_dir).join(&output_name);

            // WebP形式で保存
            tile.save_with_format(&output_path, ImageFormat::WebP)?;
            println!("Saved: {}", output_name);
        }
    }

    println!("Done! {} tiles created.", cols * rows);
    Ok(())
}
