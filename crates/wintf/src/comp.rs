use crate::dpi::*;
use ambassador::*;
use std::{cell::*, rc::*};
use windows::{
    core::*,
    Win32::Graphics::{Direct2D::*, DirectComposition::*},
};
use windows_numerics::*;

/// WinUI3 風の書き込み中心 CompVisual トレイト
/// - 同期的な getter を持たない（状態は UI 要素が真のソース）
/// - サーフェス／イメージは抽象的に受け取り、実装側で具体型へ変換して扱う
#[delegatable_trait]
pub trait CompVisual {
    // =====================================================================
    // 表示非表示制御
    fn set_is_visible(&self, visible: bool) -> Result<()>;

    // =====================================================================
    // 描画要素：子の管理
    fn add_child(&self, child: &dyn CompVisual) -> Result<()>;
    fn remove_child(&self, child: &dyn CompVisual) -> Result<()>;

    // =====================================================================
    // 描画要素：サーフェス／イメージ供給 API

    /// 描画コンテンツを与える。 IDCompositionSurface / ID2D1Imageを受け入れ可能
    fn set_content(&self, content: &IUnknown) -> Result<()> {
        {
            let image = content.cast::<ID2D1Image>();
            if let Ok(image) = image {
                return self.set_image(image);
            }
        }
        {
            let surface = content.cast::<IDCompositionSurface>()?;
            return self.set_surface(surface);
        }
    }

    /// ID2D1Image 系を content 要素として与える。
    fn set_image(&self, image: ID2D1Image) -> Result<()>;

    /// IDCompositionSurface を content 要素として与える。
    fn set_surface(&self, surface: IDCompositionSurface) -> Result<()>;

    // =====================================================================
    // 変換
    fn set_transform(&self, matrix: LxTransform3D) -> Result<()>;

    // =====================================================================
    // クリップ
    fn set_clip(&self, clip: Option<LxRect>) -> Result<()>;

    // =====================================================================
    // エフェクト
    fn set_opacity(&self, opacity: f32) -> Result<()>;

    // =====================================================================
    // ヒットテスト
    fn set_is_hit_test_visible(&self, hit_test: bool) -> Result<()>;
    fn set_size(&self, size: LxSize) -> Result<()>;

    // =====================================================================
    // 確定
    fn commit(&self) -> Result<()>;

    // =====================================================================
    // アニメーション

    fn set_transform_animation(
        &self,
        property: &str,
        animation: IDCompositionAnimation,
    ) -> Result<()>;
}

pub type VisualHandle<T: CompVisual + ?Sized> = Rc<RefCell<Box<T>>>;
