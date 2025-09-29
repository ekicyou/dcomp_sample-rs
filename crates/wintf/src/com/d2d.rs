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

#[inline(always)]
fn dup_com<T: Interface>(com_ref: &T) -> ManuallyDrop<T> {
    // SAFETY: This function is unsafe because it duplicates a COM pointer without incrementing its reference count.
    // The caller must ensure that the original COM object outlives the returned `ManuallyDrop<T>`.
    // The `ManuallyDrop` wrapper will prevent `Release` from being called when it's dropped.
    unsafe { ManuallyDrop::new(core::mem::transmute_copy(com_ref)) }
}

#[derive(Clone, Debug)]
pub struct BlendImage {
    pub image: ManuallyDrop<ID2D1Image>,
    pub blendmode: D2D1_BLEND_MODE,
    pub targetoffset: Option<Vector2>,
    pub imagerectangle: Option<D2D_RECT_F>,
    pub interpolationmode: D2D1_INTERPOLATION_MODE,
}

impl BlendImage {
    pub fn new(
        image: &ID2D1Image,
        blendmode: D2D1_BLEND_MODE,
        targetoffset: Option<Vector2>,
        imagerectangle: Option<D2D_RECT_F>,
        interpolationmode: D2D1_INTERPOLATION_MODE,
    ) -> Self {
        Self {
            image: dup_com(image),
            blendmode,
            targetoffset,
            imagerectangle,
            interpolationmode,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SetPrimitiveBlend2(pub D2D1_PRIMITIVE_BLEND);

impl SetPrimitiveBlend2 {
    pub fn new(primitive_blend: D2D1_PRIMITIVE_BLEND) -> Self {
        Self(primitive_blend)
    }
}

#[derive(Clone, Debug)]
pub struct DrawSpriteBatch {
    pub spritebatch: ManuallyDrop<ID2D1SpriteBatch>,
    pub startindex: u32,
    pub spritecount: u32,
    pub bitmap: ManuallyDrop<ID2D1Bitmap>,
    pub interpolationmode: D2D1_BITMAP_INTERPOLATION_MODE,
    pub spriteoptions: D2D1_SPRITE_OPTIONS,
}

impl DrawSpriteBatch {
    pub fn new(
        spritebatch: &ID2D1SpriteBatch,
        startindex: u32,
        spritecount: u32,
        bitmap: &ID2D1Bitmap,
        interpolationmode: D2D1_BITMAP_INTERPOLATION_MODE,
        spriteoptions: D2D1_SPRITE_OPTIONS,
    ) -> Self {
        Self {
            spritebatch: dup_com(spritebatch),
            startindex,
            spritecount,
            bitmap: dup_com(bitmap),
            interpolationmode,
            spriteoptions,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DrawInk {
    pub ink: ManuallyDrop<ID2D1Ink>,
    pub brush: ManuallyDrop<ID2D1Brush>,
    pub inkstyle: ManuallyDrop<ID2D1InkStyle>,
}

impl DrawInk {
    pub fn new(ink: &ID2D1Ink, brush: &ID2D1Brush, inkstyle: &ID2D1InkStyle) -> Self {
        Self {
            ink: dup_com(ink),
            brush: dup_com(brush),
            inkstyle: dup_com(inkstyle),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DrawGradientMesh(pub ManuallyDrop<ID2D1GradientMesh>);

impl DrawGradientMesh {
    pub fn new(gradient_mesh: &ID2D1GradientMesh) -> Self {
        Self(dup_com(gradient_mesh))
    }
}

#[derive(Clone, Debug)]
pub struct DrawGdiMetafile {
    pub gdimetafile: ManuallyDrop<ID2D1GdiMetafile>,
    pub destinationrectangle: Option<D2D_RECT_F>,
    pub sourcerectangle: Option<D2D_RECT_F>,
}

impl DrawGdiMetafile {
    pub fn new(
        gdimetafile: &ID2D1GdiMetafile,
        destinationrectangle: Option<D2D_RECT_F>,
        sourcerectangle: Option<D2D_RECT_F>,
    ) -> Self {
        Self {
            gdimetafile: dup_com(gdimetafile),
            destinationrectangle,
            sourcerectangle,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SetPrimitiveBlend1(pub D2D1_PRIMITIVE_BLEND);

impl SetPrimitiveBlend1 {
    pub fn new(primitive_blend: D2D1_PRIMITIVE_BLEND) -> Self {
        Self(primitive_blend)
    }
}

#[derive(Clone, Debug)]
pub struct SetAntialiasMode(pub D2D1_ANTIALIAS_MODE);

impl SetAntialiasMode {
    pub fn new(antialias_mode: D2D1_ANTIALIAS_MODE) -> Self {
        Self(antialias_mode)
    }
}

#[derive(Clone, Debug)]
pub struct SetTags {
    pub tag1: u64,
    pub tag2: u64,
}

impl SetTags {
    pub fn new(tag1: u64, tag2: u64) -> Self {
        Self { tag1, tag2 }
    }
}

#[derive(Clone, Debug)]
pub struct SetTextAntialiasMode(pub D2D1_TEXT_ANTIALIAS_MODE);

impl SetTextAntialiasMode {
    pub fn new(text_antialias_mode: D2D1_TEXT_ANTIALIAS_MODE) -> Self {
        Self(text_antialias_mode)
    }
}

#[derive(Clone, Debug)]
pub struct SetTextRenderingParams {
    pub textrenderingparams: ManuallyDrop<IDWriteRenderingParams>,
}

impl SetTextRenderingParams {
    pub fn new(textrenderingparams: &IDWriteRenderingParams) -> Self {
        Self {
            textrenderingparams: dup_com(textrenderingparams),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SetTransform(pub Matrix3x2);

impl SetTransform {
    pub fn new(transform: Matrix3x2) -> Self {
        Self(transform)
    }
}

#[derive(Clone, Debug)]
pub struct SetPrimitiveBlend(pub D2D1_PRIMITIVE_BLEND);

impl SetPrimitiveBlend {
    pub fn new(primitive_blend: D2D1_PRIMITIVE_BLEND) -> Self {
        Self(primitive_blend)
    }
}

#[derive(Clone, Debug)]
pub struct SetUnitMode(pub D2D1_UNIT_MODE);

impl SetUnitMode {
    pub fn new(unit_mode: D2D1_UNIT_MODE) -> Self {
        Self(unit_mode)
    }
}

#[derive(Clone, Debug)]
pub struct Clear(pub Option<D2D1_COLOR_F>);

impl Clear {
    pub fn new(color: Option<D2D1_COLOR_F>) -> Self {
        Self(color)
    }
}

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

impl DrawGlyphRun {
    pub fn new(
        baselineorigin: Vector2,
        font_face: &IDWriteFontFace,
        font_em_size: f32,
        glyph_indices: Vec<u16>,
        glyph_advances: Vec<f32>,
        glyph_offsets: Vec<DWRITE_GLYPH_OFFSET>,
        is_sideways: bool,
        bidi_level: u32,
        glyphrundescription: Option<DWRITE_GLYPH_RUN_DESCRIPTION>,
        foregroundbrush: &ID2D1Brush,
        measuringmode: DWRITE_MEASURING_MODE,
    ) -> Self {
        Self {
            baselineorigin,
            font_face: dup_com(font_face),
            font_em_size,
            glyph_indices,
            glyph_advances,
            glyph_offsets,
            is_sideways,
            bidi_level,
            glyphrundescription,
            foregroundbrush: dup_com(foregroundbrush),
            measuringmode,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DrawLine {
    pub point0: Vector2,
    pub point1: Vector2,
    pub brush: ManuallyDrop<ID2D1Brush>,
    pub strokewidth: f32,
    pub strokestyle: ManuallyDrop<ID2D1StrokeStyle>,
}

impl DrawLine {
    pub fn new(
        point0: Vector2,
        point1: Vector2,
        brush: &ID2D1Brush,
        strokewidth: f32,
        strokestyle: &ID2D1StrokeStyle,
    ) -> Self {
        Self {
            point0,
            point1,
            brush: dup_com(brush),
            strokewidth,
            strokestyle: dup_com(strokestyle),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DrawGeometry {
    pub geometry: ManuallyDrop<ID2D1Geometry>,
    pub brush: ManuallyDrop<ID2D1Brush>,
    pub strokewidth: f32,
    pub strokestyle: ManuallyDrop<ID2D1StrokeStyle>,
}

impl DrawGeometry {
    pub fn new(
        geometry: &ID2D1Geometry,
        brush: &ID2D1Brush,
        strokewidth: f32,
        strokestyle: &ID2D1StrokeStyle,
    ) -> Self {
        Self {
            geometry: dup_com(geometry),
            brush: dup_com(brush),
            strokewidth,
            strokestyle: dup_com(strokestyle),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DrawRectangle {
    pub rect: D2D_RECT_F,
    pub brush: ManuallyDrop<ID2D1Brush>,
    pub strokewidth: f32,
    pub strokestyle: ManuallyDrop<ID2D1StrokeStyle>,
}

impl DrawRectangle {
    pub fn new(
        rect: D2D_RECT_F,
        brush: &ID2D1Brush,
        strokewidth: f32,
        strokestyle: &ID2D1StrokeStyle,
    ) -> Self {
        Self {
            rect,
            brush: dup_com(brush),
            strokewidth,
            strokestyle: dup_com(strokestyle),
        }
    }
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

impl DrawBitmap {
    pub fn new(
        bitmap: &ID2D1Bitmap,
        destinationrectangle: Option<D2D_RECT_F>,
        opacity: f32,
        interpolationmode: D2D1_INTERPOLATION_MODE,
        sourcerectangle: Option<D2D_RECT_F>,
        perspectivetransform: Option<Matrix4x4>,
    ) -> Self {
        Self {
            bitmap: dup_com(bitmap),
            destinationrectangle,
            opacity,
            interpolationmode,
            sourcerectangle,
            perspectivetransform,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DrawImage {
    pub image: ManuallyDrop<ID2D1Image>,
    pub targetoffset: Option<Vector2>,
    pub imagerectangle: Option<D2D_RECT_F>,
    pub interpolationmode: D2D1_INTERPOLATION_MODE,
    pub compositemode: D2D1_COMPOSITE_MODE,
}

impl DrawImage {
    pub fn new(
        image: &ID2D1Image,
        targetoffset: Option<Vector2>,
        imagerectangle: Option<D2D_RECT_F>,
        interpolationmode: D2D1_INTERPOLATION_MODE,
        compositemode: D2D1_COMPOSITE_MODE,
    ) -> Self {
        Self {
            image: dup_com(image),
            targetoffset,
            imagerectangle,
            interpolationmode,
            compositemode,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DrawGdiMetafile2 {
    pub gdimetafile: ManuallyDrop<ID2D1GdiMetafile>,
    pub targetoffset: Option<Vector2>,
}

impl DrawGdiMetafile2 {
    pub fn new(gdimetafile: &ID2D1GdiMetafile, targetoffset: Option<Vector2>) -> Self {
        Self {
            gdimetafile: dup_com(gdimetafile),
            targetoffset,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FillMesh {
    pub mesh: ManuallyDrop<ID2D1Mesh>,
    pub brush: ManuallyDrop<ID2D1Brush>,
}

impl FillMesh {
    pub fn new(mesh: &ID2D1Mesh, brush: &ID2D1Brush) -> Self {
        Self {
            mesh: dup_com(mesh),
            brush: dup_com(brush),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FillOpacityMask {
    pub opacitymask: ManuallyDrop<ID2D1Bitmap>,
    pub brush: ManuallyDrop<ID2D1Brush>,
    pub destinationrectangle: Option<D2D_RECT_F>,
    pub sourcerectangle: Option<D2D_RECT_F>,
}

impl FillOpacityMask {
    pub fn new(
        opacitymask: &ID2D1Bitmap,
        brush: &ID2D1Brush,
        destinationrectangle: Option<D2D_RECT_F>,
        sourcerectangle: Option<D2D_RECT_F>,
    ) -> Self {
        Self {
            opacitymask: dup_com(opacitymask),
            brush: dup_com(brush),
            destinationrectangle,
            sourcerectangle,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FillGeometry {
    pub geometry: ManuallyDrop<ID2D1Geometry>,
    pub brush: ManuallyDrop<ID2D1Brush>,
    pub opacitybrush: ManuallyDrop<ID2D1Brush>,
}

impl FillGeometry {
    pub fn new(geometry: &ID2D1Geometry, brush: &ID2D1Brush, opacitybrush: &ID2D1Brush) -> Self {
        Self {
            geometry: dup_com(geometry),
            brush: dup_com(brush),
            opacitybrush: dup_com(opacitybrush),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FillRectangle {
    pub rect: D2D_RECT_F,
    pub brush: ManuallyDrop<ID2D1Brush>,
}

impl FillRectangle {
    pub fn new(rect: D2D_RECT_F, brush: &ID2D1Brush) -> Self {
        Self {
            rect,
            brush: dup_com(brush),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PushAxisAlignedClip {
    pub cliprect: D2D_RECT_F,
    pub antialiasmode: D2D1_ANTIALIAS_MODE,
}

impl PushAxisAlignedClip {
    pub fn new(cliprect: D2D_RECT_F, antialiasmode: D2D1_ANTIALIAS_MODE) -> Self {
        Self {
            cliprect,
            antialiasmode,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PushLayer {
    pub layerparameters1: D2D1_LAYER_PARAMETERS1,
    pub layer: ManuallyDrop<ID2D1Layer>,
}

impl PushLayer {
    pub fn new(layerparameters1: D2D1_LAYER_PARAMETERS1, layer: &ID2D1Layer) -> Self {
        Self {
            layerparameters1,
            layer: dup_com(layer),
        }
    }
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
