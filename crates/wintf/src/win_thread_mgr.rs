use crate::win_message_handler::*;
use std::rc::*;
use std::sync::*;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

// ウィンドウクラスの管理

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

pub struct WinThreadMgr {}

impl WinThreadMgr {
    pub fn new() -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
            SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        }
        Ok(WinThreadMgr {})
    }

    pub fn instance(&self) -> HINSTANCE {
        let singleton = WinProcessSingleton::get_or_init();
        singleton.instance()
    }

    pub fn create_window(
        &mut self,
        handler: Rc<dyn BaseWinMessageHandler>,
        window_name: &str,
        style: WINDOW_STYLE,
        ex_style: WINDOW_EX_STYLE,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Option<HWND>,
    ) -> Result<HWND> {
        let singleton = WinProcessSingleton::get_or_init();
        let window_name_hstring = HSTRING::from(window_name);
        let boxed_ptr = handler.into_boxed_ptr();

        unsafe {
            eprintln!("Window creation...");
            let rc = CreateWindowExW(
                ex_style,
                singleton.window_class_name(),
                &window_name_hstring,
                style,
                x,
                y,
                width,
                height,
                parent,
                None,
                None,
                Some(boxed_ptr),
            );
            eprintln!("Window created {:?}", rc);
            rc
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut msg = MSG::default();
        unsafe {
            while GetMessageW(&mut msg, None, 0, 0).into() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        Ok(())
    }
}
