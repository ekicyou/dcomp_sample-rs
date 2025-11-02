use std::sync::*;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::winproc::*;

const WINTF_CLASS_NAME: &str = "wintf_window_class";

static WIN_PROCESS_SINGLETON: OnceLock<WinProcessSingleton> = OnceLock::new();

pub(crate) struct WinProcessSingleton {
    instance: HINSTANCE,
    window_class_name: HSTRING,
}

unsafe impl Send for WinProcessSingleton {}
unsafe impl Sync for WinProcessSingleton {}

impl WinProcessSingleton {
    pub(crate) fn instance(&self) -> HINSTANCE {
        self.instance
    }

    pub(crate) fn window_class_name(&self) -> &HSTRING {
        &self.window_class_name
    }

    pub(crate) fn get_or_init() -> &'static Self {
        WIN_PROCESS_SINGLETON.get_or_init(|| {
            eprintln!("window class creation...");
            let instance = unsafe { GetModuleHandleW(None).unwrap().into() };
            let window_class_name = HSTRING::from(WINTF_CLASS_NAME);
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wndproc),
                hInstance: instance,
                hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap() },
                lpszClassName: PCWSTR(window_class_name.as_ptr()),
                ..Default::default()
            };
            unsafe {
                let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
                if RegisterClassExW(&wc) == 0 {
                    panic!("Failed to register window class");
                }
            }
            eprintln!("window class created");
            Self {
                instance,
                window_class_name,
            }
        })
    }
}
