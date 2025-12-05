//! αマスクデータ構造
//!
//! ビットパック形式（1ビット/ピクセル）でαマスクを保持し、
//! 高速なヒット判定を提供する。

/// 固定閾値（α ≧ 128 でヒット対象）
const ALPHA_THRESHOLD: u8 = 128;

/// αマスクデータ構造
///
/// ビットパック形式でマスクデータを保持:
/// - 1ビット/ピクセル（8ピクセル/バイト）
/// - MSBファースト（ビット7 = 最左ピクセル）
/// - 行ごとに8ピクセル単位でアラインメント
#[derive(Debug, Clone)]
pub struct AlphaMask {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl AlphaMask {
    /// PBGRA32ピクセルデータからαマスクを生成
    ///
    /// # Arguments
    /// - `pixels`: PBGRA32形式のピクセルデータ（B, G, R, A の順）
    /// - `width`: 画像幅（ピクセル）
    /// - `height`: 画像高さ（ピクセル）
    /// - `stride`: 行あたりのバイト数
    ///
    /// # Returns
    /// 生成されたαマスク
    pub fn from_pbgra32(pixels: &[u8], width: u32, height: u32, stride: u32) -> Self {
        // 行あたりのバイト数（8ピクセル単位でアラインメント）
        let row_bytes = ((width + 7) / 8) as usize;
        let mut data = vec![0u8; row_bytes * height as usize];

        for y in 0..height {
            let row_start = (y * stride) as usize;
            for x in 0..width {
                // PBGRA32: 4バイト/ピクセル、α値は4バイト目（オフセット+3）
                let pixel_offset = row_start + (x as usize * 4);
                let alpha = pixels.get(pixel_offset + 3).copied().unwrap_or(0);

                // 閾値128で2値化
                if alpha >= ALPHA_THRESHOLD {
                    // MSBファースト: ビット7 = 最左ピクセル
                    let byte_index = (y as usize * row_bytes) + (x as usize / 8);
                    let bit_index = 7 - (x % 8);
                    data[byte_index] |= 1 << bit_index;
                }
            }
        }

        Self {
            data,
            width,
            height,
        }
    }

    /// 指定座標がヒット対象かを判定
    ///
    /// # Returns
    /// - `true`: α ≧ 128 のピクセル（ヒット対象）
    /// - `false`: α < 128 または範囲外
    pub fn is_hit(&self, x: u32, y: u32) -> bool {
        // 範囲外チェック
        if x >= self.width || y >= self.height {
            return false;
        }

        let row_bytes = ((self.width + 7) / 8) as usize;
        let byte_index = (y as usize * row_bytes) + (x as usize / 8);
        let bit_index = 7 - (x % 8);

        (self.data[byte_index] >> bit_index) & 1 == 1
    }

    /// マスク幅を取得
    pub fn width(&self) -> u32 {
        self.width
    }

    /// マスク高さを取得
    pub fn height(&self) -> u32 {
        self.height
    }
}

// Send + Sync は自動導出（Vec<u8> と u32 のみ）

#[cfg(test)]
mod tests {
    use super::*;

    /// 透明部分（α < 128）でヒットしないことを確認
    #[test]
    fn test_transparent_pixel_not_hit() {
        // 2x2 PBGRA32: 全て透明（α = 0）
        let pixels = vec![
            0, 0, 0, 0, // (0,0) 透明
            0, 0, 0, 0, // (1,0) 透明
            0, 0, 0, 0, // (0,1) 透明
            0, 0, 0, 0, // (1,1) 透明
        ];
        let mask = AlphaMask::from_pbgra32(&pixels, 2, 2, 8);

        assert!(!mask.is_hit(0, 0));
        assert!(!mask.is_hit(1, 0));
        assert!(!mask.is_hit(0, 1));
        assert!(!mask.is_hit(1, 1));
    }

    /// 不透明部分（α ≧ 128）でヒットすることを確認
    #[test]
    fn test_opaque_pixel_hit() {
        // 2x2 PBGRA32: 全て不透明（α = 255）
        let pixels = vec![
            0, 0, 0, 255, // (0,0) 不透明
            0, 0, 0, 255, // (1,0) 不透明
            0, 0, 0, 255, // (0,1) 不透明
            0, 0, 0, 255, // (1,1) 不透明
        ];
        let mask = AlphaMask::from_pbgra32(&pixels, 2, 2, 8);

        assert!(mask.is_hit(0, 0));
        assert!(mask.is_hit(1, 0));
        assert!(mask.is_hit(0, 1));
        assert!(mask.is_hit(1, 1));
    }

    /// 境界値テスト: α = 127 → ヒットしない
    #[test]
    fn test_threshold_boundary_127() {
        let pixels = vec![0, 0, 0, 127]; // α = 127
        let mask = AlphaMask::from_pbgra32(&pixels, 1, 1, 4);

        assert!(!mask.is_hit(0, 0));
    }

    /// 境界値テスト: α = 128 → ヒットする
    #[test]
    fn test_threshold_boundary_128() {
        let pixels = vec![0, 0, 0, 128]; // α = 128
        let mask = AlphaMask::from_pbgra32(&pixels, 1, 1, 4);

        assert!(mask.is_hit(0, 0));
    }

    /// 範囲外座標でfalseを返すことを確認
    #[test]
    fn test_out_of_bounds() {
        let pixels = vec![0, 0, 0, 255];
        let mask = AlphaMask::from_pbgra32(&pixels, 1, 1, 4);

        assert!(!mask.is_hit(1, 0)); // x = width
        assert!(!mask.is_hit(0, 1)); // y = height
        assert!(!mask.is_hit(100, 100));
    }

    /// 混在パターン: 透明/不透明が混在
    #[test]
    fn test_mixed_pattern() {
        // 2x2: チェッカーパターン
        let pixels = vec![
            0, 0, 0, 255, // (0,0) 不透明
            0, 0, 0, 0,   // (1,0) 透明
            0, 0, 0, 0,   // (0,1) 透明
            0, 0, 0, 255, // (1,1) 不透明
        ];
        let mask = AlphaMask::from_pbgra32(&pixels, 2, 2, 8);

        assert!(mask.is_hit(0, 0));
        assert!(!mask.is_hit(1, 0));
        assert!(!mask.is_hit(0, 1));
        assert!(mask.is_hit(1, 1));
    }

    /// 8ピクセル以上の幅でビットパックが正しく動作することを確認
    #[test]
    fn test_wide_image_bit_packing() {
        // 10x1: 最初と最後のピクセルが不透明
        let mut pixels = vec![0u8; 10 * 4];
        pixels[3] = 255; // (0,0) 不透明
        pixels[9 * 4 + 3] = 255; // (9,0) 不透明

        let mask = AlphaMask::from_pbgra32(&pixels, 10, 1, 40);

        assert!(mask.is_hit(0, 0));
        for x in 1..9 {
            assert!(!mask.is_hit(x, 0), "x={} should not hit", x);
        }
        assert!(mask.is_hit(9, 0));
    }

    /// サイズアクセサのテスト
    #[test]
    fn test_size_accessors() {
        let pixels = vec![0u8; 100 * 50 * 4];
        let mask = AlphaMask::from_pbgra32(&pixels, 100, 50, 400);

        assert_eq!(mask.width(), 100);
        assert_eq!(mask.height(), 50);
    }
}
