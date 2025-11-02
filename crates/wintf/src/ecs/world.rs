use bevy_ecs::schedule::Schedule;
use bevy_ecs::world::World;
use std::cell::RefCell;

/// ECSワールドのラッパー
/// 初期化ロジックや拡張機能をここに集約
pub struct EcsWorld {
    world: RefCell<World>,
    schedule: RefCell<Schedule>,
    has_systems: RefCell<bool>,
}

impl EcsWorld {
    /// 新しいEcsWorldを作成
    pub fn new() -> Self {
        let world = World::new();
        let schedule = Schedule::default();
        // ここで初期化処理を行う
        // 例: リソースの登録、システムのセットアップなど
        Self {
            world: RefCell::new(world),
            schedule: RefCell::new(schedule),
            has_systems: RefCell::new(false),
        }
    }

    /// スケジュールへの可変参照を取得してシステムを追加
    pub fn schedule_mut(&self) -> std::cell::RefMut<'_, Schedule> {
        *self.has_systems.borrow_mut() = true;
        self.schedule.borrow_mut()
    }

    /// 内部のWorldへの参照を取得
    pub fn world(&self) -> std::cell::Ref<'_, World> {
        self.world.borrow()
    }

    /// 内部のWorldへの可変参照を取得
    pub fn world_mut(&self) -> std::cell::RefMut<'_, World> {
        self.world.borrow_mut()
    }

    /// システムを1回だけ実行
    /// システムが実行された場合はtrueを返す
    pub fn try_tick_world(&self) -> bool {
        // システムが登録されていない場合はスキップ
        if !*self.has_systems.borrow() {
            return false;
        }

        // スケジュールを実行（登録された全システムを1回実行）
        let mut world = self.world.borrow_mut();
        let mut schedule = self.schedule.borrow_mut();
        schedule.run(&mut world);
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
