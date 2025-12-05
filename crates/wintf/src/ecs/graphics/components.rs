use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::prelude::*;
use bevy_ecs::world::DeferredWorld;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows_numerics::Vector2;

use crate::com::dcomp::DCompositionVisualExt;

/// GPUリソースを使用するエンティティを宣言するマーカーコンポーネント
///
/// このコンポーネントは以下の役割を持つ：
/// 1. GPUリソースを使用するエンティティを宣言（存在自体がマーカー）
/// 2. `Changed<HasGraphicsResources>` でGPUリソース再初期化をトリガー
///
/// デバイスロスト時は `set_changed()` を呼び出すことで、
/// 各GPUリソースコンポーネント（VisualGraphics, WindowGraphics等）の
/// 再初期化システムをトリガーする。
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HasGraphicsResources;

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

impl Default for VisualGraphics {
    /// 空のVisualGraphicsを作成（GPUリソースなし）
    /// Changed<VisualGraphics> + !is_valid() でシステムが検知してGPUリソースを作成
    fn default() -> Self {
        Self {
            inner: None,
            parent_visual: None,
        }
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
///
/// Note: 旧on_add/on_replaceフックは廃止され、
/// mark_dirty_surfacesシステムでAdded<SurfaceGraphics>として検出される
#[derive(Component, Debug, Default)]
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

    /// Surfaceを設定（既存のコンポーネントを直接更新）
    ///
    /// commands.insert()の代わりにこのメソッドを使用することで、
    /// 同一フレーム内で変更が即座に反映される。
    /// サイズが異なる場合のみ更新し、Changedフラグを適切に管理。
    pub fn set_surface(&mut self, surface: IDCompositionSurface, size: (u32, u32)) {
        self.inner = Some(surface);
        self.size = size;
    }

    /// Surfaceをクリア（invalidateと同じだが意図を明確に）
    ///
    /// commands.remove()の代わりにこのメソッドを使用することで、
    /// コンポーネント自体は残したまま内容だけをクリアする。
    pub fn clear(&mut self) {
        self.inner = None;
        self.size = (0, 0);
    }
}

/// SurfaceGraphicsがダーティ（再描画が必要）であることを示すコンポーネント
///
/// マーカーコンポーネント `SurfaceUpdateRequested` の置き換え。
/// `Changed<SurfaceGraphicsDirty>` パターンにより、アーキタイプ変更を排除し、
/// 同一スケジュール内での即時伝搬を実現する。
#[derive(Component, Default, Debug, Clone, PartialEq)]
pub struct SurfaceGraphicsDirty {
    /// 最後に描画をリクエストしたフレーム番号
    pub requested_frame: u64,
}

/// 論理的なVisualコンポーネント
/// サイズ情報はArrangementから取得する（Single Source of Truth）
///
/// # ライフタイムイベント
/// - `on_add`: `Arrangement::default()`を自動挿入
///   - これにより`Arrangement`の`on_add`が連鎖的に`GlobalArrangement`と`ArrangementTreeChanged`を挿入
#[derive(Component, Debug, Clone, PartialEq)]
#[component(on_add = on_visual_add)]
pub struct Visual {
    pub is_visible: bool,
    pub opacity: f32,
    pub transform_origin: Vector2,
}

/// Visualコンポーネントが追加されたときに呼ばれるフック
/// - Arrangementを自動挿入（既に存在する場合はスキップ）
/// - VisualGraphics::default()を自動挿入（GPUリソースはシステムで作成）
/// - SurfaceGraphics::default()とSurfaceGraphicsDirty::default()を自動挿入
/// - BrushInheritマーカーを自動挿入（継承解決用）
fn on_visual_add(mut world: DeferredWorld, context: HookContext) {
    let entity = context.entity;

    // 先に全てのチェックを行う（借用の問題を回避）
    let needs_arrangement = world
        .get::<crate::ecs::layout::Arrangement>(entity)
        .is_none();
    let needs_visual_graphics = world.get::<VisualGraphics>(entity).is_none();
    let needs_surface_graphics = world.get::<SurfaceGraphics>(entity).is_none();
    let needs_surface_dirty = world.get::<SurfaceGraphicsDirty>(entity).is_none();
    let needs_brush_inherit = world
        .get::<crate::ecs::widget::BrushInherit>(entity)
        .is_none();

    // コマンドを発行
    let mut cmds = world.commands();
    let mut entity_cmds = cmds.entity(entity);

    // Arrangementがまだ存在しない場合のみ挿入
    // Arrangementのon_addフックがGlobalArrangementとArrangementTreeChangedを自動挿入する
    if needs_arrangement {
        entity_cmds.insert(crate::ecs::layout::Arrangement::default());
    }

    // VisualGraphics::default()を挿入（GPUリソースなし）
    // Changed<VisualGraphics> + !is_valid() でシステムがGPUリソースを作成
    if needs_visual_graphics {
        entity_cmds.insert(VisualGraphics::default());
    }

    // SurfaceGraphicsとSurfaceGraphicsDirtyも事前配置
    if needs_surface_graphics {
        entity_cmds.insert(SurfaceGraphics::default());
    }
    if needs_surface_dirty {
        entity_cmds.insert(SurfaceGraphicsDirty::default());
    }

    // BrushInheritマーカーを挿入（継承解決システムで処理される）
    // Note: Brushesコンポーネントは挿入しない（オプショナル設計）
    if needs_brush_inherit {
        entity_cmds.insert(crate::ecs::widget::BrushInherit);
    }
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

// ========== Surface Creation Statistics ==========

/// デバッグビルド時のSurface生成統計リソース
///
/// Surface生成の最適化状況を把握するための統計情報を収集する。
/// 開発者がパフォーマンスチューニングやデバッグを行う際に使用。
///
/// # Requirements
/// - Req 5.3: デバッグビルド時のSurface生成統計
///
/// # Usage
/// ```ignore
/// let stats = world.resource::<SurfaceCreationStats>();
/// eprintln!("Created: {}, Skipped: {}", stats.created_count, stats.skipped_count);
/// ```
#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub struct SurfaceCreationStats {
    /// 作成されたSurfaceの累計数
    pub created_count: u64,
    /// スキップされたSurface作成の累計数（CommandList不在等）
    pub skipped_count: u64,
    /// 削除されたSurfaceの累計数
    pub deleted_count: u64,
    /// リサイズされたSurfaceの累計数
    pub resize_count: u64,
}

impl SurfaceCreationStats {
    /// Surface作成を記録
    pub fn record_created(&mut self) {
        self.created_count += 1;
    }

    /// スキップを記録
    pub fn record_skipped(&mut self) {
        self.skipped_count += 1;
    }

    /// 削除を記録
    pub fn record_deleted(&mut self) {
        self.deleted_count += 1;
    }

    /// リサイズを記録
    pub fn record_resized(&mut self) {
        self.resize_count += 1;
    }
}
