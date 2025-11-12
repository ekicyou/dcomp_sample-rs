use bevy_ecs::prelude::*;
use bevy_ecs::schedule::*;
use bevy_ecs::system::*;
use std::time::Instant;
use windows::Win32::Foundation::HWND;

// 各プライオリティ用のScheduleLabelマーカー構造体
// 実行順序: Input → Update → Layout → UISetup → Draw → RenderSurface → Composition

/// 入力処理スケジュール
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Input;

/// 通常のロジック更新スケジュール（マルチスレッド）
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Update;

/// ウィンドウ/ウィジェットのレイアウト計算スケジュール
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Layout;

/// UIセットアップスケジュール（メインスレッド固定）
///
/// Layoutスケジュールの結果を使ってウィンドウを作成・更新する。
/// メッセージループに影響する処理（CreateWindowEx, PostQuitMessage等）を含むため、
/// SingleThreadedエグゼキュータで実行される。
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UISetup;

/// 描画コマンド生成スケジュール
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Draw;

/// レンダリングサーフェス更新スケジュール
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RenderSurface;

/// 最終合成スケジュール
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Composition;

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

        // スケジュールの登録
        {
            world.init_resource::<Schedules>();
            let mut schedules = world.resource_mut::<Schedules>();

            schedules.insert(Schedule::new(Input));
            schedules.insert(Schedule::new(Update));
            schedules.insert(Schedule::new(Layout));

            // UISetupだけメインスレッド固定
            {
                let mut sc = Schedule::new(UISetup);
                sc.set_executor_kind(ExecutorKind::SingleThreaded);
                schedules.insert(sc);
            }

            schedules.insert(Schedule::new(Draw));
            schedules.insert(Schedule::new(RenderSurface));
            schedules.insert(Schedule::new(Composition));
        }

        // デフォルトシステムの登録
        {
            let mut schedules = world.resource_mut::<Schedules>();
            schedules.add_systems(UISetup, crate::ecs::window_system::create_windows);
            // on_window_handle_addedとon_window_handle_removedはフックで代替
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

        // 各Scheduleを順番に実行
        let _ = self.world.try_run_schedule(Input);
        let _ = self.world.try_run_schedule(Update);
        let _ = self.world.try_run_schedule(Layout);
        let _ = self.world.try_run_schedule(UISetup);
        let _ = self.world.try_run_schedule(Draw);
        let _ = self.world.try_run_schedule(RenderSurface);
        let _ = self.world.try_run_schedule(Composition);

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
