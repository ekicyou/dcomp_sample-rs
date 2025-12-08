mod app;
pub mod common;
pub mod drag;
mod graphics;
pub mod layout;
pub mod monitor;
pub mod pointer;
mod nchittest_cache;
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
    apply_window_drag_movement, cleanup_drag_state, dispatch_drag_events, 
    DragConfig, DragConstraint, DragEndEvent, DragEvent, DragStartEvent, 
    DraggingMarker, OnDrag, OnDragEnd, OnDragStart,
};
pub use graphics::calculate_surface_size_from_global_arrangement;
pub use graphics::FrameTime;
pub use graphics::*;
pub use layout::*;
pub use monitor::*;
pub use pointer::{
    clear_transient_pointer_state, debug_pointer_leave, debug_pointer_state_changes,
    dispatch_pointer_events, process_pointer_buffers, CursorVelocity, DoubleClick, EventHandler,
    OnPointerEntered, OnPointerExited, OnPointerMoved, OnPointerPressed, OnPointerReleased,
    Phase, PhysicalPoint, PointerButton, PointerEventHandler, PointerLeave, PointerState,
    WheelDelta, WindowPointerTracking,
};
// 後方互換性エイリアス
#[allow(deprecated)]
pub use pointer::{
    clear_transient_mouse_state, debug_mouse_leave, debug_mouse_state_changes,
    process_mouse_buffers, MouseButton, MouseLeave, MouseState, WindowMouseTracking,
};
pub use transform::*;
pub use widget::{
    draw_bitmap_sources, BitmapSource, BitmapSourceGraphics, BitmapSourceResource, BoxedCommand,
    CommandSender, WicCore, WintfTaskPool,
};
pub use widget::{
    draw_typewriters, update_typewriters, Typewriter, TypewriterEvent, TypewriterEventKind,
    TypewriterState, TypewriterTalk, TypewriterTimeline, TypewriterToken,
};
pub use window::{
    flush_window_pos_commands, DpiChangeContext, SetWindowPosCommand, Window, WindowHandle,
    WindowPos, WindowPosChanged, WindowStyle, ZOrder, DPI,
};
pub(crate) use window_proc::{ecs_wndproc, set_ecs_world};
pub use world::{
    FrameCount, FrameFinalize, Input, Layout, PostLayout, PreLayout, PreRenderSurface, UISetup,
    Update,
};
