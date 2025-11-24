use bevy_ecs::prelude::*;
use bevy_ecs::schedule::*;
use bevy_ecs::system::*;
use std::time::Instant;
use windows::Win32::Foundation::HWND;

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

            schedules.insert(Schedule::new(Draw));
            schedules.insert(Schedule::new(PreRenderSurface));
            schedules.insert(Schedule::new(RenderSurface));
            schedules.insert(Schedule::new(Composition));
            schedules.insert(Schedule::new(CommitComposition));
        }

        // デフォルトシステムの登録
        {
            let mut schedules = world.resource_mut::<Schedules>();

            schedules.add_systems(UISetup, crate::ecs::window_system::create_windows);
            // on_window_handle_addedとon_window_handle_removedはフックで代替
            
            // WindowPos変更をSetWindowPosに反映（メインスレッド固定が必要）
            schedules.add_systems(UISetup, crate::ecs::graphics::apply_window_pos_changes);
            
            // WindowPos変更をSetWindowPosに反映（メインスレッド固定が必要）
            schedules.add_systems(UISetup, crate::ecs::graphics::apply_window_pos_changes);

            // Updateスケジュールにディスプレイ構成変更検知とモニター階層管理を登録
            schedules.add_systems(
                Update,
                (
                    // LayoutRoot初期化（初回のみ実行）
                    crate::ecs::layout::initialize_layout_root_system,
                    // ディスプレイ構成変更検知（initialize_layout_root_systemの後）
                    crate::ecs::layout::detect_display_change_system
                        .after(crate::ecs::layout::initialize_layout_root_system),
                    // Monitorレイアウト更新（detect_display_change_systemの後）
                    crate::ecs::layout::update_monitor_layout_system
                        .after(crate::ecs::layout::detect_display_change_system),
                    // 依存コンポーネント無効化
                    crate::ecs::graphics::invalidate_dependent_components
                        .after(crate::ecs::layout::update_monitor_layout_system),
                )
                    .chain(),
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

            // PostLayoutスケジュールにグラフィックス初期化システムを登録
            schedules.add_systems(
                PostLayout,
                (
                    crate::ecs::graphics::init_graphics_core,
                    crate::ecs::graphics::cleanup_command_list_on_reinit
                        .after(crate::ecs::graphics::init_graphics_core),
                    crate::ecs::graphics::init_window_graphics
                        .after(crate::ecs::graphics::cleanup_command_list_on_reinit),
                    // Visualリソース管理システム (新規作成・再作成)
                    crate::ecs::graphics::visual_resource_management_system
                        .after(crate::ecs::graphics::init_window_graphics),
                    crate::ecs::graphics::visual_reinit_system
                        .after(crate::ecs::graphics::visual_resource_management_system),
                    // WindowとVisualの紐付け
                    crate::ecs::graphics::window_visual_integration_system
                        .after(crate::ecs::graphics::visual_reinit_system),
                    // init_window_arrangement: Arrangementコンポーネントの初期化
                    crate::ecs::window_system::init_window_arrangement
                        .after(crate::ecs::graphics::window_visual_integration_system),
                ),
            );

            // PostLayoutスケジュールにArrangement伝播システムを登録
            schedules.add_systems(
                PostLayout,
                (
                    crate::ecs::layout::sync_simple_arrangements,
                    crate::ecs::layout::mark_dirty_arrangement_trees
                        .after(crate::ecs::layout::sync_simple_arrangements),
                    crate::ecs::layout::propagate_global_arrangements
                        .after(crate::ecs::layout::mark_dirty_arrangement_trees),
                    // Layout-to-Graphics同期システム (新規追加)
                    crate::ecs::graphics::sync_visual_from_layout_root
                        .after(crate::ecs::layout::propagate_global_arrangements),
                    crate::ecs::graphics::resize_surface_from_visual
                        .after(crate::ecs::graphics::sync_visual_from_layout_root),
                    crate::ecs::graphics::sync_window_pos
                        .after(crate::ecs::graphics::resize_surface_from_visual),
                    // apply_window_pos_changesはUISetupに移動（メインスレッド固定のため）
                    crate::ecs::layout::update_window_pos_system
                        .after(crate::ecs::graphics::sync_window_pos),
                )
                    .chain(),
            );

            // Drawスケジュールにクリーンアップシステムとウィジェット描画システムを登録
            schedules.add_systems(
                Draw,
                (
                    crate::ecs::graphics::cleanup_graphics_needs_init,
                    crate::ecs::widget::shapes::rectangle::draw_rectangles,
                    crate::ecs::widget::text::draw_labels,
                )
                    .chain(),
            );

            // PreRenderSurfaceスケジュールに変更検知システムを登録
            schedules.add_systems(PreRenderSurface, crate::ecs::graphics::mark_dirty_surfaces);

            // RenderSurfaceスケジュールに描画システムを登録
            schedules.add_systems(RenderSurface, crate::ecs::graphics::render_surface);

            // CommitCompositionスケジュールにコミットシステムを登録
            schedules.add_systems(CommitComposition, crate::ecs::graphics::commit_composition);
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
                eprintln!(
                    "[ECS] Frame rate: {:.2} fps ({} frames in {:.2}s, avg {:.2}ms/frame)",
                    fps,
                    self.frame_count,
                    elapsed.as_secs_f64(),
                    avg_frame_time
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

        // 各Scheduleを順番に実行
        let _ = self.world.try_run_schedule(Input);
        let _ = self.world.try_run_schedule(Update);
        let _ = self.world.try_run_schedule(PreLayout);
        let _ = self.world.try_run_schedule(Layout);
        let _ = self.world.try_run_schedule(PostLayout);
        let _ = self.world.try_run_schedule(UISetup);
        let _ = self.world.try_run_schedule(Draw);
        let _ = self.world.try_run_schedule(PreRenderSurface);
        let _ = self.world.try_run_schedule(RenderSurface);
        let _ = self.world.try_run_schedule(Composition);
        let _ = self.world.try_run_schedule(CommitComposition);

        true
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
