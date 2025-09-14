#![allow(non_snake_case)]
#![allow(unused_variables)]

use ambassador::*;
use euclid::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows_numerics::*;

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
    fn to_physical(&self, a: f32) -> f32 {
        a * self.scale_factor()
    }

    #[inline]
    fn to_logical(&self, a: f32) -> f32 {
        a / self.scale_factor()
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

impl<Unit> From<Rect<f32, Unit>> for D2D_RECT_F {
    fn from(r: Rect<f32, Unit>) -> Self {
        Self {
            left: r.origin.x,
            top: r.origin.y,
            right: r.origin.x + r.size.width,
            bottom: r.origin.y + r.size.height,
        }
    }
}

impl<Unit> From<D2D_RECT_F> for Rect<f32, Unit> {
    fn from(r: D2D_RECT_F) -> Self {
        let origin = Point2D::new(r.left, r.top);
        let size = Size2D::new(r.right - r.left, r.bottom - r.top);
        Self::new(origin, size)
    }
}

pub trait Scalable {
    fn to_physical(self, dpi: impl ScaleFactor) -> Self;
    fn to_logical(self, dpi: impl ScaleFactor) -> Self;
}

impl Scalable for f32 {
    fn to_physical(self, dpi: impl ScaleFactor) -> Self {
        dpi.to_physical(self)
    }

    fn to_logical(self, dpi: impl ScaleFactor) -> Self {
        dpi.to_logical(self)
    }
}

impl Scalable for Vector2 {
    fn to_physical(self, dpi: impl ScaleFactor) -> Self {
        Self {
            X: dpi.to_physical(self.X),
            Y: dpi.to_physical(self.Y),
        }
    }

    fn to_logical(self, dpi: impl ScaleFactor) -> Self {
        Self {
            X: dpi.to_logical(self.X),
            Y: dpi.to_logical(self.Y),
        }
    }
}

impl Scalable for Rect {
    fn to_physical(self, dpi: impl ScaleFactor) -> Self {
        Rect {
            point: self.point.to_physical(dpi),
            size: self.size.to_physical(dpi),
        }
    }

    fn to_logical(self, dpi: impl ScaleFactor) -> Self {
        Rect {
            point: self.point.to_logical(dpi),
            size: self.size.to_logical(dpi),
        }
    }
}
