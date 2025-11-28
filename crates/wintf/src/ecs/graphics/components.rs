use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::prelude::*;
use bevy_ecs::world::DeferredWorld;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows_numerics::Vector2;

use crate::com::dcomp::DCompositionVisualExt;

/// グラフィックスリソースを使用するエンティティを宣言 + 初期化状態管理
///
/// このコンポーネントは以下の2つの役割を持つ：
/// 1. GPUリソースを使用するエンティティを宣言（存在自体がマーカー）
/// 2. 初期化/再初期化が必要かどうかの状態を世代番号で管理
///
/// `Changed<HasGraphicsResources>` でグラフィックス初期化トリガーを検出し、
/// `needs_init()` で実際に初期化が必要かを判定する。
#[derive(Component, Default, Debug, Clone, PartialEq)]
pub struct HasGraphicsResources {
    /// 初期化が必要な世代番号
    needs_init_generation: u32,
    /// 処理済みの世代番号
    processed_generation: u32,
}

impl HasGraphicsResources {
    /// 初期化をリクエスト（ダーティにする）
    ///
    /// 世代番号をインクリメントし、`Changed` フラグを発火させる。
    pub fn request_init(&mut self) {
        self.needs_init_generation = self.needs_init_generation.wrapping_add(1);
    }

    /// 初期化が必要か判定
    ///
    /// 世代番号が不一致の場合は初期化が必要。
    pub fn needs_init(&self) -> bool {
        self.needs_init_generation != self.processed_generation
    }

    /// 初期化完了をマーク（クリーンにする）
    ///
    /// 処理済み世代番号を更新し、`needs_init()` が `false` になる。
    pub fn mark_initialized(&mut self) {
        self.processed_generation = self.needs_init_generation;
    }
}

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
