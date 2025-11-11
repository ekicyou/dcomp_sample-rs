use bevy_ecs::prelude::*;
use bevy_ecs::schedule::*;
use std::time::Instant;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Priority {
    Input,
    Update,
    Layout,
    UISetup,
    Draw,
    RenderSurface,
    Composition,
}

/// ECSワールドのラッパー
/// 初期化ロジックや拡張機能をここに集約
pub struct EcsWorld {
    world: World,
    has_systems: bool,
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
        
        // Schedulesをリソースとして初期化
        world.init_resource::<Schedules>();

        // UISetupスケジュールをメインスレッド固定に設定
        {
            let mut ui_setup = Schedule::new(Priority::UISetup);
            ui_setup.set_executor_kind(ExecutorKind::SingleThreaded);
            world.resource_mut::<Schedules>().insert(ui_setup);
        }
        
        // デフォルトシステムの登録
        // ウィンドウ作成・破棄はUISetupに登録（メインスレッド固定）
        {
            let mut schedules = world.resource_mut::<Schedules>();
            schedules.add_systems(Priority::UISetup, crate::ecs::window_system::create_windows);
            schedules.add_systems(Priority::UISetup, crate::ecs::window_system::on_window_handle_added);
            schedules.add_systems(Priority::UISetup, crate::ecs::window_system::on_window_handle_removed);
        }

        Self {
            world,
            has_systems: true, // デフォルトシステムがあるのでtrue
            frame_count: 0,
            last_log_time: None,
        }
    }

    /// Schedulesリソースへのアクセスを提供
    pub fn schedules_mut(&mut self) -> Mut<Schedules> {
        self.has_systems = true;
        self.world.resource_mut::<Schedules>()
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
        let _ = self.world.try_run_schedule(Priority::Input);
        let _ = self.world.try_run_schedule(Priority::Update);
        let _ = self.world.try_run_schedule(Priority::Layout);
        let _ = self.world.try_run_schedule(Priority::UISetup);
        let _ = self.world.try_run_schedule(Priority::Draw);
        let _ = self.world.try_run_schedule(Priority::RenderSurface);
        let _ = self.world.try_run_schedule(Priority::Composition);

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
