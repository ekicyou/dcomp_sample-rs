use bevy_ecs::message::Messages;
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::*;
use bevy_ecs::system::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use tracing::trace;
use windows::Win32::Foundation::HWND;

// ============================================================
// VSYNC優先レンダリング用トレイト
// WndProcからRefCell<EcsWorld>を安全に借用してtickを実行する。
// ============================================================

/// VSYNC駆動のtick実行を提供するトレイト
///
/// `Rc<RefCell<EcsWorld>>`に実装され、WndProcから安全にtickを呼び出せる。
/// 借用失敗時（再入時）は安全にスキップしてfalseを返す。
///
/// # 設計意図
/// - WndProc（特にWM_WINDOWPOSCHANGED）からのVSYNC駆動tick実行を可能にする
/// - モーダルループ（ウィンドウドラッグ等）中でも描画を継続する
/// - RefCellの借用状態を確認し、再入時は安全にスキップする
///
/// # 拡張ポイント
/// 他のモーダルループ関連メッセージで同様の問題が発見された場合、
/// 該当メッセージ処理でこのトレイトのメソッドを呼び出すだけで対応可能。
pub trait VsyncTick {
    /// VSYNCカウンターの変化を検知し、必要に応じてworld tickを実行
    ///
    /// # Returns
    /// - `true`: tickが実行された
    /// - `false`: tickがスキップされた（借用失敗またはカウンター変化なし）
    fn try_tick_on_vsync(&self) -> bool;
}

impl VsyncTick for Rc<RefCell<EcsWorld>> {
    fn try_tick_on_vsync(&self) -> bool {
        // RefCellの借用を試みる
        // 既に借用されている場合（再入時）は安全にスキップ
        let result = match self.try_borrow_mut() {
            Ok(mut world) => {
                let result = world.try_tick_on_vsync();

                // デバッグビルドのみ: WndProc経由のtick回数をカウント
                #[cfg(debug_assertions)]
                if result {
                    use crate::win_thread_mgr::DEBUG_WNDPROC_TICK_COUNT;
                    use std::sync::atomic::Ordering;
                    DEBUG_WNDPROC_TICK_COUNT.fetch_add(1, Ordering::Relaxed);
                }

                result
            }
            Err(_) => {
                // 借用失敗（再入時）- 安全にスキップ
                // これは正常な動作であり、エラーではない
                false
            }
        };
        // world借用スコープ終了

        // World借用解放後にSetWindowPosコマンドをフラッシュ
        // これにより、apply_window_pos_changesでキューに追加されたコマンドが
        // 安全に実行される（WM_WINDOWPOSCHANGEDがWorldを借用しても競合しない）
        crate::ecs::window::flush_window_pos_commands();

        result
    }
}

/// フレームカウンタリソース
#[derive(Resource, Default, Debug)]
pub struct FrameCount(pub u32);

// 各プライオリティ用のScheduleLabelマーカー構造体
// 実行順序: Input → Update → PreLayout → Layout → PostLayout → UISetup → Draw → Render → RenderSurface → Composition → CommitComposition

/// 入力処理スケジュール
///
/// キーボード・マウス・タッチ等の入力イベントを処理する。
/// マルチスレッド実行可能。
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Input;

/// 通常のロジック更新スケジュール
///
/// アプリケーションロジック、状態更新、アニメーション計算等を行う。
/// マルチスレッド実行可能。
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Update;

/// レイアウト計算前スケジュール
///
/// レイアウト計算の準備処理（サイズ制約の設定等）を行う。
/// マルチスレッド実行可能。
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PreLayout;

/// レイアウト計算スケジュール
///
/// ウィンドウ/ウィジェットのサイズと位置を計算する。
/// マルチスレッド実行可能。
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Layout;

/// レイアウト計算後スケジュール
///
/// レイアウト結果を使った処理（グラフィックスリソース作成等）を行う。
/// マルチスレッド実行可能。
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PostLayout;

/// UIセットアップスケジュール（メインスレッド固定）
///
/// Win32 APIを使用したウィンドウ作成・破棄等のUIスレッド専用処理を行う。
/// CreateWindowEx, DestroyWindow, PostQuitMessage等のメッセージループに影響する処理を含む。
/// SingleThreadedエグゼキュータで実行される。
///
/// **重要**: UIスレッド固定が必要な処理のみをここに配置すること。
/// DirectComposition/Direct2D等のグラフィックスAPI呼び出しは通常マルチスレッド対応なので、
/// PostLayoutやDrawで実行すべき。
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UISetup;

