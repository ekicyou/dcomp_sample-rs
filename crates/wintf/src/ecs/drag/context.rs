//! ドラッグコンテキスト（ECS→wndprocスレッド間転送用）
//!
//! dispatch_drag_events（ECSスレッド）がドラッグ開始時にWindowDragContextを書き込み、
//! update_dragging（wndprocスレッド）がJustStarted→Dragging遷移時に読み取る。

use bevy_ecs::prelude::*;
use std::sync::{Arc, Mutex};
use windows::Win32::Foundation::{HWND, POINT};

use super::DragConstraint;

/// wndprocスレッドでのドラッグに必要なWindow情報
#[derive(Debug, Clone)]
pub struct WindowDragContext {
    /// Window の Win32 ハンドル
    pub hwnd: Option<HWND>,
    /// ドラッグ開始時のウィンドウ位置（ウィンドウ枠を含むスクリーン座標、SetWindowPos用）
    pub initial_window_pos: Option<POINT>,
    /// DragConfig.move_window のキャッシュ
    pub move_window: bool,
    /// DragConstraint のキャッシュ
    pub constraint: Option<DragConstraint>,
}

// HWND は *mut c_void を含むため自動的に Send/Sync を実装しないが、
// ウィンドウハンドルは実質的にただの整数値であり、スレッド間の受け渡しは安全。
// Arc<Mutex> でラップされているため、データ競合も発生しない。
unsafe impl Send for WindowDragContext {}
unsafe impl Sync for WindowDragContext {}

impl Default for WindowDragContext {
    fn default() -> Self {
        Self {
            hwnd: None,
            initial_window_pos: None,
            move_window: false,
            constraint: None,
        }
    }
}

impl WindowDragContext {
    /// コンテキストをクリアする
    pub fn clear(&mut self) {
        self.hwnd = None;
        self.initial_window_pos = None;
        self.move_window = false;
        self.constraint = None;
    }
}

/// ECSスレッド→wndprocスレッド間でドラッグ開始時のWindow情報を転送するリソース
///
/// # ライフサイクル
/// - dispatch_drag_events (ECS スレッド) が Started 処理時に更新
/// - update_dragging (wndproc スレッド) が JustStarted → Dragging 遷移時に読み取り
#[derive(Resource, Clone)]
pub struct WindowDragContextResource {
    inner: Arc<Mutex<WindowDragContext>>,
}

impl WindowDragContextResource {
    /// 新しいWindowDragContextResourceを作成
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(WindowDragContext::default())),
        }
    }

    /// コンテキストを設定（ECSスレッドから呼ばれる）
    pub fn set(&self, context: WindowDragContext) {
        if let Ok(mut inner) = self.inner.lock() {
            *inner = context;
        }
    }

    /// コンテキストを読み取る（wndprocスレッドから呼ばれる）
    pub fn get(&self) -> Option<WindowDragContext> {
        self.inner.lock().ok().map(|inner| inner.clone())
    }

    /// コンテキストをクリアする
    pub fn clear(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.clear();
        }
    }
}

impl Default for WindowDragContextResource {
    fn default() -> Self {
        Self::new()
    }
}
