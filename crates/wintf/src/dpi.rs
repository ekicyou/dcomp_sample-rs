#![allow(non_snake_case)]
#![allow(unused_variables)]

use ambassador::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows_numerics::*;

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

pub type Point = Vector2;

pub trait PointExt<T> {
    fn x(&self) -> T;
    fn y(&self) -> T;
}

impl PointExt<f32> for Point {
    fn x(&self) -> f32 {
        self.X
    }
    fn y(&self) -> f32 {
        self.Y
    }
}

pub type Size = Vector2;

pub trait SizeExt<T> {
    fn width(&self) -> T;
    fn height(&self) -> T;
}

impl SizeExt<f32> for Size {
    fn width(&self) -> f32 {
        self.X
    }
    fn height(&self) -> f32 {
        self.Y
    }
}

#[repr(C)]
pub struct Rect {
    pub point: Point,
    pub size: Size,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            point: Point { X: x, Y: y },
            size: Size {
                X: width,
                Y: height,
            },
        }
    }

    pub fn new_from_point_size(point: Point, size: Size) -> Self {
        Self { point, size }
    }
}

impl From<Rect> for D2D_RECT_F {
    fn from(r: Rect) -> Self {
        Self {
            left: r.point.X,
            top: r.point.Y,
            right: r.point.X + r.size.X,
            bottom: r.point.Y + r.size.Y,
        }
    }
}

impl From<D2D_RECT_F> for Rect {
    fn from(r: D2D_RECT_F) -> Self {
        Self {
            point: Point {
                X: r.left,
                Y: r.top,
            },
            size: Size {
                X: r.right - r.left,
                Y: r.bottom - r.top,
            },
        }
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
