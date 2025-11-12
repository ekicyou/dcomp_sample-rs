use crate::ecs::world::*;
use crate::process_singleton::*;
use crate::win_message_handler::*;
use crate::win_style::*;
use crate::winproc::*;
use async_executor::*;
use std::cell::RefCell;
use std::future::*;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::*;
use std::thread;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Dwm::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::WindowsAndMessaging::*;

// VSync通知用のカスタムメッセージ
const WM_VSYNC: u32 = WM_APP + 1;

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
    world: Rc<RefCell<EcsWorld>>,
    message_window: HWND,
    vsync_thread_stop: Arc<AtomicBool>,
    vsync_thread_handle: Option<thread::JoinHandle<()>>,
}

impl WinThreadMgrInner {
    pub fn instance(&self) -> HINSTANCE {
        let singleton = WinProcessSingleton::get_or_init();
        singleton.instance()
    }

    pub fn world(&self) -> Rc<RefCell<EcsWorld>> {
        Rc::clone(&self.world)
    }

    fn new() -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        }

        // メッセージ専用の隠しウィンドウを作成
        let singleton = WinProcessSingleton::get_or_init();
        let message_window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                singleton.window_class_name(),
                w!("wintf Message Window"),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                None,
                None,
            )?
        };

        let world = Rc::new(RefCell::new(EcsWorld::new()));

        // メッセージウィンドウのHWNDをEcsWorldに設定
        world.borrow_mut().set_message_window(message_window);

        // EcsWorldへの弱参照を登録（wndprocからアクセスするため）
        crate::ecs::set_ecs_world(Rc::downgrade(&world));

        // VSync監視スレッドを起動
        let vsync_thread_stop = Arc::new(AtomicBool::new(false));
        let vsync_thread_handle =
            spawn_vsync_thread(message_window, Arc::clone(&vsync_thread_stop));

        let rc = WinThreadMgrInner {
            executor_normal: Executor::new(),
            world,
            message_window,
            vsync_thread_stop,
            vsync_thread_handle: Some(vsync_thread_handle),
        };
        let _ = rc.instance();
        Ok(rc)
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

    pub fn show_window(&self, hwnd: HWND) -> Result<()> {
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
        }
        Ok(())
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

                    // WM_VSYNCメッセージでECSを更新
                    if msg.message == WM_VSYNC {
                        let mut world = self.world.borrow_mut();
                        world.try_tick_world();
                        continue;
                    }

                    // WM_LAST_WINDOW_DESTROYEDメッセージでアプリ終了
                    if msg.message == crate::ecs::window::WM_LAST_WINDOW_DESTROYED {
                        eprintln!("[WinThreadMgr] WM_LAST_WINDOW_DESTROYED received. Calling PostQuitMessage(0).");
                        PostQuitMessage(0);
                        continue;
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

impl Drop for WinThreadMgrInner {
    fn drop(&mut self) {
        // VSync監視スレッドを停止
        self.vsync_thread_stop.store(true, Ordering::Relaxed);

        // スレッドの終了を待つ
        if let Some(handle) = self.vsync_thread_handle.take() {
            let _ = handle.join();
        }

        unsafe {
            let _ = DestroyWindow(self.message_window);
        }
    }
}

/// VSync監視スレッドを起動
/// DwmFlushを使用してVSyncと同期
fn spawn_vsync_thread(message_window: HWND, stop_flag: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    // HWNDはSendではないので、isizeとして保持
    let message_window_ptr = message_window.0 as isize;

    thread::spawn(move || {
        // isizeからHWNDを復元
        let message_window = HWND(message_window_ptr as *mut _);

        // VSync待機ループ
        loop {
            // 停止フラグをチェック
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            // DwmFlush: DWM（Desktop Window Manager）のVSyncを待機
            // この関数は次のVSyncまでブロックする
            unsafe {
                if DwmFlush().is_ok() {
                    // VSync到来 - メッセージウィンドウに通知
                    let _ = PostMessageW(Some(message_window), WM_VSYNC, WPARAM(0), LPARAM(0));
                } else {
                    // エラーの場合は少し待機してリトライ
                    thread::sleep(std::time::Duration::from_millis(15)); // 約60Hz
                }
            }
        }
    })
}
