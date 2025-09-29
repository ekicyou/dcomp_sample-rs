use std::mem::*;
use windows::core::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Dxgi::*;
use windows_numerics::*;

/// D2D1CreateDevice
pub fn d2d_create_device(dxgi: &IDXGIDevice4) -> Result<ID2D1Device> {
    unsafe { D2D1CreateDevice(dxgi, None) }
}

#[derive(Clone, Debug)]
pub struct BlendImage {
    pub image: ManuallyDrop<ID2D1Image>,
    pub blendmode: D2D1_BLEND_MODE,
    pub targetoffset: Option<Vector2>,
    pub imagerectangle: Option<D2D_RECT_F>,
    pub interpolationmode: D2D1_INTERPOLATION_MODE,
}

#[derive(Clone, Debug)]
pub struct SetPrimitiveBlend2(pub D2D1_PRIMITIVE_BLEND);

#[derive(Clone, Debug)]
pub struct DrawSpriteBatch {
    pub spritebatch: ManuallyDrop<ID2D1SpriteBatch>,
    pub startindex: u32,
    pub spritecount: u32,
    pub bitmap: ManuallyDrop<ID2D1Bitmap>,
    pub interpolationmode: D2D1_BITMAP_INTERPOLATION_MODE,
    pub spriteoptions: D2D1_SPRITE_OPTIONS,
}

#[derive(Clone, Debug)]
pub struct DrawInk {
    pub ink: ManuallyDrop<ID2D1Ink>,
    pub brush: ManuallyDrop<ID2D1Brush>,
    pub inkstyle: ManuallyDrop<ID2D1InkStyle>,
}

#[derive(Clone, Debug)]
pub struct DrawGradientMesh(pub ManuallyDrop<ID2D1GradientMesh>);




#[derive(Clone, Debug)]
pub unsafe fn DrawGdiMetafile<P0>(
    &self,
    gdimetafile: P0,
    destinationrectangle: Option<*const D2D_RECT_F>,
    sourcerectangle: Option<*const D2D_RECT_F>,
) -> Result<()>
where
    P0: Param<ID2D1GdiMetafile>,
Methods from Deref<Target = ID2D1CommandSink1>



#[derive(Clone, Debug)]
pub unsafe fn SetPrimitiveBlend1(
    &self,
    primitiveblend: D2D1_PRIMITIVE_BLEND,
) -> Result<()>
Methods from Deref<Target = ID2D1CommandSink>


#[derive(Clone, Debug)]
pub unsafe fn BeginDraw(&self) -> Result<()>


#[derive(Clone, Debug)]
pub unsafe fn EndDraw(&self) -> Result<()>


#[derive(Clone, Debug)]
pub unsafe fn SetAntialiasMode(
    &self,
    antialiasmode: D2D1_ANTIALIAS_MODE,
) -> Result<()>


#[derive(Clone, Debug)]
pub unsafe fn SetTags(&self, tag1: u64, tag2: u64) -> Result<()>


#[derive(Clone, Debug)]
pub unsafe fn SetTextAntialiasMode(
    &self,
    textantialiasmode: D2D1_TEXT_ANTIALIAS_MODE,
) -> Result<()>


#[derive(Clone, Debug)]
pub unsafe fn SetTextRenderingParams<P0>(
    &self,
    textrenderingparams: P0,
) -> Result<()>
where
    P0: Param<IDWriteRenderingParams>,


#[derive(Clone, Debug)]
    pub unsafe fn SetTransform(&self, transform: *const Matrix3x2) -> Result<()>


#[derive(Clone, Debug)]
    pub unsafe fn SetPrimitiveBlend(
    &self,
    primitiveblend: D2D1_PRIMITIVE_BLEND,
) -> Result<()>


#[derive(Clone, Debug)]
pub unsafe fn SetUnitMode(&self, unitmode: D2D1_UNIT_MODE) -> Result<()>


#[derive(Clone, Debug)]
pub unsafe fn Clear(&self, color: Option<*const D2D1_COLOR_F>) -> Result<()>


#[derive(Clone, Debug)]
pub unsafe fn DrawGlyphRun<P3>(
    &self,
    baselineorigin: D2D_POINT_2F,
    glyphrun: *const DWRITE_GLYPH_RUN,
    glyphrundescription: Option<*const DWRITE_GLYPH_RUN_DESCRIPTION>,
    foregroundbrush: P3,
    measuringmode: DWRITE_MEASURING_MODE,
) -> Result<()>
where
    P3: Param<ID2D1Brush>,


