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
    world: Rc<RefCell<EcsWorld>>,
    message_window: HWND,
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

        // ECS更新用タイマーを設定（10ms → 実際は15.6msに丸められる）
        unsafe {
            SetTimer(Some(message_window), TIMER_ID_ECS_TICK, 10, None);
        }

        let world = Rc::new(RefCell::new(EcsWorld::new()));
        
        // EcsWorldへの弱参照を登録（wndprocからアクセスするため）
        crate::ecs::set_ecs_world(Rc::downgrade(&world));

        let rc = WinThreadMgrInner {
            executor_normal: Executor::new(),
            world,
            message_window,
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

                    // WM_TIMERメッセージでECSを更新
                    if msg.message == WM_TIMER && msg.wParam.0 == TIMER_ID_ECS_TICK {
                        let mut world = self.world.borrow_mut();
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
        }
        Ok(())
    }

    fn try_tick_normal(&self) -> bool {
        self.executor_normal.try_tick()
    }
}

impl Drop for WinThreadMgrInner {
    fn drop(&mut self) {
        unsafe {
            let _ = KillTimer(Some(self.message_window), TIMER_ID_ECS_TICK);
            let _ = DestroyWindow(self.message_window);
        }
    }
}
