//! BitmapSource リソースコンポーネント
//!
//! CPU側（BitmapSourceResource）とGPU側（BitmapSourceGraphics）のリソース。

use bevy_ecs::prelude::*;
use windows::Win32::Graphics::Direct2D::ID2D1Bitmap1;
use windows::Win32::Graphics::Imaging::IWICBitmapSource;

/// CPU側画像リソース（WIC BitmapSource）
///
/// # Thread Safety
/// IWICBitmapSourceはthread-free marshaling対応のため
/// Send + Syncを手動実装する。
#[derive(Component)]
pub struct BitmapSourceResource {
    source: IWICBitmapSource,
}

unsafe impl Send for BitmapSourceResource {}
unsafe impl Sync for BitmapSourceResource {}

impl BitmapSourceResource {
    /// WIC BitmapSourceから作成
    pub fn new(source: IWICBitmapSource) -> Self {
        Self { source }
    }

    /// BitmapSourceへの参照を取得
    pub fn source(&self) -> &IWICBitmapSource {
        &self.source
    }
}

/// GPU側画像リソース（D2D Bitmap）
///
/// BitmapSourceのon_add時にOption::Noneで作成され、
/// BitmapSourceResourceが追加されたらD2D Bitmapを生成する。
///
/// # Device Lost対応
/// 既存のVisualGraphics/SurfaceGraphicsと同じパターン:
/// - invalidate_dependent_componentsシステムがDevice Lost時にinvalidate()を呼ぶ
/// - 次フレームでis_valid() == falseを検出しBitmapを再生成
#[derive(Component, Default)]
pub struct BitmapSourceGraphics {
    bitmap: Option<ID2D1Bitmap1>,
}

unsafe impl Send for BitmapSourceGraphics {}
unsafe impl Sync for BitmapSourceGraphics {}

impl BitmapSourceGraphics {
    /// 空のBitmapSourceGraphicsを作成
    pub fn new() -> Self {
        Self { bitmap: None }
    }

    /// Bitmapへの参照を取得
    pub fn bitmap(&self) -> Option<&ID2D1Bitmap1> {
        self.bitmap.as_ref()
    }

    /// Bitmapを設定
    pub fn set_bitmap(&mut self, bitmap: ID2D1Bitmap1) {
        self.bitmap = Some(bitmap);
    }

    /// Device Lost時にBitmapを無効化
    pub fn invalidate(&mut self) {
        self.bitmap = None;
    }

    /// Bitmapが有効か判定
    pub fn is_valid(&self) -> bool {
        self.bitmap.is_some()
    }
}

impl std::fmt::Debug for BitmapSourceGraphics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitmapSourceGraphics")
            .field("bitmap", &self.bitmap.is_some())
            .finish()
    }
}
