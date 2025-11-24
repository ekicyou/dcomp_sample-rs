use bevy_ecs::prelude::*;
use std::collections::HashMap;
use taffy::prelude::*;
use taffy::TaffyError;

/// taffyのStyle
#[derive(Component, Debug, Clone, PartialEq, Default)]
#[repr(transparent)]
pub struct TaffyStyle(pub(crate) Style);

unsafe impl Send for TaffyStyle {}
unsafe impl Sync for TaffyStyle {}

impl TaffyStyle {
    pub fn new(style: Style) -> Self {
        Self(style)
    }

    /// テスト用: 内部Styleへの参照を取得
    pub fn style(&self) -> &Style {
        &self.0
    }
}

/// レイアウト計算結果
#[derive(Component, Debug, Clone, PartialEq, Copy, Default)]
#[repr(transparent)]
pub struct TaffyComputedLayout(pub(crate) Layout);

impl From<Layout> for TaffyComputedLayout {
    fn from(layout: Layout) -> Self {
        Self(layout)
    }
}

/// Taffyレイアウトエンジンとエンティティマッピングを管理するリソース
#[derive(Resource)]
pub struct TaffyLayoutResource {
    /// Taffyレイアウトツリー
    tree: TaffyTree<()>,
    /// Entity → NodeId マッピング
    entity_to_node: HashMap<Entity, NodeId>,
    /// NodeId → Entity マッピング（逆引き用）
    node_to_entity: HashMap<NodeId, Entity>,
}

// TaffyTreeは内部的に*const ()を持つが、ECSのリソース管理により
// 所有権とライフタイムは保証されるため、Send/Syncは安全
unsafe impl Send for TaffyLayoutResource {}
unsafe impl Sync for TaffyLayoutResource {}

impl Default for TaffyLayoutResource {
    fn default() -> Self {
        Self {
            tree: TaffyTree::new(),
            entity_to_node: HashMap::new(),
            node_to_entity: HashMap::new(),
        }
    }
}

impl TaffyLayoutResource {
    /// 新しいレイアウトノードを作成し、エンティティとマッピングする
    pub fn create_node(&mut self, entity: Entity) -> Result<NodeId, TaffyError> {
        let node_id = self.tree.new_leaf(Style::default())?;
        self.entity_to_node.insert(entity, node_id);
        self.node_to_entity.insert(node_id, entity);
        Ok(node_id)
    }

    /// エンティティに対応するレイアウトノードを削除する
    pub fn remove_node(&mut self, entity: Entity) -> Result<(), TaffyError> {
        if let Some(node_id) = self.entity_to_node.remove(&entity) {
            self.node_to_entity.remove(&node_id);
            self.tree.remove(node_id)?;
        }
        Ok(())
    }

    /// エンティティに対応するNodeIdを取得
    pub fn get_node(&self, entity: Entity) -> Option<NodeId> {
        self.entity_to_node.get(&entity).copied()
    }

    /// NodeIdに対応するエンティティを取得
    pub fn get_entity(&self, node_id: NodeId) -> Option<Entity> {
        self.node_to_entity.get(&node_id).copied()
    }

    /// Taffyツリーへの参照を取得
    pub fn taffy(&self) -> &TaffyTree<()> {
        &self.tree
    }

    /// Taffyツリーへの可変参照を取得
    pub fn taffy_mut(&mut self) -> &mut TaffyTree<()> {
        &mut self.tree
    }

    /// マッピングの整合性を検証（デバッグ用）
    #[cfg(debug_assertions)]
    pub fn verify_mapping_consistency(&self) {
        assert_eq!(
            self.entity_to_node.len(),
            self.node_to_entity.len(),
            "Entity→Node と Node→Entity のマッピング数が一致しません"
        );

        for (entity, node_id) in &self.entity_to_node {
            assert_eq!(
                self.node_to_entity.get(node_id),
                Some(entity),
                "Entity {:?} のマッピングが不整合です",
                entity
            );
        }
    }
}
