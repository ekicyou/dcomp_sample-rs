#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(dead_code)]

use windows::core::*;
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WinStyle {
    pub(crate) style: WINDOW_STYLE,
    pub(crate) ex_style: WINDOW_EX_STYLE,
}

impl WinStyle {
    // new

    pub fn new(hwnd: HWND) -> Self {
        unsafe {
            let style = WINDOW_STYLE(GetWindowLongW(hwnd, GWL_STYLE) as _);
            let ex_style = WINDOW_EX_STYLE(GetWindowLongW(hwnd, GWL_EXSTYLE) as _);
            Self { style, ex_style }
        }
    }

    pub fn WS_OVERLAPPED() -> Self {
        with_style(WS_OVERLAPPED)
    }

    pub fn WS_CHILD() -> Self {
        with_style(WS_CHILD)
    }

    pub fn WS_POPUP() -> Self {
        with_style(WS_POPUP)
    }

    pub fn WS_OVERLAPPEDWINDOW() -> Self {
        with_style(WS_OVERLAPPEDWINDOW)
    }

    pub fn WS_POPUPWINDOW() -> Self {
        with_style(WS_POPUPWINDOW)
    }

    /// 値の取り出し
    pub fn get_style(&self) -> (WINDOW_STYLE, WINDOW_EX_STYLE) {
        (self.style, self.ex_style)
    }

    /// SetWindowLongWへ反映
    pub fn commit(&self, hwnd: HWND) -> Result<()> {
        unsafe {
            SetLastError(ERROR_SUCCESS);
            let res = SetWindowLongW(hwnd, GWL_STYLE, self.style.0 as _);
            let err = Error::from_win32();
            if res == 0 && err.code() != S_OK {
                return Err(err);
            }
            let res = SetWindowLongW(hwnd, GWL_EXSTYLE, self.ex_style.0 as _);
            let err = Error::from_win32();
            if res == 0 && err.code() != S_OK {
                return Err(err);
            }
            Ok(())
        }
    }

    // dwStyleの更新

    pub fn WS_MINIMIZE(self, flag: bool) -> Self {
        set_style(self, WS_MINIMIZE, flag)
    }

    pub fn WS_VISIBLE(self, flag: bool) -> Self {
        set_style(self, WS_VISIBLE, flag)
    }

    pub fn WS_DISABLED(self, flag: bool) -> Self {
        set_style(self, WS_DISABLED, flag)
    }

    pub fn WS_CLIPSIBLINGS(self, flag: bool) -> Self {
        set_style(self, WS_CLIPSIBLINGS, flag)
    }

    pub fn WS_CLIPCHILDREN(self, flag: bool) -> Self {
        set_style(self, WS_CLIPCHILDREN, flag)
    }

    pub fn WS_MAXIMIZE(self, flag: bool) -> Self {
        set_style(self, WS_MAXIMIZE, flag)
    }

    pub fn WS_BORDER(self, flag: bool) -> Self {
        set_style(self, WS_BORDER, flag)
    }

    pub fn WS_DLGFRAME(self, flag: bool) -> Self {
        set_style(self, WS_DLGFRAME, flag)
    }

    pub fn WS_VSCROLL(self, flag: bool) -> Self {
        set_style(self, WS_VSCROLL, flag)
    }

    pub fn WS_HSCROLL(self, flag: bool) -> Self {
        set_style(self, WS_HSCROLL, flag)
    }

    pub fn WS_SYSMENU(self, flag: bool) -> Self {
        set_style(self, WS_SYSMENU, flag)
    }

    pub fn WS_THICKFRAME(self, flag: bool) -> Self {
        set_style(self, WS_THICKFRAME, flag)
    }

    // dwExStyle：単一bitのスタイル

    pub fn WS_EX_DLGMODALFRAME(self, flag: bool) -> Self {
        set_ex(self, WS_EX_DLGMODALFRAME, flag)
    }

    pub fn WS_EX_NOPARENTNOTIFY(self, flag: bool) -> Self {
        set_ex(self, WS_EX_NOPARENTNOTIFY, flag)
    }

    pub fn WS_EX_TOPMOST(self, flag: bool) -> Self {
        set_ex(self, WS_EX_TOPMOST, flag)
    }

    pub fn WS_EX_ACCEPTFILES(self, flag: bool) -> Self {
        set_ex(self, WS_EX_ACCEPTFILES, flag)
    }

    pub fn WS_EX_TRANSPARENT(self, flag: bool) -> Self {
        set_ex(self, WS_EX_TRANSPARENT, flag)
    }

    pub fn WS_EX_MDICHILD(self, flag: bool) -> Self {
        set_ex(self, WS_EX_MDICHILD, flag)
    }

    pub fn WS_EX_TOOLWINDOW(self, flag: bool) -> Self {
        set_ex(self, WS_EX_TOOLWINDOW, flag)
    }

