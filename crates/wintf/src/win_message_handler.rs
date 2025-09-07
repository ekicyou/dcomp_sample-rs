#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::ffi::c_void;
use std::rc::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub trait BaseWindowMessageHandler {
    fn message_handler(
        &mut self,
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT;
}

pub trait WindowMessageHandler: BaseWindowMessageHandler {
    fn hwnd(&self) -> HWND;
    fn set_hwnd(&mut self, hwnd: HWND);

    fn mouse_tracking(&self) -> bool {
        true
    }
    fn set_mouse_tracking(&mut self, tracking: bool) {}

    /// 生のメッセージハンドラ
    #[inline(always)]
    fn raw_message_handler(
        &mut self,
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT> {
        None
    }

    // デフォルト実装
    #[inline(always)]
    fn WM_NCCREATE(&mut self, hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        self.set_hwnd(hwnd);
        None
    }

    #[inline(always)]
    fn WM_CREATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DESTROY(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CLOSE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_LBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_PAINT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_ERASEBKGND(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        Some(LRESULT(1)) // DirectCompositionで描画するため、背景消去をスキップ
    }
    #[inline(always)]
    fn WM_DISPLAYCHANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DPICHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DPICHANGED_BEFOREPARENT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DPICHANGED_AFTERPARENT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCCALCSIZE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SIZE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SIZING(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MOVE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MOVING(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_WINDOWPOSCHANGING(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_WINDOWPOSCHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_GETMINMAXINFO(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_LBUTTONDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_RBUTTONDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_RBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MBUTTONDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_LBUTTONDBLCLK(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_RBUTTONDBLCLK(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MBUTTONDBLCLK(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MOUSEMOVE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MOUSEENTER(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MOUSELEAVE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MOUSEWHEEL(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MOUSEHWHEEL(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_XBUTTONDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_XBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_XBUTTONDBLCLK(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_KEYDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_KEYUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SYSKEYDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SYSKEYUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SYSCHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DEADCHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SYSDEADCHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SETFOCUS(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_KILLFOCUS(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_ACTIVATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_ACTIVATEAPP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_TIMER(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_COMMAND(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SYSCOMMAND(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MENUCHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_ENTERMENULOOP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_EXITMENULOOP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_THEMECHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DWMCOMPOSITIONCHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CAPTURECHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
}

impl<T: WindowMessageHandler> BaseWindowMessageHandler for T {
    fn message_handler(
        &mut self,
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if let Some(handled) = self.raw_message_handler(hwnd, message, wparam, lparam) {
            return handled;
        }

        let handled = match message {
            // ウィンドウライフサイクル
            WM_NCCREATE => self.WM_NCCREATE(hwnd, wparam, lparam),
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
            WM_NCCALCSIZE => self.WM_NCCALCSIZE(wparam, lparam),
            WM_SIZE => self.WM_SIZE(wparam, lparam),
            WM_SIZING => self.WM_SIZING(wparam, lparam),
            WM_MOVE => self.WM_MOVE(wparam, lparam),
            WM_MOVING => self.WM_MOVING(wparam, lparam),
            WM_WINDOWPOSCHANGING => self.WM_WINDOWPOSCHANGING(wparam, lparam),
            WM_WINDOWPOSCHANGED => self.WM_WINDOWPOSCHANGED(wparam, lparam),
            WM_GETMINMAXINFO => self.WM_GETMINMAXINFO(wparam, lparam),

            // マウス：移動、エンター、リーブ
            WM_MOUSEMOVE => {
                if !self.mouse_tracking() {
                    let mut tt = TRACKMOUSEEVENT {
                        cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                        dwFlags: TME_LEAVE,
                        hwndTrack: self.hwnd(),
                        dwHoverTime: HOVER_DEFAULT,
                    };
                    unsafe {
                        if TrackMouseEvent(&mut tt).is_ok() {
                            self.set_mouse_tracking(true);
                            self.WM_MOUSEENTER(wparam, lparam);
                        }
                    }
                }
                self.WM_MOUSEMOVE(wparam, lparam)
            }
            WM_MOUSELEAVE => {
                self.set_mouse_tracking(false);
                self.WM_MOUSELEAVE(wparam, lparam)
            }

            // マウス：その他
            WM_MOUSEWHEEL => self.WM_MOUSEWHEEL(wparam, lparam),
            WM_MOUSEHWHEEL => self.WM_MOUSEHWHEEL(wparam, lparam),
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

            // その他
            _ => None,
        };
        match handled {
            Some(res) => res,
            None => unsafe { DefWindowProcW(self.hwnd(), message, wparam, lparam) },
        }
    }
}

pub(crate) trait WindowMessageHandlerIntoBoxedPtr {
    fn into_boxed_ptr(self) -> *mut c_void;
}

impl WindowMessageHandlerIntoBoxedPtr for Rc<dyn BaseWindowMessageHandler> {
    fn into_boxed_ptr(self) -> *mut c_void {
        let boxed = Box::new(self);
        let raw = Box::into_raw(boxed);
        let ptr = raw as _;
        ptr
    }
}

fn get_boxed_ptr<'a>(ptr: *mut c_void) -> Option<&'a mut dyn BaseWindowMessageHandler> {
    if ptr.is_null() {
        return None;
    }
    unsafe {
        let raw: *mut Rc<dyn WindowMessageHandler> = ptr as _;
        let handler = &**raw;
        #[allow(mutable_transmutes)]
        let handler = std::mem::transmute::<_, &mut dyn BaseWindowMessageHandler>(handler);
        Some(handler)
    }
}

fn from_boxed_ptr(ptr: *mut c_void) -> Option<Rc<dyn BaseWindowMessageHandler>> {
    if ptr.is_null() {
        return None;
    }
    unsafe {
        let raw: *mut Rc<dyn BaseWindowMessageHandler> = ptr as _;
        let boxed = Box::from_raw(raw);
        Some(*boxed)
    }
}

pub(crate) extern "system" fn wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    //eprintln!("wndproc message: {}", message);
    let rc = unsafe {
        match message {
            WM_NCCREATE => {
                eprintln!("WM_NCCREATE");
                let cs = lparam.0 as *const CREATESTRUCTW;
                if cs.is_null() {
                    return LRESULT(0);
                }
                let ptr = (*cs).lpCreateParams;
                if let Some(handler) = get_boxed_ptr(ptr) {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as _);
                    handler.message_handler(hwnd, message, wparam, lparam);
                }
                LRESULT(1)
            }
            WM_NCDESTROY => {
                eprintln!("WM_NCDESTROY");
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as _;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                let _ = from_boxed_ptr(ptr);
                LRESULT(1)
            }
            _ => {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as _;
                if let Some(handler) = get_boxed_ptr(ptr) {
                    handler.message_handler(hwnd, message, wparam, lparam)
                } else {
                    DefWindowProcW(hwnd, message, wparam, lparam)
                }
            }
        }
    };
    //eprintln!("wndproc message: {} --> {:?}", message, rc);
    rc
}