#[derive(Clone, Debug)]
    pub unsafe fn DrawLine<P2, P4>(
    &self,
    point0: D2D_POINT_2F,
    point1: D2D_POINT_2F,
    brush: P2,
    strokewidth: f32,
    strokestyle: P4,
) -> Result<()>
where
    P2: Param<ID2D1Brush>,
    P4: Param<ID2D1StrokeStyle>,


#[derive(Clone, Debug)]
    pub unsafe fn DrawGeometry<P0, P1, P3>(
    &self,
    geometry: P0,
    brush: P1,
    strokewidth: f32,
    strokestyle: P3,
) -> Result<()>
where
    P0: Param<ID2D1Geometry>,
    P1: Param<ID2D1Brush>,
    P3: Param<ID2D1StrokeStyle>,


#[derive(Clone, Debug)]
    pub unsafe fn DrawRectangle<P1, P3>(
    &self,
    rect: *const D2D_RECT_F,
    brush: P1,
    strokewidth: f32,
    strokestyle: P3,
) -> Result<()>
where
    P1: Param<ID2D1Brush>,
    P3: Param<ID2D1StrokeStyle>,


#[derive(Clone, Debug)]
    pub unsafe fn DrawBitmap<P0>(
    &self,
    bitmap: P0,
    destinationrectangle: Option<*const D2D_RECT_F>,
    opacity: f32,
    interpolationmode: D2D1_INTERPOLATION_MODE,
    sourcerectangle: Option<*const D2D_RECT_F>,
    perspectivetransform: Option<*const D2D_MATRIX_4X4_F>,
) -> Result<()>
where
    P0: Param<ID2D1Bitmap>,


#[derive(Clone, Debug)]
    pub unsafe fn DrawImage<P0>(
    &self,
    image: P0,
    targetoffset: Option<*const D2D_POINT_2F>,
    imagerectangle: Option<*const D2D_RECT_F>,
    interpolationmode: D2D1_INTERPOLATION_MODE,
    compositemode: D2D1_COMPOSITE_MODE,
) -> Result<()>
where
    P0: Param<ID2D1Image>,


#[derive(Clone, Debug)]
    pub unsafe fn DrawGdiMetafile<P0>(
    &self,
    gdimetafile: P0,
    targetoffset: Option<*const D2D_POINT_2F>,
) -> Result<()>
where
    P0: Param<ID2D1GdiMetafile>,


#[derive(Clone, Debug)]
    pub unsafe fn FillMesh<P0, P1>(&self, mesh: P0, brush: P1) -> Result<()>
where
    P0: Param<ID2D1Mesh>,
    P1: Param<ID2D1Brush>,


#[derive(Clone, Debug)]
    pub unsafe fn FillOpacityMask<P0, P1>(
    &self,
    opacitymask: P0,
    brush: P1,
    destinationrectangle: Option<*const D2D_RECT_F>,
    sourcerectangle: Option<*const D2D_RECT_F>,
) -> Result<()>
where
    P0: Param<ID2D1Bitmap>,
    P1: Param<ID2D1Brush>,


#[derive(Clone, Debug)]
    pub unsafe fn FillGeometry<P0, P1, P2>(
    &self,
    geometry: P0,
    brush: P1,
    opacitybrush: P2,
) -> Result<()>
where
    P0: Param<ID2D1Geometry>,
    P1: Param<ID2D1Brush>,
    P2: Param<ID2D1Brush>,


#[derive(Clone, Debug)]
    pub unsafe fn FillRectangle<P1>(
    &self,
    rect: *const D2D_RECT_F,
    brush: P1,
) -> Result<()>
where
    P1: Param<ID2D1Brush>,


#[derive(Clone, Debug)]
    pub unsafe fn PushAxisAlignedClip(
    &self,
    cliprect: *const D2D_RECT_F,
    antialiasmode: D2D1_ANTIALIAS_MODE,
) -> Result<()>


#[derive(Clone, Debug)]
pub unsafe fn PushLayer<P1>(
    &self,
    layerparameters1: *const D2D1_LAYER_PARAMETERS1,
    layer: P1,
) -> Result<()>
where
    P1: Param<ID2D1Layer>,


#[derive(Clone, Debug)]
    pub unsafe fn PopAxisAlignedClip(&self) -> Result<()>


#[derive(Clone, Debug)]
    pub unsafe fn PopLayer(&self) -> Result<()>