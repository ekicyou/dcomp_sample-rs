use bevy_ecs::prelude::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectComposition::*;

/// ウィンドウごとのグラフィックスリソース
#[derive(Component, Debug)]
pub struct WindowGraphics {
    /// DirectComposition Target (HWNDに関連付けられた合成ターゲット)
    pub target: IDCompositionTarget,
    /// Direct2D DeviceContext (このウィンドウでの描画に使用)
    pub device_context: ID2D1DeviceContext,
}

unsafe impl Send for WindowGraphics {}
unsafe impl Sync for WindowGraphics {}

impl WindowGraphics {
    /// IDCompositionTargetへの参照を取得する
    pub fn target(&self) -> &IDCompositionTarget {
        &self.target
    }

    /// ID2D1DeviceContextへの参照を取得する
    pub fn device_context(&self) -> &ID2D1DeviceContext {
        &self.device_context
    }
}

/// ウィンドウのルートビジュアルノード
#[derive(Component, Debug)]
pub struct Visual {
    /// ルートビジュアル（ビジュアルツリーの最上位ノード）
    pub visual: IDCompositionVisual3,
}

unsafe impl Send for Visual {}
unsafe impl Sync for Visual {}

impl Visual {
    /// IDCompositionVisual3への参照を取得する
    pub fn visual(&self) -> &IDCompositionVisual3 {
        &self.visual
    }
}

/// ウィンドウの描画サーフェス
#[derive(Component, Debug)]
pub struct Surface {
    /// DirectComposition Surface (描画ターゲット)
    pub surface: IDCompositionSurface,
}

unsafe impl Send for Surface {}
unsafe impl Sync for Surface {}

impl Surface {
    /// IDCompositionSurfaceへの参照を取得
    pub fn surface(&self) -> &IDCompositionSurface {
        &self.surface
    }
}
