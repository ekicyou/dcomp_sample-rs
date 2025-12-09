#![allow(deprecated)]

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
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::*;
use std::thread;
use std::time::Instant;
use tracing::{debug, info, trace};
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Dwm::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::WindowsAndMessaging::*;

// デバッグ用カウンター
static DEBUG_LOOP_COUNT: AtomicU64 = AtomicU64::new(0);
static DEBUG_VSYNC_COUNT: AtomicU64 = AtomicU64::new(0);
static DEBUG_OTHER_MSG_COUNT: AtomicU64 = AtomicU64::new(0);
static DEBUG_NO_MSG_COUNT: AtomicU64 = AtomicU64::new(0);

// ============================================================
// VSYNC優先レンダリング用カウンター
// モーダルループ（ウィンドウドラッグ等）中でもWndProcからVSYNC
// タイミングを検知してworld tickを実行可能にする。
// ============================================================

/// VSYNCスレッドがインクリメントするカウンター（VSYNC到来回数）
/// メインスレッドからload()で読み取り、tickが必要かどうかを判断する。
pub(crate) static VSYNC_TICK_COUNT: AtomicU64 = AtomicU64::new(0);

/// メインスレッドのみが更新するカウンター（前回処理したtick_count値）
/// try_tick_on_vsync()内でVSYNC_TICK_COUNTと比較し、異なればtickを実行する。
pub(crate) static LAST_VSYNC_TICK: AtomicU64 = AtomicU64::new(0);

// デバッグ用: WndProc経由のtick回数とrun()経由のtick回数を区別して計測
#[cfg(debug_assertions)]
pub(crate) static DEBUG_WNDPROC_TICK_COUNT: AtomicU64 = AtomicU64::new(0);
#[cfg(debug_assertions)]
pub(crate) static DEBUG_RUN_TICK_COUNT: AtomicU64 = AtomicU64::new(0);

// フレームワーク内部用カスタムメッセージ定義
// WM_USER (0x0400) ベース: ウィンドウクラス/フレームワーク固有のメッセージ
// WM_APP (0x8000) はアプリケーション側で自由に使用可能
const WM_VSYNC: u32 = WM_USER + 1;
pub(crate) const WM_LAST_WINDOW_DESTROYED: u32 = WM_USER + 2;

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
            debug!("Window creation...");
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
            debug!(hwnd = ?rc, "Window created");
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
        let mut last_stats_time = Instant::now();

        unsafe {
            loop {
                DEBUG_LOOP_COUNT.fetch_add(1, Ordering::Relaxed);

                // 1秒ごとに統計をログ出力
                let now = Instant::now();
                if now.duration_since(last_stats_time).as_secs() >= 1 {
                    let loop_count = DEBUG_LOOP_COUNT.swap(0, Ordering::Relaxed);
                    let vsync_count = DEBUG_VSYNC_COUNT.swap(0, Ordering::Relaxed);
                    let other_msg_count = DEBUG_OTHER_MSG_COUNT.swap(0, Ordering::Relaxed);
                    let no_msg_count = DEBUG_NO_MSG_COUNT.swap(0, Ordering::Relaxed);

                    // デバッグビルドのみ: tick実行元の区別
                    #[cfg(debug_assertions)]
                    {
                        let wndproc_tick = DEBUG_WNDPROC_TICK_COUNT.swap(0, Ordering::Relaxed);
                        let run_tick = DEBUG_RUN_TICK_COUNT.swap(0, Ordering::Relaxed);
                        debug!(
                            loop_count,
                            vsync_count,
                            other_msg_count,
                            no_msg_count,
                            wndproc_tick,
                            run_tick,
                            "Message loop stats"
                        );
                    }
                    #[cfg(not(debug_assertions))]
                    trace!(
                        loop_count,
                        vsync_count,
                        other_msg_count,
                        no_msg_count,
                        "Message loop stats"
                    );
                    last_stats_time = now;
                }

                if PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    if msg.message == WM_QUIT {
                        break;
                    }

                    // WM_VSYNCメッセージでECSを更新
                    // try_tick_on_vsync()を使用することで、WndProcで既に処理済みの場合は
                    // カウンター比較によりスキップされ、重複実行が防止される
                    if msg.message == WM_VSYNC {
                        DEBUG_VSYNC_COUNT.fetch_add(1, Ordering::Relaxed);

                        // World借用→tick→借用解放→flushの順で処理
                        // VsyncTickトレイトのtry_tick_on_vsync()と同じ流れを維持
                        let ticked = {
                            let mut world = self.world.borrow_mut();
                            world.try_tick_on_vsync()
                        };
                        // World借用解放後にSetWindowPosコマンドをフラッシュ
                        crate::ecs::window::flush_window_pos_commands();

                        // デバッグビルドのみ: run()経由のtick回数をカウント
                        #[cfg(debug_assertions)]
                        if ticked {
                            DEBUG_RUN_TICK_COUNT.fetch_add(1, Ordering::Relaxed);
                        }

                        continue;
                    }

                    // WM_LAST_WINDOW_DESTROYEDメッセージでアプリ終了
                    if msg.message == WM_LAST_WINDOW_DESTROYED {
                        info!("WM_LAST_WINDOW_DESTROYED received, calling PostQuitMessage(0)");
                        PostQuitMessage(0);
                        continue;
                    }

                    // デバッグ: 全メッセージをカウント
                    DEBUG_OTHER_MSG_COUNT.fetch_add(1, Ordering::Relaxed);

                    // デバッグ: WM_USER ベースのメッセージを監視
                    if msg.message >= 0x0400 && msg.message <= 0x040F {
                        trace!(
                            msg = format_args!("0x{:04X}", msg.message),
                            hwnd = ?msg.hwnd,
                            "WM_USER range message"
                        );
                    }

                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                    continue;
                }

                DEBUG_NO_MSG_COUNT.fetch_add(1, Ordering::Relaxed);

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
                    // VSync到来 - まずカウンターをインクリメント
                    // （WM_VSYNC送信より前に実行することで、WndProcからの検知を可能にする）
                    VSYNC_TICK_COUNT.fetch_add(1, Ordering::Relaxed);

                    // メッセージウィンドウに通知（従来の動作を維持）
                    let _ = PostMessageW(Some(message_window), WM_VSYNC, WPARAM(0), LPARAM(0));
                } else {
                    // エラーの場合は少し待機してリトライ
                    thread::sleep(std::time::Duration::from_millis(15)); // 約60Hz
                }
            }
        }
    })
}
