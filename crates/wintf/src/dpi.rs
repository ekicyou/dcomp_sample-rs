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

use windows_numerics::Vector2;

/// 論理ピクセル座標（windows_numerics::Vector2のエイリアス）
pub type LxPoint = Vector2;

/// 論理ピクセルサイズ（windows_numerics::Vector2のエイリアス）
pub type LxSize = Vector2;

/// 物理ピクセル座標（windows_numerics::Vector2のエイリアス）
pub type PxPoint = Vector2;

/// 物理ピクセルサイズ（windows_numerics::Vector2のエイリアス）
pub type PxSize = Vector2;

/// Vector2用のヘルパー関数
pub trait Vector2Ext {
    fn from_lengths(x: LxLength, y: LxLength) -> Self;
    fn to_physical(&self, dpi: impl ScaleFactor) -> Vector2;
    fn to_logical(&self, dpi: impl ScaleFactor) -> Vector2;
    fn into_raw(self) -> RawSize;
}

impl Vector2Ext for Vector2 {
    fn from_lengths(x: LxLength, y: LxLength) -> Self {
        Vector2 { X: x.0, Y: y.0 }
    }
    
    fn to_physical(&self, dpi: impl ScaleFactor) -> Vector2 {
        let scale = dpi.scale_factor();
        Vector2 {
            X: self.X * scale,
            Y: self.Y * scale,
        }
    }
    
    fn to_logical(&self, dpi: impl ScaleFactor) -> Vector2 {
        let scale = dpi.scale_factor();
        Vector2 {
            X: self.X / scale,
            Y: self.Y / scale,
        }
    }
    
    fn into_raw(self) -> RawSize {
        RawSize {
            width: self.X.ceil() as i32,
            height: self.Y.ceil() as i32,
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

impl IntoDpi<Vector2> for Vector2 {
    fn into_dpi(self, dpi: impl ScaleFactor) -> Vector2 {
        // Vector2はコンテキストによって論理→物理、物理→論理の両方に使えるため
        // 明示的にto_physicalまたはto_logicalを使う方が推奨されますが
        // 既存コードの互換性のため、論理→物理として扱います
        self.to_physical(dpi)
    }
}

impl IntoDpi<PxLength> for LxLength {
    fn into_dpi(self, dpi: impl ScaleFactor) -> PxLength {
        PxLength(self.to_physical(dpi))
    }
}
