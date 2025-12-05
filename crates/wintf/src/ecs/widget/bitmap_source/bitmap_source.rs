//! BitmapSource コンポーネント
//!
//! 画像パスを保持する論理コンポーネント。
//! on_add時にVisualとBitmapSourceGraphicsを自動挿入し、非同期読み込みを開始する。

use bevy_ecs::component::Component;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::world::DeferredWorld;

/// 画像表示ウィジェット（WIC BitmapSourceベース）
///
/// # Example
/// ```ignore
/// commands.spawn((
///     BitmapSource::new("assets/logo.png"),
///     BoxSize::fixed(200.0, 100.0),
/// ));
/// ```
#[derive(Component, Debug, Clone)]
#[component(on_add = on_bitmap_source_add, on_remove = on_bitmap_source_remove)]
pub struct BitmapSource {
    /// 画像ファイルパス（相対または絶対）
    pub path: String,
}

impl BitmapSource {
    /// 新しいBitmapSourceコンポーネントを作成
    pub fn new<S: Into<String>>(path: S) -> Self {
        Self { path: path.into() }
    }
}

/// BitmapSource追加時のフック
///
/// - Visual + BitmapSourceGraphics + HitTest::alpha_mask()を自動挿入
/// - 非同期読み込みタスクを起動
fn on_bitmap_source_add(mut world: DeferredWorld, hook: HookContext) {
    use super::resource::BitmapSourceGraphics;
    use super::task_pool::BoxedCommand;
    use super::wic_core::WicCore;
    use super::WintfTaskPool;
    use crate::ecs::layout::HitTest;
    use crate::ecs::Visual;
    use tracing::warn;

    let entity = hook.entity;

    // Visual + BitmapSourceGraphics + HitTest::alpha_mask()を自動挿入（Rectangle/Labelパターン踏襲）
    // HitTestが未設定の場合のみデフォルトでαマスクモードを設定
    let has_visual = world.get::<Visual>(entity).is_some();
    let has_graphics = world.get::<BitmapSourceGraphics>(entity).is_some();
    let has_hit_test = world.get::<HitTest>(entity).is_some();

    if !has_visual && !has_graphics && !has_hit_test {
        world.commands().entity(entity).insert((
            Visual::default(),
            BitmapSourceGraphics::new(),
            HitTest::alpha_mask(),
        ));
    } else if !has_visual && !has_graphics {
        world
            .commands()
            .entity(entity)
            .insert((Visual::default(), BitmapSourceGraphics::new()));
    } else if !has_visual {
        world.commands().entity(entity).insert(Visual::default());
    } else if !has_graphics {
        world
            .commands()
            .entity(entity)
            .insert(BitmapSourceGraphics::new());
    }

    // HitTestが未設定の場合のみデフォルトでαマスクモードを追加
    if !has_hit_test {
        world
            .commands()
            .entity(entity)
            .insert(HitTest::alpha_mask());
    }

    // WicCoreをcloneして取得
    let wic_core = match world.get_resource::<WicCore>() {
        Some(wic) => wic.clone(),
        None => {
            warn!("[BitmapSource] WicCore not found");
            return;
        }
    };

    // パスを取得
    let path = match world.get::<BitmapSource>(entity) {
        Some(bs) => bs.path.clone(),
        None => return,
    };

    // 非同期読み込みタスクを起動
    if let Some(task_pool) = world.get_resource::<WintfTaskPool>() {
        task_pool.spawn(move |tx| async move {
            // パス解決
            let resolved = match super::systems::resolve_path(&path) {
                Ok(p) => p,
                Err(e) => {
                    warn!("[BitmapSource] Failed to resolve path '{}': {:?}", path, e);
                    return;
                }
            };

            // 画像読み込み
            match super::systems::load_bitmap_source(wic_core.factory(), &resolved) {
                Ok(source) => {
                    use super::resource::BitmapSourceResource;
                    // BitmapSourceResourceにラップしてからクロージャに渡す
                    // (BitmapSourceResourceはSend実装済み、IWICBitmapSourceは未実装)
                    let resource = BitmapSourceResource::new(source);
                    let cmd: BoxedCommand = Box::new(move |world: &mut bevy_ecs::world::World| {
                        // エンティティが存在するか確認
                        if let Ok(mut entity_ref) = world.get_entity_mut(entity) {
                            entity_ref.insert(resource);
                        }
                    });
                    let _ = tx.send(cmd);
                }
                Err(e) => {
                    warn!(
                        "[BitmapSource] Failed to load '{}': {:?}",
                        resolved.display(),
                        e
                    );
                }
            }
        });
    }
}

/// BitmapSource削除時のフック
fn on_bitmap_source_remove(mut world: DeferredWorld, hook: HookContext) {
    use crate::ecs::GraphicsCommandList;
    use bevy_ecs::change_detection::DetectChangesMut;

    let entity = hook.entity;

    // GraphicsCommandListをクリア（Changed検出のため）
    if let Some(mut cmd_list) = world.get_mut::<GraphicsCommandList>(entity) {
        cmd_list.set_if_neq(GraphicsCommandList::empty());
    }
}
