#![allow(non_snake_case)]
#![allow(unused_variables)]

use ambassador::*;
use euclid::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::*;

// Physical pixels
pub struct Px;

// Logical pixels
pub struct Lx;

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

    #[inline]
    fn to_physical(&self) -> Scale<f32, Lx, Px> {
        Scale::new(self.scale_factor())
    }

    #[inline]
    fn to_logical(&self) -> Scale<f32, Px, Lx> {
        Scale::new(1.0 / self.scale_factor())
    }
}

#[derive(Debug, Default, Clone, Copy)]
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

pub type PxLength = Length<f32, Px>;
pub type LxLength = Length<f32, Lx>;

pub type PxPoint = Point2D<f32, Px>;
pub type LxPoint = Point2D<f32, Lx>;

pub type PxSize = Size2D<f32, Px>;
pub type LxSize = Size2D<f32, Lx>;

pub type PxRect = Rect<f32, Px>;
pub type LxRect = Rect<f32, Lx>;

pub trait FromDpi<T> {
    fn from_dpi(value: T, dpi: impl ScaleFactor) -> Self;
}

pub trait IntoDpi<T> {
    fn into_dpi(self, dpi: impl ScaleFactor) -> T;
}

impl<T, U: FromDpi<T>> IntoDpi<U> for T {
    fn into_dpi(self, dpi: impl ScaleFactor) -> U {
        U::from_dpi(self, dpi)
    }
}

impl FromDpi<PxLength> for LxLength {
    fn from_dpi(value: PxLength, dpi: impl ScaleFactor) -> Self {
        value * dpi.to_logical()
    }
}
impl FromDpi<LxLength> for PxLength {
    fn from_dpi(value: LxLength, dpi: impl ScaleFactor) -> Self {
        value * dpi.to_physical()
    }
}

impl FromDpi<PxPoint> for LxPoint {
    fn from_dpi(value: PxPoint, dpi: impl ScaleFactor) -> Self {
        value * dpi.to_logical()
    }
}
impl FromDpi<LxPoint> for PxPoint {
    fn from_dpi(value: LxPoint, dpi: impl ScaleFactor) -> Self {
        value * dpi.to_physical()
    }
}

impl FromDpi<PxSize> for LxSize {
    fn from_dpi(value: PxSize, dpi: impl ScaleFactor) -> Self {
        value * dpi.to_logical()
    }
}
impl FromDpi<LxSize> for PxSize {
    fn from_dpi(value: LxSize, dpi: impl ScaleFactor) -> Self {
        value * dpi.to_physical()
    }
}

impl FromDpi<PxRect> for LxRect {
    fn from_dpi(value: PxRect, dpi: impl ScaleFactor) -> Self {
        value * dpi.to_logical()
    }
}
impl FromDpi<LxRect> for PxRect {
    fn from_dpi(value: LxRect, dpi: impl ScaleFactor) -> Self {
        value * dpi.to_physical()
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