/// グラフィックスセットアップスケジュール
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct GraphicsSetup;

/// 描画コマンド生成スケジュール
///
/// ID2D1CommandListを使用した描画コマンドリストを作成する。
/// 実際の描画は行わず、描画命令をバッファに記録するだけ。
/// マルチスレッド実行可能（各ウィンドウのCommandListを並列作成可能）。
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Draw;

/// サーフェス更新前スケジュール
///
/// IDCompositionSurfaceへの更新前に行う。
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PreRenderSurface;

/// サーフェス更新スケジュール
///
/// IDCompositionSurfaceへの描画を実行する。
/// BeginDraw/EndDrawでSurfaceを取得し、ID2D1DeviceContextで描画を行う。
/// CommandListがある場合はDrawImage()で再生する。
/// マルチスレッド実行可能（各Surfaceは独立）。
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RenderSurface;

/// 最終合成スケジュール
///
/// IDCompositionVisualの操作（Transform, Opacity, Effect設定等）を行う。
/// ビジュアルツリーの構築・更新、アニメーション設定等。
/// マルチスレッド実行可能。
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Composition;

/// DirectCompositionコミットスケジュール
///
/// IDCompositionDevice3::Commit()を呼び出し、すべてのビジュアル変更を確定する。
/// このスケジュールは常にワールドスケジュールの最後に実行される。
/// マルチスレッド実行可能（Commit()はスレッドセーフ）。
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CommitComposition;

/// フレーム終了時クリーンアップスケジュール
///
/// CommitComposition の後に実行される。
/// - 一時的なマーカーコンポーネント（MouseLeave等）の除去
/// - 1フレームのみ有効な状態（DoubleClick, Wheel等）のリセット
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FrameFinalize;

/// ECSワールドのラッパー
/// 初期化ロジックや拡張機能をここに集約
pub struct EcsWorld {
    world: World,
    has_systems: bool,
    message_window: Option<HWND>,
    // デバッグ用: フレームレート計測
    frame_count: u64,
    last_log_time: Option<Instant>,
}

