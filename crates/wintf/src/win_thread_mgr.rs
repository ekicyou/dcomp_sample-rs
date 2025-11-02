use crate::ecs::*;
use crate::process_singleton::*;
use crate::win_message_handler::*;
use crate::win_style::*;
use crate::win_timer::*;
use crate::winproc::*;
use async_executor::*;
use std::cell::RefCell;
use std::future::*;
use std::ops::Deref;
use std::sync::*;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::WindowsAndMessaging::*;

const TIMER_ID_ECS_TICK: usize = 1;

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
    world: RefCell<EcsWorld>,
    timer: RefCell<Option<WinTimer>>,
}

impl WinThreadMgrInner {
    fn new() -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        }
        let rc = WinThreadMgrInner {
            executor_normal: Executor::new(),
            world: RefCell::new(EcsWorld::new()),
            timer: RefCell::new(None),
        };
        let _ = rc.instance();
        Ok(rc)
    }

    pub fn world(&self) -> std::cell::Ref<'_, EcsWorld> {
        self.world.borrow()
    }

    pub fn world_mut(&self) -> std::cell::RefMut<'_, EcsWorld> {
        self.world.borrow_mut()
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
        // ECSシステムがある場合はタイマーウィンドウを作成
        let timer_hwnd = if self.world.borrow().world().entities().len() > 0 || true {
            // 隠しウィンドウを作成してタイマーを設定
            let singleton = WinProcessSingleton::get_or_init();
            unsafe {
                let hwnd = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    singleton.window_class_name(),
                    w!("ECS Timer Window"),
                    WS_OVERLAPPED,
                    0,
                    0,
                    0,
                    0,
                    Some(HWND_MESSAGE), // メッセージ専用ウィンドウ
                    None,
                    None,
                    None,
                )?;

                // 60fps用タイマーを設定
                *self.timer.borrow_mut() = WinTimer::new(hwnd, TIMER_ID_ECS_TICK, 16);
                Some(hwnd)
            }
        } else {
            None
        };

        let mut world = self.world.borrow_mut();
        let mut msg = MSG::default();
        unsafe {
            loop {
                if PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    if msg.message == WM_QUIT {
                        break;
                    }

                    // WM_TIMERメッセージでECSを更新
                    if msg.message == WM_TIMER && msg.wParam.0 == TIMER_ID_ECS_TICK {
                        world.try_tick_world();
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

            // タイマーウィンドウを破棄
            if let Some(hwnd) = timer_hwnd {
                let _ = DestroyWindow(hwnd);
            }
        }
        Ok(())
    }

    fn try_tick_normal(&self) -> bool {
        self.executor_normal.try_tick()
    }
}
