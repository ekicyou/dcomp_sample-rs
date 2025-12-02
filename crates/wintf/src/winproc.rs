#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(deprecated)]

use crate::win_message_handler::*;
use std::ffi::c_void;
use std::sync::*;
use tracing::{debug, info};
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

pub(crate) trait WinMessageHandlerIntoBoxedPtr {
    fn into_boxed_ptr(self) -> *mut c_void;
}

impl WinMessageHandlerIntoBoxedPtr for Arc<dyn BaseWinMessageHandler> {
    fn into_boxed_ptr(self) -> *mut c_void {
        let boxed = Box::new(self);
        let raw = Box::into_raw(boxed);
        let ptr = raw as _;
        ptr
    }
}

fn get_boxed_ptr<'a>(ptr: *mut c_void) -> Option<&'a mut dyn BaseWinMessageHandler> {
    if ptr.is_null() {
        return None;
    }
    unsafe {
        let raw: *mut Arc<dyn WinMessageHandler> = ptr as _;
        let handler = &**raw;
        #[allow(mutable_transmutes)]
        let handler = std::mem::transmute::<_, &mut dyn BaseWinMessageHandler>(handler);
        Some(handler)
    }
}

fn from_boxed_ptr(ptr: *mut c_void) -> Option<Arc<dyn BaseWinMessageHandler>> {
    if ptr.is_null() {
        return None;
    }
    unsafe {
        let raw: *mut Arc<dyn BaseWinMessageHandler> = ptr as _;
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
                debug!(hwnd = ?hwnd, "WM_NCCREATE");
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
                debug!(hwnd = ?hwnd, "WM_NCDESTROY");
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as _;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                let _ = from_boxed_ptr(ptr);
                LRESULT(1)
            }
            crate::win_thread_mgr::WM_LAST_WINDOW_DESTROYED => {
                // 最後のウィンドウが閉じられた通知を受け取ったらアプリケーションを終了
                info!("Received WM_LAST_WINDOW_DESTROYED, posting quit message");
                PostQuitMessage(0);
                LRESULT(0)
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
