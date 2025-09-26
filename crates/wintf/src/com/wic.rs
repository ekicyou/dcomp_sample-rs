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
