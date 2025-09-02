use crate::win_message_handler::{wndproc, WindowMessageHandler, WindowMessageHandlerExt};
use std::ffi::c_void;
use windows::core::*;
use windows::Win32::Foundation::{
    GetLastError, ERROR_CLASS_ALREADY_EXISTS, HWND, LPARAM, LRESULT, WPARAM,
};
use windows::Win32::System::Com::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

const WINTF_CLASS_NAME: &str = "WinTF_Window_Class";

pub struct WinThreadMgr {
    class_name: HSTRING,
}

impl WinThreadMgr {
    pub fn new() -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
            SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        }

        let class_name = HSTRING::from(WINTF_CLASS_NAME);
        let instance = unsafe { GetModuleHandleW(None)? };

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        unsafe {
            if RegisterClassExW(&wc) == 0 {
                // 他のスレッド等で既に登録されている場合はエラーを無視
                if GetLastError() != ERROR_CLASS_ALREADY_EXISTS {
                    return Err(Error::from_win32());
                }
            }
        }

        Ok(WinThreadMgr { class_name })
    }

    pub fn create_window<T: WindowMessageHandler + 'static>(
        &self,
        handler: T,
        window_name: &str,
        style: WINDOW_STYLE,
        ex_style: WINDOW_EX_STYLE,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: HWND,
    ) -> Result<HWND> {
        let window_name_hstring = HSTRING::from(window_name);

        let instance = unsafe { GetModuleHandleW(None)? };

        let handler_ptr = handler.into_raw();

        let hwnd = unsafe {
            CreateWindowExW(
                ex_style,
                &self.class_name,
                &window_name_hstring,
                style,
                x,
                y,
                width,
                height,
                parent,
                None,
                instance,
                Some(handler_ptr as *mut c_void),
            )
        };

        if hwnd.0 == 0 {
            Err(Error::from_win32())
        } else {
            Ok(hwnd)
        }
    }

    pub fn run_message_loop(&self) {
        let mut msg = MSG::default();
        unsafe {
            while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}
