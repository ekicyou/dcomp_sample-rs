#![allow(non_snake_case)]
#![allow(unused_variables)]

use ambassador::*;
use euclid::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::*;

// Physical pixels (PPX)。デバイス依存ピクセル
pub struct Ppx;

// Device Independent Pixels (DIP)。論理ピクセル（96DPI）
pub struct Dip;

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
    fn to_physical(&self) -> Scale<f32, Dip, Ppx> {
        Scale::new(self.scale_factor())
    }

    #[inline]
    fn to_logical(&self) -> Scale<f32, Ppx, Dip> {
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

/// デバイス依存ピクセル長 (Physical Pixels)
pub type PpxLength = Length<f32, Ppx>;

/// デバイス依存座標 (Physical Pixels)
pub type PpxPoint = Point2D<f32, Ppx>;

/// デバイス依存サイズ (Physical Pixels)
pub type PpxSize = Size2D<f32, Ppx>;

/// デバイス依存矩形 (Physical Pixels)
pub type PpxRect = Rect<f32, Ppx>;

/// 96DPI（論理ピクセル）長 (Device Independent Pixels)
pub type DipLength = Length<f32, Dip>;

/// 96DPI（論理ピクセル）座標 (Device Independent Pixels)
pub type DipPoint = Point2D<f32, Dip>;

/// 96DPI（論理ピクセル）サイズ (Device Independent Pixels)
pub type DipSize = Size2D<f32, Dip>;

/// 96DPI（論理ピクセル）矩形 (Device Independent Pixels)
pub type DipRect = Rect<f32, Dip>;

pub type DipPoint3D = Point3D<f32, Dip>;
pub type DipVector2D = Vector2D<f32, Dip>;
pub type DipVector3D = Vector3D<f32, Dip>;

pub type DipTransform2D = Transform2D<f32, Dip, Dip>;
pub type DipTransform3D = Transform3D<f32, Dip, Dip>;

/// ４次数（3D回転）
pub type DipRotation3D = Rotation3D<f32, Dip, Dip>;

pub type RawLength = Length<i32, Ppx>;
pub type RawPoint = Point2D<i32, Ppx>;
pub type RawSize = Size2D<i32, Ppx>;
pub type RawRect = Rect<i32, Ppx>;

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

impl FromDpi<PpxLength> for DipLength {
    fn from_dpi(value: PpxLength, dpi: impl ScaleFactor) -> Self {
        value * dpi.to_logical()
    }
}
impl FromDpi<DipLength> for PpxLength {
    fn from_dpi(value: DipLength, dpi: impl ScaleFactor) -> Self {
        value * dpi.to_physical()
    }
}

impl FromDpi<PpxPoint> for DipPoint {
    fn from_dpi(value: PpxPoint, dpi: impl ScaleFactor) -> Self {
        value * dpi.to_logical()
    }
}
impl FromDpi<DipPoint> for PpxPoint {
    fn from_dpi(value: DipPoint, dpi: impl ScaleFactor) -> Self {
        value * dpi.to_physical()
    }
}

impl FromDpi<PpxSize> for DipSize {
    fn from_dpi(value: PpxSize, dpi: impl ScaleFactor) -> Self {
        value * dpi.to_logical()
    }
}
impl FromDpi<DipSize> for PpxSize {
    fn from_dpi(value: DipSize, dpi: impl ScaleFactor) -> Self {
        value * dpi.to_physical()
    }
}

impl FromDpi<PpxRect> for DipRect {
    fn from_dpi(value: PpxRect, dpi: impl ScaleFactor) -> Self {
        value * dpi.to_logical()
    }
}
impl FromDpi<DipRect> for PpxRect {
    fn from_dpi(value: DipRect, dpi: impl ScaleFactor) -> Self {
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

pub trait ToRaw<T> {
    fn into_raw(self) -> T;
}

impl ToRaw<RawLength> for PpxLength {
    fn into_raw(self) -> RawLength {
        RawLength::new(self.0.ceil() as i32)
    }
}

impl ToRaw<RawPoint> for PpxPoint {
    fn into_raw(self) -> RawPoint {
        RawPoint::new(self.x.ceil() as i32, self.y.ceil() as i32)
    }
}

impl ToRaw<RawSize> for PpxSize {
    fn into_raw(self) -> RawSize {
        RawSize::new(self.width.ceil() as i32, self.height.ceil() as i32)
    }
}

impl ToRaw<RawRect> for PpxRect {
    fn into_raw(self) -> RawRect {
        RawRect::new(self.origin.into_raw(), self.size.into_raw())
    }
}
