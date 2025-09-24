use crate::win_message_handler::*;
use crate::win_style::*;
use crate::winproc::*;
use async_executor::*;
use std::future::*;
use std::ops::Deref;
use std::sync::*;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::LibraryLoader::*;
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

#[derive(Clone)]
pub struct WinThreadMgr(Arc<WinThreadMgrInner>);

impl WinThreadMgr {
    pub fn new() -> Result<Self> {
        let inner = Arc::new(WinThreadMgrInner::new()?);
        Ok(Self(inner))
    }
}

impl Deref for WinThreadMgr {
    type Target = Arc<WinThreadMgrInner>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct WinThreadMgrInner {
    executor_normal: Executor<'static>,
}

impl WinThreadMgrInner {
    fn new() -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        }
        let rc = WinThreadMgrInner {
            executor_normal: Executor::new(),
        };
        let _ = rc.instance();
        Ok(rc)
    }

    pub fn instance(&self) -> HINSTANCE {
        let singleton = WinProcessSingleton::get_or_init();
        singleton.instance()
    }

    pub fn create_window<S1>(
        &self,
        handler: Arc<dyn BaseWinMessageHandler>,
        window_name: S1,
        style: WinStyle,
    ) -> Result<HWND>
    where
        S1: Into<HSTRING>,
    {
        let singleton = WinProcessSingleton::get_or_init();
        let window_name_hstring: HSTRING = window_name.into();
        let boxed_ptr = handler.into_boxed_ptr();

        unsafe {
            eprintln!("Window creation...");
            let rc = CreateWindowExW(
                style.ex_style,
                singleton.window_class_name(),
                &window_name_hstring,
                style.style,
                style.x,
                style.y,
                style.width,
                style.height,
                style.parent,
                None,
                None,
                Some(boxed_ptr),
            );
            eprintln!("Window created {:?}", rc);
            rc
        }
    }

    pub fn spawn_normal<T: Send + 'static>(
        &self,
        fut: impl Future<Output = T> + Send + 'static,
    ) -> Task<T> {
        self.executor_normal.spawn(fut)
    }

    pub fn run(&self) -> Result<()> {
        let mut msg = MSG::default();
        unsafe {
            loop {
                if PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    if msg.message == WM_QUIT {
                        break;
                    }
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                    continue;
                }

                if self.try_tick_normal() {
                    continue;
                }

                // メッセージがない場合は待機
                let _ = WaitMessage();
            }
        }
        Ok(())
    }

    fn try_tick_normal(&self) -> bool {
        self.executor_normal.try_tick()
    }
}
