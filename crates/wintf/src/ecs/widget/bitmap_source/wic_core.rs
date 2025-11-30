//! WicCore - WIC関連リソース（Device Lostの影響を受けない）
//!
//! WICはCPUベースのイメージ処理のため、GPUのDevice Lostとは独立。
//! GraphicsCore.invalidate()時もWicCoreは有効なまま。

use bevy_ecs::prelude::*;
use windows::core::Result;
use windows::Win32::Graphics::Imaging::D2D::*;
use windows::Win32::Graphics::Imaging::*;
use windows::Win32::System::Com::*;

/// WIC関連リソース
///
/// WICファクトリを保持し、画像デコード機能を提供する。
/// Device Lostの影響を受けない独立リソース。
#[derive(Resource, Clone)]
pub struct WicCore {
    factory: IWICImagingFactory2,
}

// IWICImagingFactory2はthread-free marshaling対応
unsafe impl Send for WicCore {}
unsafe impl Sync for WicCore {}

impl WicCore {
    /// WicCoreを作成
    pub fn new() -> Result<Self> {
        let factory: IWICImagingFactory2 =
            unsafe { CoCreateInstance(&CLSID_WICImagingFactory2, None, CLSCTX_INPROC_SERVER)? };
        Ok(Self { factory })
    }

    /// WICファクトリへの参照を取得
    pub fn factory(&self) -> &IWICImagingFactory2 {
        &self.factory
    }
}
