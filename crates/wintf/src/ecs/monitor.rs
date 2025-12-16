//! # Monitor - マルチモニター管理
//!
//! このモジュールは、物理モニターのECSコンポーネントとモニター列挙機能を提供します。
//!
//! ## 主要コンポーネント
//!
//! - **`Monitor`**: 物理モニター情報（handle, bounds, DPI, プライマリフラグ）
//!
//! ## 主要関数
//!
//! - **`enumerate_monitors()`**: システムの全モニターを列挙
//!
//! ## 使用例
//!
//! ```rust,ignore
//! use wintf::ecs::monitor::{Monitor, enumerate_monitors};
//!
//! // 全モニターを列挙
//! let monitors = enumerate_monitors();
//! for monitor in monitors {
//!     println!("Monitor: {:?}, DPI: {}, Primary: {}",
//!              monitor.bounds, monitor.dpi, monitor.is_primary);
//! }
//! ```

use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::prelude::*;
use bevy_ecs::world::DeferredWorld;
use std::fmt;
use tracing::warn;
use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HMONITOR, MONITORINFOEXW,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows_core::BOOL;

/// Monitorコンポーネントが追加されたときに呼ばれるフック
/// Arrangementを自動挿入する（既に存在する場合はスキップ）
fn on_monitor_add(mut world: DeferredWorld, context: HookContext) {
    let entity = context.entity;
    // Arrangementがまだ存在しない場合のみ挿入
    if world
        .get::<crate::ecs::layout::Arrangement>(entity)
        .is_none()
    {
        world
            .commands()
            .entity(entity)
            .insert(crate::ecs::layout::Arrangement::default());
    }
}

/// モニターコンポーネント
///
/// 物理モニターの情報を保持します。
/// - `handle`: モニターハンドル
/// - `bounds`: モニターの画面座標系での矩形（ピクセル単位）
/// - `work_area`: タスクバーなどを除いた作業領域
/// - `dpi`: モニターのDPI値
/// - `is_primary`: プライマリモニターかどうか
///
/// # ライフタイムイベント
/// - `on_add`: `Arrangement::default()`を自動挿入
///   - これにより`Arrangement`の`on_add`が連鎖的に`GlobalArrangement`と`ArrangementTreeChanged`を挿入
#[derive(Component, Clone)]
#[component(on_add = on_monitor_add)]
pub struct Monitor {
    pub handle: isize,
    pub bounds: RECT,
    pub work_area: RECT,
    pub dpi: u32,
    pub is_primary: bool,
}

impl fmt::Debug for Monitor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Monitor")
            .field("handle", &self.handle)
            .field(
                "bounds",
                &format_args!(
                    "({},{},{},{})",
                    self.bounds.left, self.bounds.top, self.bounds.right, self.bounds.bottom
                ),
            )
            .field(
                "work_area",
                &format_args!(
                    "({},{},{},{})",
                    self.work_area.left,
                    self.work_area.top,
                    self.work_area.right,
                    self.work_area.bottom
                ),
            )
            .field("dpi", &self.dpi)
            .field("is_primary", &self.is_primary)
            .finish()
    }
}

impl PartialEq for Monitor {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl Monitor {
    /// HMONITORからMonitorを構築
    pub fn from_hmonitor(hmonitor: HMONITOR) -> Result<Self, MonitorError> {
        unsafe {
            // GetMonitorInfoWでモニター情報を取得
            let mut monitor_info: MONITORINFOEXW = std::mem::zeroed();
            monitor_info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

            let success =
                GetMonitorInfoW(hmonitor, &mut monitor_info.monitorInfo as *mut _ as *mut _);
            if !success.as_bool() {
                return Err(MonitorError::GetMonitorInfoFailed);
            }

            // GetDpiForMonitorでDPI取得
            let mut dpi_x: u32 = 0;
            let mut dpi_y: u32 = 0;
            let hr = GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
            if hr.is_err() {
                return Err(MonitorError::GetDpiFailed);
            }

            Ok(Monitor {
                handle: hmonitor.0 as isize,
                bounds: monitor_info.monitorInfo.rcMonitor,
                work_area: monitor_info.monitorInfo.rcWork,
                dpi: dpi_x,
                is_primary: (monitor_info.monitorInfo.dwFlags & 1) != 0,
            })
        }
    }

    /// モニターの物理サイズ（幅、高さ）を取得
    pub fn physical_size(&self) -> (f32, f32) {
        let width = (self.bounds.right - self.bounds.left) as f32;
        let height = (self.bounds.bottom - self.bounds.top) as f32;
        (width, height)
    }

    /// モニターの左上座標を取得
    pub fn top_left(&self) -> (f32, f32) {
        (self.bounds.left as f32, self.bounds.top as f32)
    }
}

/// モニター関連エラー
#[derive(Debug, Clone, Copy)]
pub enum MonitorError {
    GetMonitorInfoFailed,
    GetDpiFailed,
}

impl fmt::Display for MonitorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MonitorError::GetMonitorInfoFailed => write!(f, "GetMonitorInfoW failed"),
            MonitorError::GetDpiFailed => write!(f, "GetDpiForMonitor failed"),
        }
    }
}

impl std::error::Error for MonitorError {}

/// システムの全モニターを列挙
pub fn enumerate_monitors() -> Vec<Monitor> {
    let mut monitors = Vec::new();

    // EnumDisplayMonitorsのコールバック関数
    extern "system" fn enum_proc(
        hmonitor: HMONITOR,
        _hdc: windows::Win32::Graphics::Gdi::HDC,
        _rect: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        let monitors = unsafe { &mut *(lparam.0 as *mut Vec<Monitor>) };
        if let Ok(monitor) = Monitor::from_hmonitor(hmonitor) {
            monitors.push(monitor);
        } else {
            warn!(hmonitor = ?hmonitor, "Failed to get monitor info");
        }
        BOOL(1)
    }

    let monitors_ptr = &mut monitors as *mut Vec<Monitor> as isize;
    let result = unsafe { EnumDisplayMonitors(None, None, Some(enum_proc), LPARAM(monitors_ptr)) };

    if !result.as_bool() {
        warn!("EnumDisplayMonitors failed");
        return Vec::new();
    }

    monitors
}
