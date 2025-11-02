use bevy_ecs::schedule::Schedule;
use bevy_ecs::world::World;
use std::time::{Duration, Instant};

/// ECSワールドのラッパー
/// 初期化ロジックや拡張機能をここに集約
pub struct EcsWorld {
    world: World,
    schedule: Schedule,
    has_systems: bool,
    start_time: Option<Instant>,
    next_frame_time: Option<Instant>,
    tick_interval: Duration,
    frame_count: u64,
}

impl EcsWorld {
    /// 新しいEcsWorldを作成
    pub fn new() -> Self {
        let world = World::new();
        let schedule = Schedule::default();
        // ここで初期化処理を行う
        // 例: リソースの登録、システムのセットアップなど
        Self {
            world,
            schedule,
            has_systems: false,
            start_time: None,
            next_frame_time: None,
            tick_interval: Duration::from_micros(16667), // 約60fps (1/60秒)
            frame_count: 0,
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

    /// システムを1回だけ実行
    /// システムが実行された場合はtrueを返す
    pub fn try_tick_world(&mut self) -> bool {
        // システムが登録されていない場合はスキップ
        if !self.has_systems {
            return false;
        }

        let now = Instant::now();

        // 初回実行時に基準時刻を設定
        if self.start_time.is_none() {
            self.start_time = Some(now);
            self.next_frame_time = Some(now);
        }

        // 次のフレームタイミングに到達しているかチェック
        if let Some(next_time) = self.next_frame_time {
            if now < next_time {
                // まだ実行するには早すぎる
                return false;
            }
        }

        // スケジュールを実行（登録された全システムを1回実行）
        self.schedule.run(&mut self.world);

        // 次のフレームタイミングを計算（基準時刻からの経過フレーム数で計算）
        self.frame_count += 1;
        if let Some(start) = self.start_time {
            // 理想的な次フレームの時刻 = 開始時刻 + (フレーム数 × フレーム間隔)
            self.next_frame_time = Some(start + self.tick_interval * (self.frame_count as u32));
        }

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
