mod app;
pub mod common;
pub mod drag;
mod graphics;
pub mod layout;
pub mod monitor;
mod nchittest_cache;
pub mod pointer;
pub mod transform;
pub mod widget;
pub mod window;
mod window_proc;
mod window_system;
pub mod world;

/// 後方互換性のためのエイリアス
#[deprecated(since = "0.1.0", note = "Use pointer module instead")]
pub mod mouse {
    pub use super::pointer::*;
}

pub use app::*;
pub use bevy_ecs::hierarchy::{ChildOf, Children};
pub use common::tree_system::*;
pub use drag::{
    DragConfig, DragConstraint, DragEndEvent, DragEvent, DragStartEvent, DraggingState, OnDrag,
    OnDragEnd, OnDragStart, cleanup_drag_state, dispatch_drag_events,
};
pub use graphics::FrameTime;
pub use graphics::calculate_surface_size_from_global_arrangement;
pub use graphics::*;
pub use layout::*;
pub use monitor::*;
pub use pointer::{
    CursorVelocity, DoubleClick, EventHandler, OnPointerEntered, OnPointerExited, OnPointerMoved,
    OnPointerPressed, OnPointerReleased, Phase, PhysicalPoint, PointerButton, PointerEventHandler,
    PointerLeave, PointerState, WheelDelta, WindowPointerTracking, clear_transient_pointer_state,
    debug_pointer_leave, debug_pointer_state_changes, dispatch_pointer_events,
    process_pointer_buffers,
};
// 後方互換性エイリアス
#[allow(deprecated)]
pub use pointer::{
    MouseButton, MouseLeave, MouseState, WindowMouseTracking, clear_transient_mouse_state,
    debug_mouse_leave, debug_mouse_state_changes, process_mouse_buffers,
};
pub use transform::*;
pub use widget::{
    BitmapSource, BitmapSourceGraphics, BitmapSourceResource, BoxedCommand, CommandSender, WicCore,
    WintfTaskPool, draw_bitmap_sources,
};
pub use widget::{
    Typewriter, TypewriterEvent, TypewriterEventKind, TypewriterState, TypewriterTalk,
    TypewriterTimeline, TypewriterToken, draw_typewriters, update_typewriters,
};
pub use window::{
    DPI, DpiChangeContext, SetWindowPosCommand, Window, WindowHandle, WindowPos, WindowStyle,
    ZOrder, flush_window_pos_commands, guarded_set_window_pos, is_self_initiated,
};
pub(crate) use window_proc::{ecs_wndproc, set_ecs_world};
pub use world::{
    FrameCount, FrameFinalize, Input, Layout, PostLayout, PreLayout, PreRenderSurface, UISetup,
    Update,
};