    pub fn WS_EX_WINDOWEDGE(self, flag: bool) -> Self {
        set_ex(self, WS_EX_WINDOWEDGE, flag)
    }

    pub fn WS_EX_CLIENTEDGE(self, flag: bool) -> Self {
        set_ex(self, WS_EX_CLIENTEDGE, flag)
    }

    pub fn WS_EX_CONTEXTHELP(self, flag: bool) -> Self {
        set_ex(self, WS_EX_CONTEXTHELP, flag)
    }

    pub fn WS_EX_CONTROLPARENT(self, flag: bool) -> Self {
        set_ex(self, WS_EX_CONTROLPARENT, flag)
    }

    pub fn WS_EX_STATICEDGE(self, flag: bool) -> Self {
        set_ex(self, WS_EX_STATICEDGE, flag)
    }

    pub fn WS_EX_APPWINDOW(self, flag: bool) -> Self {
        set_ex(self, WS_EX_APPWINDOW, flag)
    }

    pub fn WS_EX_LAYERED(self, flag: bool) -> Self {
        set_ex(self, WS_EX_LAYERED, flag)
    }

    pub fn WS_EX_NOINHERITLAYOUT(self, flag: bool) -> Self {
        set_ex(self, WS_EX_NOINHERITLAYOUT, flag)
    }

    pub fn WS_EX_NOACTIVATE(self, flag: bool) -> Self {
        set_ex(self, WS_EX_NOACTIVATE, flag)
    }

    pub fn WS_EX_COMPOSITED(self, flag: bool) -> Self {
        set_ex(self, WS_EX_COMPOSITED, flag)
    }

    pub fn WS_EX_NOREDIRECTIONBITMAP(self, flag: bool) -> Self {
        set_ex(self, WS_EX_NOREDIRECTIONBITMAP, flag)
    }

    // dwExStyle：複合スタイル

    pub fn WS_EX_OVERLAPPEDWINDOW(self) -> Self {
        set_ex(self, WS_EX_OVERLAPPEDWINDOW, true)
    }

    pub fn WS_EX_PALETTEWINDOW(self) -> Self {
        set_ex(self, WS_EX_PALETTEWINDOW, true)
    }

    // dwExStyle：排他スタイル

    pub fn WS_EX_LEFT(self) -> Self {
        set_ex(self, WS_EX_RIGHT, false)
    }
    pub fn WS_EX_RIGHT(self) -> Self {
        set_ex(self, WS_EX_RIGHT, true)
    }

    pub fn WS_EX_LTRREADING(self) -> Self {
        set_ex(self, WS_EX_RTLREADING, false)
    }
    pub fn WS_EX_RTLREADING(self) -> Self {
        set_ex(self, WS_EX_RTLREADING, true)
    }

    pub fn WS_EX_RIGHTSCROLLBAR(self) -> Self {
        set_ex(self, WS_EX_LEFTSCROLLBAR, false)
    }
    pub fn WS_EX_LEFTSCROLLBAR(self) -> Self {
        set_ex(self, WS_EX_LEFTSCROLLBAR, true)
    }

    pub fn WS_EX_LAYOUTLTR(self) -> Self {
        set_ex(self, WS_EX_LAYOUTRTL, false)
    }
    pub fn WS_EX_LAYOUTRTL(self) -> Self {
        set_ex(self, WS_EX_LAYOUTRTL, true)
    }
}

#[inline(always)]
fn with_style(style: WINDOW_STYLE) -> WinStyle {
    WinStyle {
        style,
        ..Default::default()
    }
}

#[inline(always)]
fn set_style(src: WinStyle, style: WINDOW_STYLE, flag: bool) -> WinStyle {
    let org = src.style.0;
    let style = style.0;
    let style = if flag { org | style } else { org & !style };
    let style = WINDOW_STYLE(style);
    WinStyle { style, ..src }
}

#[inline(always)]
fn set_ex(src: WinStyle, ex_style: WINDOW_EX_STYLE, flag: bool) -> WinStyle {
    let org = src.ex_style.0;
    let ex_style = ex_style.0;
    let ex_style = if flag {
        org | ex_style
    } else {
        org & !ex_style
    };
    let ex_style = WINDOW_EX_STYLE(ex_style);
    WinStyle { ex_style, ..src }
}

#[inline(always)]
fn set_ex2(src: WinStyle, on: WINDOW_EX_STYLE, off: WINDOW_EX_STYLE, flag: bool) -> WinStyle {
    let org = src.ex_style.0;
    let cleared = org & !(on.0 | off.0);
    let new_ex_style_value = if flag {
        cleared | on.0
    } else {
        cleared | off.0
    };
    let ex_style = WINDOW_EX_STYLE(new_ex_style_value);
    WinStyle { ex_style, ..src }
}
