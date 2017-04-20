#![allow(dead_code)]

use winapi::shared::minwindef::UINT;


pub const DXGI_MWA_NO_WINDOW_CHANGES: UINT = (1 << 0);
pub const DXGI_MWA_NO_ALT_ENTER: UINT = (1 << 1);
pub const DXGI_MWA_NO_PRINT_SCREEN: UINT = (1 << 2);
pub const DXGI_MWA_VALID: UINT = (0x7);
