use bevy_ecs::schedule::Schedule;
use bevy_ecs::world::World;
use std::time::{Duration, Instant};

/// ECSワールドのラッパー
/// 初期化ロジックや拡張機能をここに集約
pub struct EcsWorld {
    world: World,
    schedule: Schedule,
    has_systems: bool,
    last_tick: Option<Instant>,
    tick_interval: Duration,
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
            last_tick: None,
            tick_interval: Duration::from_micros(16667), // 約60fps (1/60秒)
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

        // 前回の実行から十分な時間が経過しているかチェック
        let now = Instant::now();

        if let Some(last) = self.last_tick {
            if now.duration_since(last) < self.tick_interval {
                // まだ実行するには早すぎる
                return false;
            }
        }

        // スケジュールを実行（登録された全システムを1回実行）
        self.schedule.run(&mut self.world);

        // 実行時刻を記録
        self.last_tick = Some(now);
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
