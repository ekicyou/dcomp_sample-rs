pub const FRAME_COUNT: u32 = 2;
pub const CIRCLE_SEGMENTS: u32 = 64;
pub const TEXTURE_WIDTH: u64 = 256;
pub const TEXTURE_HEIGHT: u32 = 256;

pub mod t {
    use std;
    use std::ffi::CStr;
    lazy_static! {
        pub static ref POSITION: &'static CStr = c_str!("POSITION");
        pub static ref TEXCOORD: &'static CStr = c_str!("TEXCOORD");
    }
}
