//! アプリケーション全体の状態を管理するリソース

use bevy_ecs::prelude::*;
use windows::Win32::UI::WindowsAndMessaging::PostQuitMessage;

/// アプリケーション全体の状態を管理するリソース
#[derive(Resource, Default)]
pub struct App {
    window_count: usize,
}

impl App {
    /// 新しいAppリソースを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// ウィンドウが作成されたときに呼ばれる
    pub fn on_window_created(&mut self) {
        self.window_count += 1;
        eprintln!("[App] Window created. Total windows: {}", self.window_count);
    }

    /// ウィンドウが破棄されたときに呼ばれる
    /// 最後のウィンドウが閉じられた場合はtrueを返す
    pub fn on_window_destroyed(&mut self) -> bool {
        self.window_count = self.window_count.saturating_sub(1);
        eprintln!("[App] Window destroyed. Remaining windows: {}", self.window_count);
        
        if self.window_count == 0 {
            eprintln!("[App] Last window closed. Quitting application...");
            unsafe {
                PostQuitMessage(0);
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
