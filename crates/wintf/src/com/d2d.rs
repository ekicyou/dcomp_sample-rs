use std::mem::*;
use windows::core::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;
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
pub struct DrawGdiMetafile {
    pub gdimetafile: ManuallyDrop<ID2D1GdiMetafile>,
    pub destinationrectangle: Option<D2D_RECT_F>,
    pub sourcerectangle: Option<D2D_RECT_F>,
}

#[derive(Clone, Debug)]
pub struct SetPrimitiveBlend1(pub D2D1_PRIMITIVE_BLEND);

#[derive(Clone, Debug)]
pub struct SetAntialiasMode(pub D2D1_ANTIALIAS_MODE);

#[derive(Clone, Debug)]
pub struct SetTags {
    pub tag1: u64,
    pub tag2: u64,
}

#[derive(Clone, Debug)]
pub struct SetTextAntialiasMode(pub D2D1_TEXT_ANTIALIAS_MODE);

#[derive(Clone, Debug)]
pub struct SetTextRenderingParams {
    pub textrenderingparams: ManuallyDrop<IDWriteRenderingParams>,
}

#[derive(Clone, Debug)]
pub struct SetTransform(pub Matrix3x2);

#[derive(Clone, Debug)]
pub struct SetPrimitiveBlend(pub D2D1_PRIMITIVE_BLEND);

#[derive(Clone, Debug)]
pub struct SetUnitMode(pub D2D1_UNIT_MODE);

#[derive(Clone, Debug)]
pub struct Clear(pub Option<D2D1_COLOR_F>);

#[derive(Clone, Debug)]
pub struct DrawGlyphRun {
    pub baselineorigin: Vector2,
    // --- DWRITE_GLYPH_RUN START ---
    pub font_face: ManuallyDrop<IDWriteFontFace>,
    pub font_em_size: f32,
    pub glyph_indices: Vec<u16>,
    pub glyph_advances: Vec<f32>,
    pub glyph_offsets: Vec<DWRITE_GLYPH_OFFSET>,
    pub is_sideways: bool,
    pub bidi_level: u32,
    // --- DWRITE_GLYPH_RUN END ---
    pub glyphrundescription: Option<DWRITE_GLYPH_RUN_DESCRIPTION>,
    pub foregroundbrush: ManuallyDrop<ID2D1Brush>,
    pub measuringmode: DWRITE_MEASURING_MODE,
}

#[derive(Clone, Debug)]
pub struct DrawLine {
    pub point0: Vector2,
    pub point1: Vector2,
    pub brush: ManuallyDrop<ID2D1Brush>,
    pub strokewidth: f32,
    pub strokestyle: ManuallyDrop<ID2D1StrokeStyle>,
}

#[derive(Clone, Debug)]
pub struct DrawGeometry {
    pub geometry: ManuallyDrop<ID2D1Geometry>,
    pub brush: ManuallyDrop<ID2D1Brush>,
    pub strokewidth: f32,
    pub strokestyle: ManuallyDrop<ID2D1StrokeStyle>,
}

#[derive(Clone, Debug)]
pub struct DrawRectangle {
    pub rect: D2D_RECT_F,
    pub brush: ManuallyDrop<ID2D1Brush>,
    pub strokewidth: f32,
    pub strokestyle: ManuallyDrop<ID2D1StrokeStyle>,
}

#[derive(Clone, Debug)]
pub struct DrawBitmap {
    pub bitmap: ManuallyDrop<ID2D1Bitmap>,
    pub destinationrectangle: Option<D2D_RECT_F>,
    pub opacity: f32,
    pub interpolationmode: D2D1_INTERPOLATION_MODE,
    pub sourcerectangle: Option<D2D_RECT_F>,
    pub perspectivetransform: Option<Matrix4x4>,
}

#[derive(Clone, Debug)]
pub struct DrawImage {
    pub image: ManuallyDrop<ID2D1Image>,
    pub targetoffset: Option<Vector2>,
    pub imagerectangle: Option<D2D_RECT_F>,
    pub interpolationmode: D2D1_INTERPOLATION_MODE,
    pub compositemode: D2D1_COMPOSITE_MODE,
}

#[derive(Clone, Debug)]
pub struct DrawGdiMetafile2 {
    pub gdimetafile: ManuallyDrop<ID2D1GdiMetafile>,
    pub targetoffset: Option<Vector2>,
}

#[derive(Clone, Debug)]
pub struct FillMesh {
    pub mesh: ManuallyDrop<ID2D1Mesh>,
    pub brush: ManuallyDrop<ID2D1Brush>,
}

#[derive(Clone, Debug)]
pub struct FillOpacityMask {
    pub opacitymask: ManuallyDrop<ID2D1Bitmap>,
    pub brush: ManuallyDrop<ID2D1Brush>,
    pub destinationrectangle: Option<D2D_RECT_F>,
    pub sourcerectangle: Option<D2D_RECT_F>,
}

#[derive(Clone, Debug)]
pub struct FillGeometry {
    pub geometry: ManuallyDrop<ID2D1Geometry>,
    pub brush: ManuallyDrop<ID2D1Brush>,
    pub opacitybrush: ManuallyDrop<ID2D1Brush>,
}

#[derive(Clone, Debug)]
pub struct FillRectangle {
    pub rect: D2D_RECT_F,
    pub brush: ManuallyDrop<ID2D1Brush>,
}

#[derive(Clone, Debug)]
pub struct PushAxisAlignedClip {
    pub cliprect: D2D_RECT_F,
    pub antialiasmode: D2D1_ANTIALIAS_MODE,
}

#[derive(Clone, Debug)]
pub struct PushLayer {
    pub layerparameters1: D2D1_LAYER_PARAMETERS1,
    pub layer: ManuallyDrop<ID2D1Layer>,
}

#[derive(Clone, Debug)]
pub enum DrawCommand {
    // 描画コマンド（状態を変更しない）
    BlendImage(BlendImage),
    DrawSpriteBatch(DrawSpriteBatch),
    DrawInk(DrawInk),
    DrawGradientMesh(DrawGradientMesh),
    DrawGdiMetafile(DrawGdiMetafile),
    DrawGlyphRun(DrawGlyphRun),
    DrawLine(DrawLine),
    DrawGeometry(DrawGeometry),
    DrawRectangle(DrawRectangle),
    DrawBitmap(DrawBitmap),
    DrawImage(DrawImage),
    DrawGdiMetafile2(DrawGdiMetafile2),
    FillMesh(FillMesh),
    FillOpacityMask(FillOpacityMask),
    FillGeometry(FillGeometry),
    FillRectangle(FillRectangle),
    Clear(Clear),

    // 描画開始/終了
    BeginDraw,
    EndDraw,

    // 設定コマンド（状態を変更する）
    SetPrimitiveBlend2(SetPrimitiveBlend2),
    SetPrimitiveBlend1(SetPrimitiveBlend1),
    SetPrimitiveBlend(SetPrimitiveBlend),

    SetAntialiasMode(SetAntialiasMode),

    SetTags(SetTags),

    SetTextAntialiasMode(SetTextAntialiasMode),

    SetTextRenderingParams(SetTextRenderingParams),
    SetTransform(SetTransform),
    SetUnitMode(SetUnitMode),

    PushAxisAlignedClip(PushAxisAlignedClip),
    PopAxisAlignedClip,

    PushLayer(PushLayer),
    PopLayer,
}
