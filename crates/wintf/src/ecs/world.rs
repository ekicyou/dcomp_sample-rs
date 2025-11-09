use bevy_ecs::prelude::*;
use std::time::Instant;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Priority {
    Input,
    Update,
    Layout,
    Draw,
    RenderSurface,
    Composition,
}

/// ECSワールドのラッパー
/// 初期化ロジックや拡張機能をここに集約
pub struct EcsWorld {
    world: World,
    schedule: Schedule,
    has_systems: bool,
    // デバッグ用: フレームレート計測
    frame_count: u64,
    last_log_time: Option<Instant>,
}

impl EcsWorld {
    /// 新しいEcsWorldを作成
    pub fn new() -> Self {
        let world = World::new();

        let mut schedule = Schedule::default();
        schedule.configure_sets(
            (
                Priority::Input,
                Priority::Update,
                Priority::Layout,
                Priority::Draw,
                Priority::RenderSurface,
                Priority::Composition,
            )
                .chain(),
        );

        // ここで初期化処理を行う
        // 例: リソースの登録、システムのセットアップなど
        Self {
            world,
            schedule,
            has_systems: false,
            frame_count: 0,
            last_log_time: None,
        }
    }

    /// スケジュールへの可変参照を取得してシステムを追加
    pub fn schedule_mut(&mut self) -> &mut Schedule {
        self.has_systems = true;
        &mut self.schedule
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

        // スケジュールを実行（登録された全システムを1回実行）
        self.schedule.run(&mut self.world);

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
