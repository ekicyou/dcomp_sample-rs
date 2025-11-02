use bevy_ecs::schedule::Schedule;
use bevy_ecs::world::World;

/// ECSワールドのラッパー
/// 初期化ロジックや拡張機能をここに集約
pub struct EcsWorld {
    world: World,
    schedule: Schedule,
    has_systems: bool,
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

    /// システムを1回実行
    /// システムが実行された場合はtrueを返す
    pub fn try_tick_world(&mut self) -> bool {
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
