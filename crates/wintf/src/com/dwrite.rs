use windows::core::*;
use windows::Win32::Graphics::DirectWrite::*;

/// DWriteCreateFactory
pub fn dwrite_create_factory(factorytype: DWRITE_FACTORY_TYPE) -> Result<IDWriteFactory2> {
    unsafe { DWriteCreateFactory(factorytype) }
}

pub trait DWriteFactoryExt {
    /// CreateTextFormat
    fn create_text_format<P0, P1>(
        &self,
        fontfamilyname: P0,
        fontcollection: P1,
        fontweight: DWRITE_FONT_WEIGHT,
        fontstyle: DWRITE_FONT_STYLE,
        fontstretch: DWRITE_FONT_STRETCH,
        fontsize: f32,
        localename: P0,
    ) -> Result<IDWriteTextFormat>
    where
        P0: Param<PCWSTR>,
        P1: Param<IDWriteFontCollection>;

    /// CreateTextLayout
    fn create_text_layout<P0>(
        &self,
        text: P0,
        text_format: &IDWriteTextFormat,
        max_width: f32,
        max_height: f32,
    ) -> Result<IDWriteTextLayout>
    where
        P0: Param<PCWSTR>;
}

impl DWriteFactoryExt for IDWriteFactory2 {
    #[inline(always)]
    fn create_text_format<P0, P1>(
        &self,
        fontfamilyname: P0,
        fontcollection: P1,
        fontweight: DWRITE_FONT_WEIGHT,
        fontstyle: DWRITE_FONT_STYLE,
        fontstretch: DWRITE_FONT_STRETCH,
        fontsize: f32,
        localename: P0,
    ) -> Result<IDWriteTextFormat>
    where
        P0: Param<PCWSTR>,
        P1: Param<IDWriteFontCollection>,
    {
        unsafe {
            self.CreateTextFormat(
                fontfamilyname,
                fontcollection,
                fontweight,
                fontstyle,
                fontstretch,
                fontsize,
                localename,
            )
        }
    }

    fn create_text_layout<P0>(
        &self,
        text: P0,
        text_format: &IDWriteTextFormat,
        max_width: f32,
        max_height: f32,
    ) -> Result<IDWriteTextLayout>
    where
        P0: Param<PCWSTR>,
    {
        unsafe {
            let text_param = text.param();
            let text_pcwstr = text_param.abi();

            // Calculate string length and create slice
            if text_pcwstr.is_null() {
                // Empty string case
                let empty: &[u16] = &[];
                return self.CreateTextLayout(empty, text_format, max_width, max_height);
            }

            let mut len = 0;
            let mut ptr = text_pcwstr.0 as *const u16;
            while *ptr != 0 {
                len += 1;
                ptr = ptr.add(1);
            }

            // Create slice from raw pointer
            let text_slice = core::slice::from_raw_parts(text_pcwstr.0 as *const u16, len);

            self.CreateTextLayout(text_slice, text_format, max_width, max_height)
        }
    }
}

pub trait DWriteTextFormatExt {
    /// SetTextAlignment
    fn set_text_alignment(&self, textalignment: DWRITE_TEXT_ALIGNMENT) -> Result<()>;
    /// SetParagraphAlignment
    fn set_paragraph_alignment(&self, paragraphalignment: DWRITE_PARAGRAPH_ALIGNMENT)
        -> Result<()>;
}

impl DWriteTextFormatExt for IDWriteTextFormat {
    #[inline(always)]
    fn set_text_alignment(&self, textalignment: DWRITE_TEXT_ALIGNMENT) -> Result<()> {
        unsafe { self.SetTextAlignment(textalignment) }
    }

    #[inline(always)]
    fn set_paragraph_alignment(
        &self,
        paragraphalignment: DWRITE_PARAGRAPH_ALIGNMENT,
    ) -> Result<()> {
        unsafe { self.SetParagraphAlignment(paragraphalignment) }
    }
}
