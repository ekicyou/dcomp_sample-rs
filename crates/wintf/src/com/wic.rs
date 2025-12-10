use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Imaging::{D2D::*, *};
use windows::Win32::System::Com::*;

pub fn wic_factory() -> Result<IWICImagingFactory2> {
    unsafe { CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER) }
}

pub trait WICImagingFactoryExt {
    /// CreateDecoderFromFilename
    fn create_decoder_from_filename<P0>(
        &self,
        wzfilename: P0,
        pguidvendor: Option<*const GUID>,
        dwdesiredaccess: GENERIC_ACCESS_RIGHTS,
        metadataoptions: WICDecodeOptions,
    ) -> Result<IWICBitmapDecoder>
    where
        P0: Param<PCWSTR>;

    /// CreateFormatConverter
    fn create_format_converter(&self) -> Result<IWICFormatConverter>;
}

impl WICImagingFactoryExt for IWICImagingFactory2 {
    #[inline(always)]
    fn create_decoder_from_filename<P0>(
        &self,
        wzfilename: P0,
        pguidvendor: Option<*const GUID>,
        dwdesiredaccess: GENERIC_ACCESS_RIGHTS,
        metadataoptions: WICDecodeOptions,
    ) -> Result<IWICBitmapDecoder>
    where
        P0: Param<PCWSTR>,
    {
        unsafe {
            self.CreateDecoderFromFilename(
                wzfilename,
                pguidvendor,
                dwdesiredaccess,
                metadataoptions,
            )
        }
    }

    #[inline(always)]
    fn create_format_converter(&self) -> Result<IWICFormatConverter> {
        unsafe { self.CreateFormatConverter() }
    }
}

pub trait WICBitmapDecoderExt {
    /// GetFrame
    fn frame(&self, index: u32) -> Result<IWICBitmapFrameDecode>;
}

impl WICBitmapDecoderExt for IWICBitmapDecoder {
    #[inline(always)]
    fn frame(&self, index: u32) -> Result<IWICBitmapFrameDecode> {
        unsafe { self.GetFrame(index) }
    }
}

pub trait WICFormatConverterExt {
    /// Initialize
    fn init(
        &self,
        pisource: &IWICBitmapSource,
        dstformat: *const GUID,
        dither: WICBitmapDitherType,
        pipalette: Option<&IWICPalette>,
        alpha_threshold_percent: f64,
        palette_translate: WICBitmapPaletteType,
    ) -> Result<()>;
}

impl WICFormatConverterExt for IWICFormatConverter {
    #[inline(always)]
    fn init(
        &self,
        pisource: &IWICBitmapSource,
        dstformat: *const GUID,
        dither: WICBitmapDitherType,
        pipalette: Option<&IWICPalette>,
        alpha_threshold_percent: f64,
        palette_translate: WICBitmapPaletteType,
    ) -> Result<()> {
        unsafe {
            self.Initialize(
                pisource,
                dstformat,
                dither,
                pipalette,
                alpha_threshold_percent,
                palette_translate,
            )
        }
    }
}

/// IWICBitmapSource 拡張トレイト
pub trait WICBitmapSourceExt {
    /// 画像サイズを取得
    fn get_size(&self) -> Result<(u32, u32)>;

    /// ピクセルデータをバッファにコピー
    ///
    /// # Arguments
    /// - `rect`: コピー対象の矩形（Noneで全体）
    /// - `stride`: 行あたりのバイト数
    /// - `buffer`: 出力バッファ
    fn copy_pixels(&self, rect: Option<&WICRect>, stride: u32, buffer: &mut [u8]) -> Result<()>;
}

impl WICBitmapSourceExt for IWICBitmapSource {
    #[inline(always)]
    fn get_size(&self) -> Result<(u32, u32)> {
        let mut width = 0u32;
        let mut height = 0u32;
        unsafe {
            self.GetSize(&mut width, &mut height)?;
        }
        Ok((width, height))
    }

    #[inline(always)]
    fn copy_pixels(&self, rect: Option<&WICRect>, stride: u32, buffer: &mut [u8]) -> Result<()> {
        let rect_ptr = rect
            .map(|r| r as *const WICRect)
            .unwrap_or(std::ptr::null());
        unsafe { self.CopyPixels(rect_ptr, stride, buffer) }
    }
}
