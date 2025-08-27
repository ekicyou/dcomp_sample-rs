#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::ffi::c_void;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;

// Object-safe trait（winitスタイル）
pub trait WindowMessageHandler {
    fn hwnd(&self) -> HWND;
    fn set_hwnd(&mut self, hwnd: HWND);

    #[inline(always)]
    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let handled = match message {
            // ウィンドウライフサイクル
            WM_CREATE => self.WM_CREATE(wparam, lparam),
            WM_DESTROY => self.WM_DESTROY(wparam, lparam),
            WM_CLOSE => self.WM_CLOSE(wparam, lparam),

            // 描画関連
            WM_PAINT => self.WM_PAINT(wparam, lparam),
            WM_ERASEBKGND => self.WM_ERASEBKGND(wparam, lparam),
            WM_DISPLAYCHANGE => self.WM_DISPLAYCHANGE(wparam, lparam),
            WM_DPICHANGED => self.WM_DPICHANGED(wparam, lparam),
            WM_DPICHANGED_BEFOREPARENT => self.WM_DPICHANGED_BEFOREPARENT(wparam, lparam),
            WM_DPICHANGED_AFTERPARENT => self.WM_DPICHANGED_AFTERPARENT(wparam, lparam),

            // ウィンドウサイズ・位置
            WM_SIZE => self.WM_SIZE(wparam, lparam),
            WM_SIZING => self.WM_SIZING(wparam, lparam),
            WM_MOVE => self.WM_MOVE(wparam, lparam),
            WM_MOVING => self.WM_MOVING(wparam, lparam),
            WM_WINDOWPOSCHANGING => self.WM_WINDOWPOSCHANGING(wparam, lparam),
            WM_WINDOWPOSCHANGED => self.WM_WINDOWPOSCHANGED(wparam, lparam),
            WM_GETMINMAXINFO => self.WM_GETMINMAXINFO(wparam, lparam),

            // マウス入力
            WM_MOUSEMOVE => self.WM_MOUSEMOVE(wparam, lparam),
            WM_MOUSEWHEEL => self.WM_MOUSEWHEEL(wparam, lparam),
            WM_MOUSEHWHEEL => self.WM_MOUSEHWHEEL(wparam, lparam),
            WM_MOUSELEAVE => self.WM_MOUSELEAVE(wparam, lparam),
            WM_LBUTTONDOWN => self.WM_LBUTTONDOWN(wparam, lparam),
            WM_LBUTTONUP => self.WM_LBUTTONUP(wparam, lparam),
            WM_LBUTTONDBLCLK => self.WM_LBUTTONDBLCLK(wparam, lparam),
            WM_RBUTTONDOWN => self.WM_RBUTTONDOWN(wparam, lparam),
            WM_RBUTTONUP => self.WM_RBUTTONUP(wparam, lparam),
            WM_RBUTTONDBLCLK => self.WM_RBUTTONDBLCLK(wparam, lparam),
            WM_MBUTTONDOWN => self.WM_MBUTTONDOWN(wparam, lparam),
            WM_MBUTTONUP => self.WM_MBUTTONUP(wparam, lparam),
            WM_MBUTTONDBLCLK => self.WM_MBUTTONDBLCLK(wparam, lparam),
            WM_XBUTTONDOWN => self.WM_XBUTTONDOWN(wparam, lparam),
            WM_XBUTTONUP => self.WM_XBUTTONUP(wparam, lparam),
            WM_XBUTTONDBLCLK => self.WM_XBUTTONDBLCLK(wparam, lparam),

            // キーボード入力
            WM_KEYDOWN => self.WM_KEYDOWN(wparam, lparam),
            WM_KEYUP => self.WM_KEYUP(wparam, lparam),
            WM_SYSKEYDOWN => self.WM_SYSKEYDOWN(wparam, lparam),
            WM_SYSKEYUP => self.WM_SYSKEYUP(wparam, lparam),
            WM_CHAR => self.WM_CHAR(wparam, lparam),
            WM_SYSCHAR => self.WM_SYSCHAR(wparam, lparam),
            WM_DEADCHAR => self.WM_DEADCHAR(wparam, lparam),
            WM_SYSDEADCHAR => self.WM_SYSDEADCHAR(wparam, lparam),

            // フォーカス管理
            WM_SETFOCUS => self.WM_SETFOCUS(wparam, lparam),
            WM_KILLFOCUS => self.WM_KILLFOCUS(wparam, lparam),
            WM_ACTIVATE => self.WM_ACTIVATE(wparam, lparam),
            WM_ACTIVATEAPP => self.WM_ACTIVATEAPP(wparam, lparam),
            WM_CAPTURECHANGED => self.WM_CAPTURECHANGED(wparam, lparam),

            // システム・その他
            WM_TIMER => self.WM_TIMER(wparam, lparam),
            WM_COMMAND => self.WM_COMMAND(wparam, lparam),
            WM_SYSCOMMAND => self.WM_SYSCOMMAND(wparam, lparam),
            WM_MENUCHAR => self.WM_MENUCHAR(wparam, lparam),
            WM_ENTERMENULOOP => self.WM_ENTERMENULOOP(wparam, lparam),
            WM_EXITMENULOOP => self.WM_EXITMENULOOP(wparam, lparam),
            WM_THEMECHANGED => self.WM_THEMECHANGED(wparam, lparam),
            WM_DWMCOMPOSITIONCHANGED => self.WM_DWMCOMPOSITIONCHANGED(wparam, lparam),

            _ => None,
        };
        match handled {
            Some(res) => res,
            None => unsafe { DefWindowProcW(self.hwnd(), message, wparam, lparam) },
        }
    }

    // デフォルト実装
    fn WM_CREATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_DESTROY(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_CLOSE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_LBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_PAINT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_ERASEBKGND(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        Some(LRESULT(1)) // DirectCompositionで描画するため、背景消去をスキップ
    }
    fn WM_DISPLAYCHANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_DPICHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_DPICHANGED_BEFOREPARENT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_DPICHANGED_AFTERPARENT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_SIZE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_SIZING(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_MOVE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_MOVING(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_WINDOWPOSCHANGING(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_WINDOWPOSCHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_GETMINMAXINFO(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_LBUTTONDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_RBUTTONDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_RBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_MBUTTONDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_MBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_LBUTTONDBLCLK(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_RBUTTONDBLCLK(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_MBUTTONDBLCLK(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_MOUSEMOVE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_MOUSEWHEEL(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_MOUSEHWHEEL(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_MOUSELEAVE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_XBUTTONDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_XBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_XBUTTONDBLCLK(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_KEYDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_KEYUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_SYSKEYDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_SYSKEYUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_CHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_SYSCHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_DEADCHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_SYSDEADCHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_SETFOCUS(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_KILLFOCUS(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_ACTIVATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_ACTIVATEAPP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_TIMER(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_COMMAND(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_SYSCOMMAND(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_MENUCHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_ENTERMENULOOP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_EXITMENULOOP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_THEMECHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_DWMCOMPOSITIONCHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    fn WM_CAPTURECHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
}

pub(crate) trait WindowMessageHandlerExt: WindowMessageHandler {
    fn into_raw(self) -> *mut c_void
    where
        Self: Sized,
    {
        let b1: Box<dyn WindowMessageHandler> = Box::new(self);
        let b2 = Box::new(b1);
        let ptr = Box::into_raw(b2);
        let ptr = ptr as *mut c_void;
        ptr
    }
}

impl<T: WindowMessageHandler + Sized> WindowMessageHandlerExt for T {}

pub extern "system" fn wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match message {
            WM_NCCREATE => {
                let cs = lparam.0 as *const CREATESTRUCTW;
                if cs.is_null() {
                    return LRESULT(0);
                }
                let ptr = (*cs).lpCreateParams;
                let boxed_handler = ptr as *mut Box<dyn WindowMessageHandler>;
                if !boxed_handler.is_null() {
                    (*boxed_handler).set_hwnd(hwnd);
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, boxed_handler as _);
                }
                LRESULT(1)
            }
            WM_NCDESTROY => {
                let boxed_handler =
                    GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Box<dyn WindowMessageHandler>;
                if !boxed_handler.is_null() {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    let _box = Box::from_raw(boxed_handler);
                    LRESULT(1)
                } else {
                    DefWindowProcW(hwnd, message, wparam, lparam)
                }
            }
            _ => {
                let boxed_handler =
                    GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Box<dyn WindowMessageHandler>;
                if !boxed_handler.is_null() {
                    (*boxed_handler).message_handler(message, wparam, lparam)
                } else {
                    DefWindowProcW(hwnd, message, wparam, lparam)
                }
            }
        }
    }
}
