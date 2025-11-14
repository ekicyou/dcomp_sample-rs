use std::sync::*;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::winproc::*;

const WINTF_CLASS_NAME: &str = "wintf_window_class";
const WINTF_ECS_CLASS_NAME: &str = "wintf_ecs_window_class";

static WIN_PROCESS_SINGLETON: OnceLock<WinProcessSingleton> = OnceLock::new();

#[derive(Debug)]
pub struct WinProcessSingleton {
    instance: HINSTANCE,
    window_class_name: HSTRING,
    ecs_window_class_name: HSTRING,
    #[allow(dead_code)]
    hidden_window: Option<HWND>,
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

    pub(crate) fn ecs_window_class_name(&self) -> &HSTRING {
        &self.ecs_window_class_name
    }

    #[allow(dead_code)]
    pub(crate) fn hidden_window(&self) -> Option<HWND> {
        self.hidden_window
    }

    pub(crate) fn get_or_init() -> &'static Self {
        WIN_PROCESS_SINGLETON.get_or_init(|| {
            eprintln!("window class creation...");
            let instance = unsafe { GetModuleHandleW(None).unwrap().into() };
            let window_class_name = HSTRING::from(WINTF_CLASS_NAME);
            let ecs_window_class_name = HSTRING::from(WINTF_ECS_CLASS_NAME);

            // 既存のウィンドウクラスを登録（dcomp_demo用）
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

            // ECS用のウィンドウクラスを登録
            let ecs_wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(crate::ecs::ecs_wndproc),
                hInstance: instance,
                hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap() },
                lpszClassName: PCWSTR(ecs_window_class_name.as_ptr()),
                ..Default::default()
            };
            unsafe {
                if RegisterClassExW(&ecs_wc) == 0 {
                    panic!("Failed to register ECS window class");
                }
            }

            eprintln!("window classes created");
            Self {
                instance,
                window_class_name,
                ecs_window_class_name,
                hidden_window: None,
            }
        })
    }
}