impl EcsWorld {
    /// 新しいEcsWorldを作成
    pub fn new() -> Self {
        let mut world = World::new();

        // リソースの初期化
        world.insert_resource(crate::ecs::app::App::new());
        world.insert_resource(FrameCount::default());
        world.insert_resource(crate::ecs::layout::taffy::TaffyLayoutResource::default());
        // Surface生成統計リソース（Req 5.3）
        world.insert_resource(crate::ecs::graphics::SurfaceCreationStats::default());

        // WicCore初期化（Device Lostの影響を受けない独立リソース）
        if let Ok(wic_core) = crate::ecs::widget::bitmap_source::WicCore::new() {
            world.insert_resource(wic_core);
        }

        // FrameTime初期化（FILETIMEベースのフレーム時刻）
        world.insert_resource(crate::ecs::graphics::FrameTime::new());

        // WintfTaskPool初期化（非同期タスク実行基盤）
        world.insert_resource(crate::ecs::widget::bitmap_source::WintfTaskPool::new());

        // LayoutRootとMonitor階層を初期化（Window spawnより前に必要）
        crate::ecs::layout::initialize_layout_root(&mut world);

        // ドラッグ累積器の登録
        world.insert_resource(crate::ecs::drag::DragAccumulatorResource::new());

        // イベントの登録
        world.init_resource::<Messages<crate::ecs::drag::DragStartEvent>>();
        world.init_resource::<Messages<crate::ecs::drag::DragEvent>>();
        world.init_resource::<Messages<crate::ecs::drag::DragEndEvent>>();

        // スケジュールの登録
        {
            world.init_resource::<Schedules>();
            let mut schedules = world.resource_mut::<Schedules>();

            schedules.insert(Schedule::new(Input));
            schedules.insert(Schedule::new(Update));
            schedules.insert(Schedule::new(PreLayout));
            schedules.insert(Schedule::new(Layout));
            schedules.insert(Schedule::new(PostLayout));

            // UISetupだけメインスレッド固定
            {
                let mut sc = Schedule::new(UISetup);
                sc.set_executor_kind(ExecutorKind::SingleThreaded);
                schedules.insert(sc);
            }

            schedules.insert(Schedule::new(GraphicsSetup));
            schedules.insert(Schedule::new(Draw));
            schedules.insert(Schedule::new(PreRenderSurface));
            schedules.insert(Schedule::new(RenderSurface));
            schedules.insert(Schedule::new(Composition));
            schedules.insert(Schedule::new(CommitComposition));
            schedules.insert(Schedule::new(FrameFinalize));
        }

        // デフォルトシステムの登録
        {
            let mut schedules = world.resource_mut::<Schedules>();

            // Inputスケジュール: 非同期タスク完了コマンドのドレイン
            schedules.add_systems(
                Input,
                crate::ecs::widget::bitmap_source::systems::drain_task_pool_commands,
            );

            // Inputスケジュール: ポインターイベントディスパッチ
            // transfer_buffers_to_world()でWorldに直接データ投入済み
            schedules.add_systems(
                Input,
                crate::ecs::pointer::dispatch_pointer_events
                    .after(crate::ecs::widget::bitmap_source::systems::drain_task_pool_commands),
            );

            // 注: process_pointer_buffersは廃止
            // WndProcスレッドのthread_localバッファは、try_tick_world()内の
            // transfer_buffers_to_world()で直接Worldに転送される

            // Inputスケジュール: ドラッグイベントディスパッチ
            schedules.add_systems(
                Input,
                crate::ecs::drag::dispatch_drag_events
                    .after(crate::ecs::pointer::dispatch_pointer_events),
            );

            // Inputスケジュール: ドラッグ状態クリーンアップ（dispatch_drag_eventsの後）
            schedules.add_systems(
                Input,
                crate::ecs::drag::cleanup_drag_state.after(crate::ecs::drag::dispatch_drag_events),
            );

            // Inputスケジュール: ポインターデバッグ監視（デバッグビルドのみ）
            #[cfg(debug_assertions)]
            schedules.add_systems(
                Input,
                (
                    crate::ecs::pointer::debug_pointer_state_changes,
                    crate::ecs::pointer::debug_pointer_leave,
                )
                    .after(crate::ecs::pointer::dispatch_pointer_events),
            );

            // UISetupスケジュール：ウィンドウ作成とWindowPos反映
            schedules.add_systems(UISetup, crate::ecs::window_system::create_windows);
            // on_window_handle_addedとon_window_handle_removedはフックで代替

            // WindowPos変更をSetWindowPosに反映（メインスレッド固定が必要）
            schedules.add_systems(UISetup, crate::ecs::graphics::apply_window_pos_changes);

            // Updateスケジュールにディスプレイ構成変更検知とモニター階層管理を登録
            schedules.add_systems(
                Update,
                (
                    // ディスプレイ構成変更検知
                    crate::ecs::layout::detect_display_change_system,
                    // Monitorレイアウト更新（detect_display_change_systemの後）
                    crate::ecs::layout::update_monitor_layout_system
                        .after(crate::ecs::layout::detect_display_change_system),
                    // 依存コンポーネント無効化
                    crate::ecs::graphics::invalidate_dependent_components
                        .after(crate::ecs::layout::update_monitor_layout_system),
                    // Typewriter更新（アニメーション進行）
                    crate::ecs::widget::text::update_typewriters
                        .after(crate::ecs::graphics::invalidate_dependent_components),
                )
                    .chain(),
            );

            // PreLayoutスケジュール: GraphicsCore初期化とVisual作成
            // Phase 6: VisualはPreLayoutで早期作成、SurfaceはDrawで遅延作成
            schedules.add_systems(
                PreLayout,
                (
                    crate::ecs::graphics::init_graphics_core,
                    // Visualリソース作成（Surfaceは作成しない）
                    // Changed<VisualGraphics> + !is_valid() で初期化と再初期化を統一処理
                    crate::ecs::graphics::visual_resource_management_system
                        .after(crate::ecs::graphics::init_graphics_core),
                    // Visual階層同期（parent_visual==Noneで未同期を検出）
                    crate::ecs::graphics::visual_hierarchy_sync_system
                        .after(crate::ecs::graphics::visual_resource_management_system),
                ),
            );

            // Layoutスケジュールにtaffyレイアウトシステムを登録
            schedules.add_systems(
                Layout,
                (
                    crate::ecs::layout::build_taffy_styles_system,
                    crate::ecs::layout::sync_taffy_tree_system
                        .after(crate::ecs::layout::build_taffy_styles_system),
                    crate::ecs::layout::compute_taffy_layout_system
                        .after(crate::ecs::layout::sync_taffy_tree_system),
                    crate::ecs::layout::update_arrangements_system
                        .after(crate::ecs::layout::compute_taffy_layout_system),
                    crate::ecs::layout::cleanup_removed_entities_system,
                )
                    .chain(),
            );

            // PostLayoutスケジュール: 論理計算系（Arrangement伝播まで）
            // Note: Arrangementは各コンポーネントのon_addフックで自動挿入されるため、
            //       init_window_arrangementシステムは廃止されました。
            schedules.add_systems(
                PostLayout,
                (
                    crate::ecs::layout::sync_window_arrangement_from_window_pos,
                    crate::ecs::layout::sync_simple_arrangements,
                    crate::ecs::layout::mark_dirty_arrangement_trees
                        .after(crate::ecs::layout::sync_simple_arrangements),
                    crate::ecs::layout::propagate_global_arrangements
                        .after(crate::ecs::layout::mark_dirty_arrangement_trees),
                    crate::ecs::layout::window_pos_sync_system
                        .after(crate::ecs::layout::propagate_global_arrangements),
                )
                    .chain(),
            );

            // GraphicsSetupスケジュール: グラフィックスリソース系
            // UISetupの後に実行され、WindowHandleが利用可能
            // Note: sync_surface_from_arrangementは廃止（deferred_surface_creation_systemに統合）
            schedules.add_systems(
                GraphicsSetup,
                (
                    crate::ecs::graphics::init_window_graphics,
                    crate::ecs::graphics::window_visual_integration_system
                        .after(crate::ecs::graphics::init_window_graphics),
                )
                    .chain(),
            );

            // Drawスケジュールにクリーンアップシステムとウィジェット描画システムを登録
            // Surface生成とクリーンアップを統合管理
            // Brush継承解決を最初に実行し、その後ウィジェット描画
            schedules.add_systems(
                Draw,
                (
                    // Brush継承解決（描画システムより前に実行）
                    crate::ecs::graphics::resolve_inherited_brushes,
                    crate::ecs::widget::shapes::rectangle::draw_rectangles
                        .after(crate::ecs::graphics::resolve_inherited_brushes),
                    crate::ecs::widget::text::draw_labels
                        .after(crate::ecs::graphics::resolve_inherited_brushes),
                    // Typewriter: Arrangement変更で無効化 → LayoutCache初期化 → 描画の順
                    crate::ecs::widget::text::invalidate_typewriter_layout_on_arrangement_change
                        .after(crate::ecs::graphics::resolve_inherited_brushes),
                    crate::ecs::widget::text::init_typewriter_layout
                        .after(crate::ecs::widget::text::invalidate_typewriter_layout_on_arrangement_change),
                    crate::ecs::widget::text::draw_typewriters
                        .after(crate::ecs::widget::text::init_typewriter_layout),
                    // 空トーク時の背景描画（draw_typewritersの後）
                    crate::ecs::widget::text::draw_typewriter_backgrounds
                        .after(crate::ecs::widget::text::draw_typewriters),
                    crate::ecs::widget::bitmap_source::draw_bitmap_sources
                        .after(crate::ecs::graphics::resolve_inherited_brushes),
                    // αマスク生成（draw_bitmap_sourcesの後、BitmapSourceResource追加検出時に実行）
                    crate::ecs::widget::bitmap_source::generate_alpha_mask_system
                        .after(crate::ecs::widget::bitmap_source::draw_bitmap_sources),
                    // 遅延Surface作成（GraphicsCommandList存在時、GlobalArrangementベース）
                    crate::ecs::graphics::deferred_surface_creation_system
                        .after(crate::ecs::widget::bitmap_source::generate_alpha_mask_system),
                    // GraphicsCommandList削除時のSurface解放（Req 1.3, 1.4）
                    crate::ecs::graphics::cleanup_surface_on_commandlist_removed
                        .after(crate::ecs::graphics::deferred_surface_creation_system),
                )
                    .chain(),
            );

            // PreRenderSurfaceスケジュールに変更検知システムを登録
            schedules.add_systems(PreRenderSurface, crate::ecs::graphics::mark_dirty_surfaces);

            // RenderSurfaceスケジュールに描画システムを登録
            schedules.add_systems(RenderSurface, crate::ecs::graphics::render_surface);

            // Compositionスケジュールに Visual プロパティ同期システムを登録
            // visual_property_sync_systemはArrangementに依存するのでレイアウト後に実行
            // Offset と Opacity を一括で同期
            schedules.add_systems(
                Composition,
                crate::ecs::graphics::visual_property_sync_system,
            );

            // CommitCompositionスケジュールにコミットシステムを登録
            schedules.add_systems(CommitComposition, crate::ecs::graphics::commit_composition);

            // FrameFinalizeスケジュール: 一時的ポインター状態クリア
            schedules.add_systems(
                FrameFinalize,
                crate::ecs::pointer::clear_transient_pointer_state,
            );

            // FrameFinalizeスケジュール: Messagesの更新
            schedules.add_systems(
                FrameFinalize,
                (
                    |world: &mut World| {
                        world
                            .resource_mut::<Messages<crate::ecs::drag::DragStartEvent>>()
                            .update()
                    },
                    |world: &mut World| {
                        world
                            .resource_mut::<Messages<crate::ecs::drag::DragEvent>>()
                            .update()
                    },
                    |world: &mut World| {
                        world
                            .resource_mut::<Messages<crate::ecs::drag::DragEndEvent>>()
                            .update()
                    },
                ),
            );
        }

        Self {
            world,
            has_systems: true, // デフォルトシステムがあるのでtrue
            message_window: None,
            frame_count: 0,
            last_log_time: None,
        }
    }

