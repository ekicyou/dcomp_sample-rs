#![allow(non_snake_case)]

use ambassador::*;
use bevy_ecs::prelude::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::*;

fn dpi_to_scale_factor(dpi: f32) -> f32 {
    dpi / 96.0
}

#[delegatable_trait]
pub trait ScaleFactor: Clone + Copy {
    fn value(&self) -> f32;

    #[inline]
    fn scale_factor(&self) -> f32 {
        dpi_to_scale_factor(self.value())
    }
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Dpi {
    dpi: f32,
    scale_factor: f32,
}

impl Dpi {
    pub fn new(dpi: f32) -> Self {
        Self {
            dpi,
            scale_factor: dpi_to_scale_factor(dpi),
        }
    }

    pub fn from_WM_DPICHANGED(wparam: WPARAM, _lparam: LPARAM) -> Self {
        let (x_dpi, _) = (wparam.0 as u16 as f32, (wparam.0 >> 16) as f32);
        Self::new(x_dpi)
    }

    #[inline]
    pub fn value(&self) -> f32 {
        self.dpi
    }

    #[inline]
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }
}

impl ScaleFactor for Dpi {
    #[inline]
    fn value(&self) -> f32 {
        self.dpi
    }

    #[inline]
    fn scale_factor(&self) -> f32 {
        self.scale_factor
    }
}

#[delegatable_trait]
pub trait SetDpi {
    fn set_dpi(&self, dpi: impl ScaleFactor);
}

impl SetDpi for ID2D1RenderTarget {
    fn set_dpi(&self, dpi: impl ScaleFactor) {
        unsafe {
            self.SetDpi(dpi.value(), dpi.value());
        }
    }
}

//=============================================================
// シンプルな型定義（euclid不使用）
//=============================================================

/// 物理ピクセル座標（i32）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RawPoint {
    pub x: i32,
    pub y: i32,
}

impl RawPoint {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// 物理ピクセルサイズ（i32）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RawSize {
    pub width: i32,
    pub height: i32,
}

impl RawSize {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}

/// 論理ピクセル長（f32）
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LxLength(pub f32);

impl LxLength {
    pub const fn new(value: f32) -> Self {
        Self(value)
    }
    
    pub fn to_physical(&self, dpi: impl ScaleFactor) -> f32 {
        self.0 * dpi.scale_factor()
    }
}

impl std::ops::Add for LxLength {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Mul<f32> for LxLength {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self(self.0 * rhs)
    }
}

/// 論理ピクセル座標（f32）
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LxPoint {
    pub x: f32,
    pub y: f32,
}

impl LxPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub fn from_lengths(x: LxLength, y: LxLength) -> Self {
        Self { x: x.0, y: y.0 }
    }
    
    pub fn to_physical(&self, dpi: impl ScaleFactor) -> PxPoint {
        let scale = dpi.scale_factor();
        PxPoint {
            x: self.x * scale,
            y: self.y * scale,
        }
    }
}

/// 論理ピクセルサイズ（f32）
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LxSize {
    pub width: f32,
    pub height: f32,
}

impl LxSize {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
    
    pub fn to_physical(&self, dpi: impl ScaleFactor) -> PxSize {
        let scale = dpi.scale_factor();
        PxSize {
            width: self.width * scale,
            height: self.height * scale,
        }
    }
}

/// 物理ピクセル座標（f32）
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PxPoint {
    pub x: f32,
    pub y: f32,
}

impl PxPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub fn to_logical(&self, dpi: impl ScaleFactor) -> LxPoint {
        let scale = dpi.scale_factor();
        LxPoint {
            x: self.x / scale,
            y: self.y / scale,
        }
    }
}

/// 物理ピクセルサイズ（f32）
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PxSize {
    pub width: f32,
    pub height: f32,
}

impl PxSize {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
    
    pub fn to_logical(&self, dpi: impl ScaleFactor) -> LxSize {
        let scale = dpi.scale_factor();
        LxSize {
            width: self.width / scale,
            height: self.height / scale,
        }
    }
    
    pub fn into_raw(self) -> RawSize {
        RawSize {
            width: self.width.ceil() as i32,
            height: self.height.ceil() as i32,
        }
    }
}

/// 物理ピクセル長（f32）
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PxLength(pub f32);

impl PxLength {
    pub const fn new(value: f32) -> Self {
        Self(value)
    }
}

// DPI変換のための便利メソッド
pub trait IntoDpi<T> {
    fn into_dpi(self, dpi: impl ScaleFactor) -> T;
}

impl IntoDpi<PxSize> for LxSize {
    fn into_dpi(self, dpi: impl ScaleFactor) -> PxSize {
        self.to_physical(dpi)
    }
}

impl IntoDpi<PxPoint> for LxPoint {
    fn into_dpi(self, dpi: impl ScaleFactor) -> PxPoint {
        self.to_physical(dpi)
    }
}

impl IntoDpi<PxLength> for LxLength {
    fn into_dpi(self, dpi: impl ScaleFactor) -> PxLength {
        PxLength(self.to_physical(dpi))
    }
}

impl IntoDpi<LxPoint> for PxPoint {
    fn into_dpi(self, dpi: impl ScaleFactor) -> LxPoint {
        self.to_logical(dpi)
    }
}
