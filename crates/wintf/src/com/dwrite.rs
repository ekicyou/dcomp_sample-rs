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
