use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

/// Windowsタイマーのラッパー（RAII管理）
#[derive(Debug)]
pub struct WinTimer {
    hwnd: HWND,
    timer_id: usize,
}

impl WinTimer {
    /// タイマーを作成
    /// interval_ms: タイマー間隔（ミリ秒）
    pub fn new(hwnd: HWND, timer_id: usize, interval_ms: u32) -> Option<Self> {
        unsafe {
            let result = SetTimer(Some(hwnd), timer_id, interval_ms, None);
            if result == 0 {
                return None;
            }
        }
        Some(Self { hwnd, timer_id })
    }
}

impl Drop for WinTimer {
    fn drop(&mut self) {
        unsafe {
            let _ = KillTimer(Some(self.hwnd), self.timer_id);
        }
    }
}
