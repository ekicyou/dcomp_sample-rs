#![allow(unused_variables)]
use std::cell::RefCell;
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
    pub inkstyle: Option<ManuallyDrop<ID2D1InkStyle>>,
}

impl DrawInk {
    pub fn new(ink: &ID2D1Ink, brush: &ID2D1Brush, inkstyle: Option<&ID2D1InkStyle>) -> Self {
        Self {
            ink: dup_com(ink),
            brush: dup_com(brush),
            inkstyle: inkstyle.map(dup_com),
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
    pub textrenderingparams: Option<ManuallyDrop<IDWriteRenderingParams>>,
}

impl SetTextRenderingParams {
    pub fn new(textrenderingparams: Option<&IDWriteRenderingParams>) -> Self {
        Self {
            textrenderingparams: textrenderingparams.map(dup_com),
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
    pub strokestyle: Option<ManuallyDrop<ID2D1StrokeStyle>>,
}

impl DrawLine {
    pub fn new(
        point0: Vector2,
        point1: Vector2,
        brush: &ID2D1Brush,
        strokewidth: f32,
        strokestyle: Option<&ID2D1StrokeStyle>,
    ) -> Self {
        Self {
            point0,
            point1,
            brush: dup_com(brush),
            strokewidth,
            strokestyle: strokestyle.map(dup_com),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DrawGeometry {
    pub geometry: ManuallyDrop<ID2D1Geometry>,
    pub brush: ManuallyDrop<ID2D1Brush>,
    pub strokewidth: f32,
    pub strokestyle: Option<ManuallyDrop<ID2D1StrokeStyle>>,
}

impl DrawGeometry {
    pub fn new(
        geometry: &ID2D1Geometry,
        brush: &ID2D1Brush,
        strokewidth: f32,
        strokestyle: Option<&ID2D1StrokeStyle>,
    ) -> Self {
        Self {
            geometry: dup_com(geometry),
            brush: dup_com(brush),
            strokewidth,
            strokestyle: strokestyle.map(dup_com),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DrawRectangle {
    pub rect: D2D_RECT_F,
    pub brush: ManuallyDrop<ID2D1Brush>,
    pub strokewidth: f32,
    pub strokestyle: Option<ManuallyDrop<ID2D1StrokeStyle>>,
}

impl DrawRectangle {
    pub fn new(
        rect: D2D_RECT_F,
        brush: &ID2D1Brush,
        strokewidth: f32,
        strokestyle: Option<&ID2D1StrokeStyle>,
    ) -> Self {
        Self {
            rect,
            brush: dup_com(brush),
            strokewidth,
            strokestyle: strokestyle.map(dup_com),
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
    pub opacitybrush: Option<ManuallyDrop<ID2D1Brush>>,
}

impl FillGeometry {
    pub fn new(
        geometry: &ID2D1Geometry,
        brush: &ID2D1Brush,
        opacitybrush: Option<&ID2D1Brush>,
    ) -> Self {
        Self {
            geometry: dup_com(geometry),
            brush: dup_com(brush),
            opacitybrush: opacitybrush.map(dup_com),
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
    // --- D2D1_LAYER_PARAMETERS1 START ---
    pub content_bounds: D2D_RECT_F,
    pub geometric_mask: Option<ManuallyDrop<ID2D1Geometry>>,
    pub mask_antialias_mode: D2D1_ANTIALIAS_MODE,
    pub mask_transform: Matrix3x2,
    pub opacity: f32,
    pub opacity_brush: Option<ManuallyDrop<ID2D1Brush>>,
    pub layer_options: D2D1_LAYER_OPTIONS1,
    // --- D2D1_LAYER_PARAMETERS1 END ---
    pub layer: Option<ManuallyDrop<ID2D1Layer>>,
}

impl PushLayer {
    pub fn new(
        content_bounds: D2D_RECT_F,
        geometric_mask: Option<&ID2D1Geometry>,
        mask_antialias_mode: D2D1_ANTIALIAS_MODE,
        mask_transform: Matrix3x2,
        opacity: f32,
        opacity_brush: Option<&ID2D1Brush>,
        layer_options: D2D1_LAYER_OPTIONS1,
        layer: Option<&ID2D1Layer>,
    ) -> Self {
        Self {
            content_bounds,
            geometric_mask: geometric_mask.map(dup_com),
            mask_antialias_mode,
            mask_transform,
            opacity,
            opacity_brush: opacity_brush.map(dup_com),
            layer_options,
            layer: layer.map(dup_com),
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

#[implement(ID2D1CommandSink5)]
#[derive(Clone, Debug)]
pub struct RecCommandSink {
    commands: RefCell<Vec<DrawCommand>>,
}

impl RecCommandSink {
    pub fn new() -> Self {
        Self {
            commands: RefCell::new(Vec::new()),
        }
    }

    pub fn clear(&mut self) {
        self.commands.borrow_mut().clear();
    }

    pub fn push(&self, command: DrawCommand) {
        self.commands.borrow_mut().push(command);
    }

    pub fn commands(&self) -> &Vec<DrawCommand> {
        // This is tricky because we can't return a reference to the inside of a RefCell.
        // For now, let's assume the caller will handle this. A better API might be needed.
        // A quick fix is to make this method unsafe or change its signature.
        // Let's try to make it work by returning a Ref<Vec<...>>
        // But the original signature returns &Vec.
        // Let's comment it out for now and see if it's used.
        // &self.commands
        todo!()
    }
}

impl ID2D1CommandSink_Impl for RecCommandSink_Impl {
    fn BeginDraw(&self) -> Result<()> {
        self.push(DrawCommand::BeginDraw);
        Ok(())
    }

    fn EndDraw(&self) -> Result<()> {
        self.push(DrawCommand::EndDraw);
        Ok(())
    }

    fn SetAntialiasMode(&self, antialiasmode: D2D1_ANTIALIAS_MODE) -> Result<()> {
        self.push(DrawCommand::SetAntialiasMode(SetAntialiasMode::new(
            antialiasmode,
        )));
        Ok(())
    }
    fn SetTags(&self, tag1: u64, tag2: u64) -> Result<()> {
        self.push(DrawCommand::SetTags(SetTags::new(tag1, tag2)));
        Ok(())
    }
    fn SetTextAntialiasMode(&self, textantialiasmode: D2D1_TEXT_ANTIALIAS_MODE) -> Result<()> {
        self.push(DrawCommand::SetTextAntialiasMode(
            SetTextAntialiasMode::new(textantialiasmode),
        ));
        Ok(())
    }

    fn SetTextRenderingParams(
        &self,
        textrenderingparams: Ref<IDWriteRenderingParams>,
    ) -> Result<()> {
        self.push(DrawCommand::SetTextRenderingParams(
            SetTextRenderingParams::new(textrenderingparams.as_ref()),
        ));
        Ok(())
    }

    fn SetTransform(&self, transform: *const Matrix3x2) -> Result<()> {
        let transform = unsafe { *transform };
        self.push(DrawCommand::SetTransform(SetTransform::new(transform)));
        Ok(())
    }

    fn SetPrimitiveBlend(&self, primitiveblend: D2D1_PRIMITIVE_BLEND) -> Result<()> {
        self.push(DrawCommand::SetPrimitiveBlend(SetPrimitiveBlend::new(
            primitiveblend,
        )));
        Ok(())
    }

    fn SetUnitMode(&self, unitmode: D2D1_UNIT_MODE) -> Result<()> {
        self.push(DrawCommand::SetUnitMode(SetUnitMode::new(unitmode)));
        Ok(())
    }

    fn Clear(&self, color: *const D2D1_COLOR_F) -> Result<()> {
        let color = unsafe { color.as_ref() }.cloned();
        self.push(DrawCommand::Clear(Clear::new(color)));
        Ok(())
    }

    fn DrawGlyphRun(
        &self,
        baselineorigin: &Vector2,
        glyphrun: *const DWRITE_GLYPH_RUN,
        glyphrundescription: *const DWRITE_GLYPH_RUN_DESCRIPTION,
        foregroundbrush: Ref<ID2D1Brush>,
        measuringmode: DWRITE_MEASURING_MODE,
    ) -> Result<()> {
        unsafe {
            let glyph_run = &*glyphrun;
            let glyph_indices =
                std::slice::from_raw_parts(glyph_run.glyphIndices, glyph_run.glyphCount as usize)
                    .to_vec();
            let glyph_advances = if glyph_run.glyphAdvances.is_null() {
                Vec::new()
            } else {
                std::slice::from_raw_parts(glyph_run.glyphAdvances, glyph_run.glyphCount as usize)
                    .to_vec()
            };
            let glyph_offsets = if glyph_run.glyphOffsets.is_null() {
                Vec::new()
            } else {
                std::slice::from_raw_parts(glyph_run.glyphOffsets, glyph_run.glyphCount as usize)
                    .to_vec()
            };

            self.push(DrawCommand::DrawGlyphRun(DrawGlyphRun::new(
                *baselineorigin,
                glyph_run.fontFace.as_ref().unwrap(),
                glyph_run.fontEmSize,
                glyph_indices,
                glyph_advances,
                glyph_offsets,
                glyph_run.isSideways.as_bool(),
                glyph_run.bidiLevel,
                glyphrundescription.as_ref().cloned(),
                foregroundbrush.as_ref().unwrap(),
                measuringmode,
            )));
        }
        Ok(())
    }

    fn DrawLine(
        &self,
        point0: &Vector2,
        point1: &Vector2,
        brush: Ref<ID2D1Brush>,
        strokewidth: f32,
        strokestyle: Ref<ID2D1StrokeStyle>,
    ) -> Result<()> {
        self.push(DrawCommand::DrawLine(DrawLine::new(
            *point0,
            *point1,
            brush.as_ref().unwrap(),
            strokewidth,
            strokestyle.as_ref(),
        )));
        Ok(())
    }

    fn DrawGeometry(
        &self,
        geometry: Ref<ID2D1Geometry>,
        brush: Ref<ID2D1Brush>,
        strokewidth: f32,
        strokestyle: Ref<ID2D1StrokeStyle>,
    ) -> Result<()> {
        self.push(DrawCommand::DrawGeometry(DrawGeometry::new(
            geometry.as_ref().unwrap(),
            brush.as_ref().unwrap(),
            strokewidth,
            strokestyle.as_ref(),
        )));
        Ok(())
    }

    fn DrawRectangle(
        &self,
        rect: *const D2D_RECT_F,
        brush: Ref<ID2D1Brush>,
        strokewidth: f32,
        strokestyle: Ref<ID2D1StrokeStyle>,
    ) -> Result<()> {
        let rect = unsafe { *rect };
        self.push(DrawCommand::DrawRectangle(DrawRectangle::new(
            rect,
            brush.as_ref().unwrap(),
            strokewidth,
            strokestyle.as_ref(),
        )));
        Ok(())
    }

    fn DrawBitmap(
        &self,
        bitmap: Ref<ID2D1Bitmap>,
        destinationrectangle: *const D2D_RECT_F,
        opacity: f32,
        interpolationmode: D2D1_INTERPOLATION_MODE,
        sourcerectangle: *const D2D_RECT_F,
        perspectivetransform: *const Matrix4x4,
    ) -> Result<()> {
        self.push(DrawCommand::DrawBitmap(DrawBitmap::new(
            bitmap.as_ref().unwrap(),
            unsafe { destinationrectangle.as_ref() }.cloned(),
            opacity,
            interpolationmode,
            unsafe { sourcerectangle.as_ref() }.cloned(),
            unsafe { perspectivetransform.as_ref() }.cloned(),
        )));
        Ok(())
    }

    fn DrawImage(
        &self,
        image: Ref<ID2D1Image>,
        targetoffset: *const Vector2,
        imagerectangle: *const D2D_RECT_F,
        interpolationmode: D2D1_INTERPOLATION_MODE,
        compositemode: D2D1_COMPOSITE_MODE,
    ) -> Result<()> {
        self.push(DrawCommand::DrawImage(DrawImage::new(
            image.as_ref().unwrap(),
            unsafe { targetoffset.as_ref() }.cloned(),
            unsafe { imagerectangle.as_ref() }.cloned(),
            interpolationmode,
            compositemode,
        )));
        Ok(())
    }

    fn DrawGdiMetafile(
        &self,
        gdimetafile: Ref<ID2D1GdiMetafile>,
        targetoffset: *const Vector2,
    ) -> Result<()> {
        self.push(DrawCommand::DrawGdiMetafile2(DrawGdiMetafile2::new(
            gdimetafile.as_ref().unwrap(),
            unsafe { targetoffset.as_ref() }.cloned(),
        )));
        Ok(())
    }

    fn FillMesh(&self, mesh: Ref<ID2D1Mesh>, brush: Ref<ID2D1Brush>) -> Result<()> {
        self.push(DrawCommand::FillMesh(FillMesh::new(
            mesh.as_ref().unwrap(),
            brush.as_ref().unwrap(),
        )));
        Ok(())
    }

    fn FillOpacityMask(
        &self,
        opacitymask: Ref<ID2D1Bitmap>,
        brush: Ref<ID2D1Brush>,
        destinationrectangle: *const D2D_RECT_F,
        sourcerectangle: *const D2D_RECT_F,
    ) -> Result<()> {
        self.push(DrawCommand::FillOpacityMask(FillOpacityMask::new(
            opacitymask.as_ref().unwrap(),
            brush.as_ref().unwrap(),
            unsafe { destinationrectangle.as_ref() }.cloned(),
            unsafe { sourcerectangle.as_ref() }.cloned(),
        )));
        Ok(())
    }

    fn FillGeometry(
        &self,
        geometry: Ref<ID2D1Geometry>,
        brush: Ref<ID2D1Brush>,
        opacitybrush: Ref<ID2D1Brush>,
    ) -> Result<()> {
        self.push(DrawCommand::FillGeometry(FillGeometry::new(
            geometry.as_ref().unwrap(),
            brush.as_ref().unwrap(),
            opacitybrush.as_ref(),
        )));
        Ok(())
    }

    fn FillRectangle(&self, rect: *const D2D_RECT_F, brush: Ref<ID2D1Brush>) -> Result<()> {
        let rect = unsafe { *rect };
        self.push(DrawCommand::FillRectangle(FillRectangle::new(
            rect,
            brush.as_ref().unwrap(),
        )));
        Ok(())
    }

    fn PushAxisAlignedClip(
        &self,
        cliprect: *const D2D_RECT_F,
        antialiasmode: D2D1_ANTIALIAS_MODE,
    ) -> Result<()> {
        let cliprect = unsafe { *cliprect };
        self.push(DrawCommand::PushAxisAlignedClip(PushAxisAlignedClip::new(
            cliprect,
            antialiasmode,
        )));
        Ok(())
    }

    fn PushLayer(
        &self,
        layerparameters1: *const D2D1_LAYER_PARAMETERS1,
        layer: Ref<ID2D1Layer>,
    ) -> Result<()> {
        let params = unsafe { &*layerparameters1 };
        self.push(DrawCommand::PushLayer(PushLayer::new(
            params.contentBounds,
            params.geometricMask.as_ref(),
            params.maskAntialiasMode,
            params.maskTransform,
            params.opacity,
            params.opacityBrush.as_ref(),
            params.layerOptions,
            layer.as_ref(),
        )));
        Ok(())
    }

    fn PopAxisAlignedClip(&self) -> Result<()> {
        self.push(DrawCommand::PopAxisAlignedClip);
        Ok(())
    }

    fn PopLayer(&self) -> Result<()> {
        self.push(DrawCommand::PopLayer);
        Ok(())
    }
}

impl ID2D1CommandSink1_Impl for RecCommandSink_Impl {
    fn SetPrimitiveBlend1(&self, primitiveblend: D2D1_PRIMITIVE_BLEND) -> Result<()> {
        self.push(DrawCommand::SetPrimitiveBlend1(SetPrimitiveBlend1::new(
            primitiveblend,
        )));
        Ok(())
    }
}

impl ID2D1CommandSink2_Impl for RecCommandSink_Impl {
    fn DrawInk(
        &self,
        ink: Ref<ID2D1Ink>,
        brush: Ref<ID2D1Brush>,
        inkstyle: Ref<ID2D1InkStyle>,
    ) -> Result<()> {
        self.push(DrawCommand::DrawInk(DrawInk::new(
            ink.as_ref().unwrap(),
            brush.as_ref().unwrap(),
            inkstyle.as_ref(),
        )));
        Ok(())
    }
    fn DrawGradientMesh(&self, gradientmesh: Ref<ID2D1GradientMesh>) -> Result<()> {
        self.push(DrawCommand::DrawGradientMesh(DrawGradientMesh::new(
            gradientmesh.as_ref().unwrap(),
        )));
        Ok(())
    }
    fn DrawGdiMetafile(
        &self,
        gdimetafile: Ref<ID2D1GdiMetafile>,
        destinationrectangle: *const D2D_RECT_F,
        sourcerectangle: *const D2D_RECT_F,
    ) -> Result<()> {
        self.push(DrawCommand::DrawGdiMetafile(DrawGdiMetafile::new(
            gdimetafile.as_ref().unwrap(),
            unsafe { destinationrectangle.as_ref() }.cloned(),
            unsafe { sourcerectangle.as_ref() }.cloned(),
        )));
        Ok(())
    }
}

impl ID2D1CommandSink3_Impl for RecCommandSink_Impl {
    fn DrawSpriteBatch(
        &self,
        spritebatch: Ref<ID2D1SpriteBatch>,
        startindex: u32,
        spritecount: u32,
        bitmap: Ref<ID2D1Bitmap>,
        interpolationmode: D2D1_BITMAP_INTERPOLATION_MODE,
        spriteoptions: D2D1_SPRITE_OPTIONS,
    ) -> Result<()> {
        self.push(DrawCommand::DrawSpriteBatch(DrawSpriteBatch::new(
            spritebatch.as_ref().unwrap(),
            startindex,
            spritecount,
            bitmap.as_ref().unwrap(),
            interpolationmode,
            spriteoptions,
        )));
        Ok(())
    }
}

impl ID2D1CommandSink4_Impl for RecCommandSink_Impl {
    fn SetPrimitiveBlend2(&self, primitiveblend: D2D1_PRIMITIVE_BLEND) -> Result<()> {
        self.push(DrawCommand::SetPrimitiveBlend2(SetPrimitiveBlend2::new(
            primitiveblend,
        )));
        Ok(())
    }
}

impl ID2D1CommandSink5_Impl for RecCommandSink_Impl {
    fn BlendImage(
        &self,
        image: Ref<ID2D1Image>,
        blendmode: D2D1_BLEND_MODE,
        targetoffset: *const Vector2,
        imagerectangle: *const D2D_RECT_F,
        interpolationmode: D2D1_INTERPOLATION_MODE,
    ) -> Result<()> {
        self.push(DrawCommand::BlendImage(BlendImage::new(
            image.as_ref().unwrap(),
            blendmode,
            unsafe { targetoffset.as_ref() }.cloned(),
            unsafe { imagerectangle.as_ref() }.cloned(),
            interpolationmode,
        )));
        Ok(())
    }
}
