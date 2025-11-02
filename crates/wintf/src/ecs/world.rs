use bevy_ecs::world::World;

/// ECSワールドのラッパー
/// 初期化ロジックや拡張機能をここに集約
#[derive(Debug)]
pub struct EcsWorld {
    world: World,
}

impl EcsWorld {
    /// 新しいEcsWorldを作成
    pub fn new() -> Self {
        let world = World::new();
        // ここで初期化処理を行う
        // 例: リソースの登録、システムのセットアップなど
        Self { world }
    }

    /// 内部のWorldへの参照を取得
    pub fn world(&self) -> &World {
        &self.world
    }

    /// 内部のWorldへの可変参照を取得
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

impl Default for EcsWorld {
    fn default() -> Self {
        Self::new()
    }
}
