//! アプリケーション全体の状態を管理するリソース

use bevy_ecs::prelude::*;
use windows::Win32::{Foundation::{HWND, WPARAM, LPARAM}, UI::WindowsAndMessaging::PostMessageW};

/// アプリケーション全体の状態を管理するリソース
#[derive(Resource)]
pub struct App {
    window_count: usize,
    message_window: Option<isize>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window_count: 0,
            message_window: None,
        }
    }
}

impl App {
    /// 新しいAppリソースを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// メッセージウィンドウのHWNDを設定
    pub fn set_message_window(&mut self, hwnd: HWND) {
        self.message_window = Some(hwnd.0 as isize);
    }

    /// ウィンドウが作成されたときに呼ばれる
    pub fn on_window_created(&mut self, entity: Entity) {
        self.window_count += 1;
        eprintln!("[App] Window created. Entity: {:?}, Total windows: {}", entity, self.window_count);
    }

    /// ウィンドウが破棄されたときに呼ばれる
    /// 最後のウィンドウが閉じられた場合はtrueを返す
    pub fn on_window_destroyed(&mut self, entity: Entity) -> bool {
        self.window_count = self.window_count.saturating_sub(1);
        eprintln!("[App] Window destroyed. Entity: {:?}, Remaining windows: {}", entity, self.window_count);
        
        if self.window_count == 0 {
            eprintln!("[App] Last window closed. Quitting application...");
            if let Some(hwnd_raw) = self.message_window {
                unsafe {
                    let _ = PostMessageW(
                        Some(HWND(hwnd_raw as *mut _)),
                        crate::win_thread_mgr::WM_LAST_WINDOW_DESTROYED,
                        WPARAM(0),
                        LPARAM(0),
                    );
                }
            }
            true
        } else {
            false
        }
    }

    /// 現在のウィンドウ数を取得
    pub fn window_count(&self) -> usize {
        self.window_count
    }
}