    /// メッセージウィンドウのHWNDを設定
    pub fn set_message_window(&mut self, hwnd: HWND) {
        self.message_window = Some(hwnd);
        // Appリソースにもメッセージウィンドウを設定
        if let Some(mut app) = self.world.get_resource_mut::<crate::ecs::app::App>() {
            app.set_message_window(hwnd);
        }
    }

    /// メッセージウィンドウのHWNDを取得
    pub fn message_window(&self) -> Option<HWND> {
        self.message_window
    }

    /// Schedulesリソースへのアクセスを提供
    pub fn schedules_mut(&mut self) -> Mut<'_, Schedules> {
        self.has_systems = true;
        self.world.resource_mut::<Schedules>()
    }

    /// 指定したスケジュールにシステムを追加
    ///
    /// # Example
    /// ```ignore
    /// world.add_systems(Update, my_system);
    /// ```
    pub fn add_systems<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) {
        self.has_systems = true;
        self.world
            .resource_mut::<Schedules>()
            .add_systems(schedule, systems);
    }

    /// 内部のWorldへの参照を取得
    pub fn world(&self) -> &World {
        &self.world
    }

    /// 内部のWorldへの可変参照を取得
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// 非同期タスクを生成（WintfTaskPool経由）
    ///
    /// タスク内で`CommandSender`を使ってECSコマンドを送信できる。
    /// コマンドはInputスケジュールで自動的にWorldに適用される。
    ///
    /// # Example
    /// ```ignore
    /// ecs_world.spawn(|tx| async move {
    ///     let result = some_async_work().await;
    ///     let cmd: BoxedCommand = Box::new(move |world: &mut World| {
    ///         // worldを操作
    ///     });
    ///     let _ = tx.send(cmd);
    /// });
    /// ```
    pub fn spawn<F, Fut>(&self, f: F)
    where
        F: FnOnce(crate::ecs::widget::bitmap_source::CommandSender) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        if let Some(task_pool) = self
            .world
            .get_resource::<crate::ecs::widget::bitmap_source::WintfTaskPool>()
        {
            task_pool.spawn(f);
        }
    }

    /// フレームレート計測とログ出力（デバッグ用）
    /// 10秒ごとにフレームレートをログ出力
    fn measure_and_log_framerate(&mut self) {
        self.frame_count += 1;

        let now = Instant::now();
        if let Some(last_log) = self.last_log_time {
            let elapsed = now.duration_since(last_log);
            if elapsed.as_secs() >= 10 {
                let fps = self.frame_count as f64 / elapsed.as_secs_f64();
                let avg_frame_time = elapsed.as_secs_f64() * 1000.0 / self.frame_count as f64;
                trace!(
                    fps,
                    frame_count = self.frame_count,
                    elapsed_secs = format_args!("{:.2}", elapsed.as_secs_f64()),
                    avg_frame_time_ms = format_args!("{:.2}", avg_frame_time),
                    "Frame rate measurement"
                );
                self.frame_count = 0;
                self.last_log_time = Some(now);
            }
        } else {
            // 初回
            self.last_log_time = Some(now);
        }
    }

    /// システムを1回実行
    /// システムが実行された場合はtrueを返す
    pub fn try_tick_world(&mut self) -> bool {
        // デバッグ: フレームレート計測（不要になったらコメントアウト）
        self.measure_and_log_framerate();

        // システムが登録されていない場合はスキップ
        if !self.has_systems {
            return false;
        }

        // FrameCountをインクリメント
        if let Some(mut frame_count) = self.world.get_resource_mut::<FrameCount>() {
            frame_count.0 += 1;
        }

        // WndProcスレッドのthread_localバッファからWorldに直接データを投入
        // これにより、マルチスレッドで実行されるシステムでもデータにアクセス可能になる
        crate::ecs::pointer::transfer_buffers_to_world(&mut self.world);

        // 各Scheduleを順番に実行
        let _ = self.world.try_run_schedule(Input);
        let _ = self.world.try_run_schedule(Update);
        let _ = self.world.try_run_schedule(PreLayout);
        let _ = self.world.try_run_schedule(Layout);
        let _ = self.world.try_run_schedule(PostLayout);
        let _ = self.world.try_run_schedule(UISetup);
        let _ = self.world.try_run_schedule(GraphicsSetup);
        let _ = self.world.try_run_schedule(Draw);
        let _ = self.world.try_run_schedule(PreRenderSurface);
        let _ = self.world.try_run_schedule(RenderSurface);
        let _ = self.world.try_run_schedule(Composition);
        let _ = self.world.try_run_schedule(CommitComposition);
        let _ = self.world.try_run_schedule(FrameFinalize);

        // Layout スケジュール実行後のタイミングで NCHITTEST キャッシュをクリア
        crate::ecs::nchittest_cache::clear_nchittest_cache();

        true
    }

    /// VSYNCカウンターの変化を検知し、必要に応じてworld tickを実行
    ///
    /// この関数は`run()`のWM_VSYNC処理と`WndProc`のWM_WINDOWPOSCHANGED処理の
    /// 両方から呼び出され、重複実行を防ぐ。
    ///
    /// # Returns
    /// - `true`: tickが実行された
    /// - `false`: tickがスキップされた（カウンター変化なし）
    ///
    /// # 拡張ポイント
    /// 他のモーダルループ関連メッセージ（WM_ENTERSIZEMOVEなど）で同様の問題が
    /// 発見された場合、該当メッセージ処理でこの関数を呼び出すだけで対応可能。
    pub fn try_tick_on_vsync(&mut self) -> bool {
        use crate::win_thread_mgr::{LAST_VSYNC_TICK, VSYNC_TICK_COUNT};
        use std::sync::atomic::Ordering;

        // 現在のVSYNCカウンターを取得
        let current_tick = VSYNC_TICK_COUNT.load(Ordering::Relaxed);
        let last_tick = LAST_VSYNC_TICK.load(Ordering::Relaxed);

        // カウンターが変化していなければスキップ
        if current_tick == last_tick {
            return false;
        }

        // 前回処理値を更新（try_tick_world()呼び出し前に更新することで、
        // 再入時の重複tickを防止）
        LAST_VSYNC_TICK.store(current_tick, Ordering::Relaxed);

        // world tickを実行
        self.try_tick_world()
    }
}

impl Default for EcsWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for EcsWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EcsWorld").finish_non_exhaustive()
    }
}
