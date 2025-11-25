use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::prelude::*;
use bevy_ecs::world::DeferredWorld;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows_numerics::Vector2;

use crate::com::dcomp::DCompositionVisualExt;

/// グラフィックスリソースを使用するエンティティを宣言（静的マーカー）
#[derive(Component, Default)]
pub struct HasGraphicsResources;

/// 初期化が必要な状態を示す動的マーカー
#[derive(Component, Default)]
pub struct GraphicsNeedsInit;

#[derive(Debug)]
struct WindowGraphicsInner {
    pub target: IDCompositionTarget,
    pub device_context: ID2D1DeviceContext,
}

/// ウィンドウごとのグラフィックスリソース
#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct WindowGraphics {
    inner: Option<WindowGraphicsInner>,
    generation: u32,
}

unsafe impl Send for WindowGraphics {}
unsafe impl Sync for WindowGraphics {}

impl WindowGraphics {
    pub fn new(target: IDCompositionTarget, device_context: ID2D1DeviceContext) -> Self {
        Self {
            inner: Some(WindowGraphicsInner {
                target,
                device_context,
            }),
            generation: 0,
        }
    }

    pub fn invalidate(&mut self) {
        self.inner = None;
    }

    pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }

    pub fn increment_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn get_target(&self) -> Option<&IDCompositionTarget> {
        self.inner.as_ref().map(|i| &i.target)
    }

    pub fn get_dc(&self) -> Option<&ID2D1DeviceContext> {
        self.inner.as_ref().map(|i| &i.device_context)
    }
}

/// ウィンドウのルートビジュアルノード (R9: parent_visualキャッシュ方式)
#[derive(Component)]
#[component(on_remove = on_visual_graphics_remove)]
pub struct VisualGraphics {
    inner: Option<IDCompositionVisual3>,
    /// 親Visual参照（RemoveVisual用にキャッシュ）
    /// 階層同期時にAddVisualと同時に設定される
    parent_visual: Option<IDCompositionVisual3>,
}

// on_remove フック: 親Visualから自分を削除
fn on_visual_graphics_remove(world: DeferredWorld, hook: HookContext) {
    // 親Visualから自分を削除
    // エラーは無視（親が先に削除されている場合など）
    if let Some(vg) = world.get::<VisualGraphics>(hook.entity) {
        if let (Some(parent), Some(visual)) = (&vg.parent_visual, &vg.inner) {
            let _ = parent.remove_visual(visual); // エラー無視
        }
    }
}

unsafe impl Send for VisualGraphics {}
unsafe impl Sync for VisualGraphics {}

impl std::fmt::Debug for VisualGraphics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VisualGraphics")
            .field("inner", &self.inner.is_some())
            .field("parent_visual", &self.parent_visual.is_some())
            .finish()
    }
}

impl VisualGraphics {
    pub fn new(visual: IDCompositionVisual3) -> Self {
        Self {
            inner: Some(visual),
            parent_visual: None,
        }
    }

    /// 親Visualを指定してVisualGraphicsを作成
    pub fn new_with_parent(
        visual: IDCompositionVisual3,
        parent_visual: Option<IDCompositionVisual3>,
    ) -> Self {
        Self {
            inner: Some(visual),
            parent_visual,
        }
    }

    pub fn invalidate(&mut self) {
        self.inner = None;
        self.parent_visual = None;
    }

    pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }

    /// IDCompositionVisual3への参照を取得する
    pub fn visual(&self) -> Option<&IDCompositionVisual3> {
        self.inner.as_ref()
    }

    /// 親Visualへの参照を取得する
    pub fn parent_visual(&self) -> Option<&IDCompositionVisual3> {
        self.parent_visual.as_ref()
    }

    /// 親Visualを設定/更新する
    pub fn set_parent_visual(&mut self, parent: Option<IDCompositionVisual3>) {
        self.parent_visual = parent;
    }
}

/// ウィンドウの描画サーフェス
#[derive(Component, Debug, Default)]
#[component(on_add = on_surface_graphics_changed, on_replace = on_surface_graphics_changed)]
pub struct SurfaceGraphics {
    inner: Option<IDCompositionSurface>,
    pub size: (u32, u32),
}

unsafe impl Send for SurfaceGraphics {}
unsafe impl Sync for SurfaceGraphics {}

impl SurfaceGraphics {
    pub fn new(surface: IDCompositionSurface, size: (u32, u32)) -> Self {
        Self {
            inner: Some(surface),
            size,
        }
    }

    pub fn invalidate(&mut self) {
        self.inner = None;
        self.size = (0, 0);
    }

    pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }

    /// IDCompositionSurfaceへの参照を取得
    pub fn surface(&self) -> Option<&IDCompositionSurface> {
        self.inner.as_ref()
    }
}

struct SafeInsertSurfaceUpdateRequested {
    entity: Entity,
}

impl Command for SafeInsertSurfaceUpdateRequested {
    fn apply(self, world: &mut World) {
        if let Ok(mut entity_mut) = world.get_entity_mut(self.entity) {
            entity_mut.insert(SurfaceUpdateRequested);
        }
    }
}

fn on_surface_graphics_changed(mut world: DeferredWorld, context: HookContext) {
    let mut commands = world.commands();
    commands.queue(SafeInsertSurfaceUpdateRequested {
        entity: context.entity,
    });
}

/// 描画更新が必要なサーフェスを示すマーカーコンポーネント
#[derive(Component, Default)]
pub struct SurfaceUpdateRequested;

/// 論理的なVisualコンポーネント
/// サイズ情報はArrangementから取得する（Single Source of Truth）
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Visual {
    pub is_visible: bool,
    pub opacity: f32,
    pub transform_origin: Vector2,
}

impl Default for Visual {
    fn default() -> Self {
        Self {
            is_visible: true,
            opacity: 1.0,
            transform_origin: Vector2::default(),
        }
    }
}
