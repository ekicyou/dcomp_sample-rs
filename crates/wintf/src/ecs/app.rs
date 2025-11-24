//! アプリケーション全体の状態を管理するリソース

use bevy_ecs::prelude::*;
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::PostMessageW,
};

/// アプリケーション全体の状態を管理するリソース
#[derive(Resource)]
pub struct App {
    window_count: usize,
    message_window: Option<isize>,
    display_configuration_changed: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window_count: 0,
            message_window: None,
            display_configuration_changed: false,
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

    /// ディスプレイ構成が変更されたことをマーク
    pub fn mark_display_change(&mut self) {
        self.display_configuration_changed = true;
        eprintln!("[App] Display configuration changed");
    }

    /// ディスプレイ構成変更フラグをリセット
    pub fn reset_display_change(&mut self) {
        self.display_configuration_changed = false;
    }

    /// ディスプレイ構成が変更されたかどうかを取得
    pub fn display_configuration_changed(&self) -> bool {
        self.display_configuration_changed
    }

    /// ウィンドウが作成されたときに呼ばれる
    pub fn on_window_created(&mut self, entity: Entity) {
        self.window_count += 1;
        eprintln!(
            "[App] Window created. Entity: {:?}, Total windows: {}",
            entity, self.window_count
        );
    }

    /// ウィンドウが破棄されたときに呼ばれる
    /// 最後のウィンドウが閉じられた場合はtrueを返す
    pub fn on_window_destroyed(&mut self, entity: Entity) -> bool {
        self.window_count = self.window_count.saturating_sub(1);
        eprintln!(
            "[App] Window destroyed. Entity: {:?}, Remaining windows: {}",
            entity, self.window_count
        );

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
