pub const CIRCLE_SEGMENTS: i32 = 64;
pub const NUM_TEXTURE_COLORS: u32 = 8;
pub const NUM_ALPHA_SHADES: u32 = 256;
pub const FRAME_COUNT: u32 = 2;
pub const TEXTURE_WIDTH: u64 = 256;
pub const TEXTURE_HEIGHT: u32 = 256;
pub const TEXTURE_PIXEL_SIZE_X: u32 = (TEXTURE_WIDTH as u32) / NUM_ALPHA_SHADES;
pub const TEXTURE_PIXEL_SIZE_Y: u32 = TEXTURE_HEIGHT / NUM_TEXTURE_COLORS;

pub mod t {
    use std;
    use std::ffi::CStr;
    lazy_static! {
        pub static ref POSITION: &'static CStr = c_str!("POSITION");
        pub static ref TEXCOORD: &'static CStr = c_str!("TEXCOORD");
    }
}
