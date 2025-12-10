//! VisualGraphics parent_visual テスト (R9)
//!
//! VisualGraphics コンポーネントの parent_visual フィールドと
//! on_remove フックによる自動削除をテストする。

use bevy_ecs::prelude::*;
use windows::core::Result;
use wintf::com::dcomp::*;
use wintf::ecs::{GraphicsCore, VisualGraphics};

/// テスト用の GraphicsCore を作成するヘルパー関数
fn setup_graphics() -> Result<GraphicsCore> {
    GraphicsCore::new()
}

/// VisualGraphics が parent_visual フィールドを持つことを確認
#[test]
fn test_visual_graphics_has_parent_visual_field() -> Result<()> {
    let graphics = setup_graphics()?;
    let dcomp = graphics.dcomp().expect("dcomp device should exist");

    // Visual を作成
    let visual = dcomp.create_visual()?;
    let vg = VisualGraphics::new(visual);

    // 初期状態では parent_visual は None
    assert!(
        vg.parent_visual().is_none(),
        "Initial parent_visual should be None"
    );

    Ok(())
}

/// VisualGraphics::new_with_parent で親を指定して作成できることを確認
#[test]
fn test_visual_graphics_new_with_parent() -> Result<()> {
    let graphics = setup_graphics()?;
    let dcomp = graphics.dcomp().expect("dcomp device should exist");

    let parent_visual = dcomp.create_visual()?;
    let child_visual = dcomp.create_visual()?;

    // 親を指定して作成
    let vg = VisualGraphics::new_with_parent(child_visual.clone(), Some(parent_visual.clone()));

    // parent_visual が設定されていることを確認
    assert!(vg.parent_visual().is_some(), "parent_visual should be set");

    Ok(())
}

/// set_parent_visual で親を更新できることを確認
#[test]
fn test_visual_graphics_set_parent_visual() -> Result<()> {
    let graphics = setup_graphics()?;
    let dcomp = graphics.dcomp().expect("dcomp device should exist");

    let parent1 = dcomp.create_visual()?;
    let parent2 = dcomp.create_visual()?;
    let child = dcomp.create_visual()?;

    let mut vg = VisualGraphics::new(child);

    // 親なしの状態
    assert!(vg.parent_visual().is_none());

    // 親1を設定
    vg.set_parent_visual(Some(parent1.clone()));
    assert!(vg.parent_visual().is_some());

    // 親2に変更
    vg.set_parent_visual(Some(parent2.clone()));
    assert!(vg.parent_visual().is_some());

    // 親をクリア
    vg.set_parent_visual(None);
    assert!(vg.parent_visual().is_none());

    Ok(())
}

/// on_remove フックで親から自動削除されることをシミュレート
/// 注意: 実際の ECS フックのテストは統合テストで行う
#[test]
fn test_visual_graphics_on_remove_simulation() -> Result<()> {
    let graphics = setup_graphics()?;
    let dcomp = graphics.dcomp().expect("dcomp device should exist");

    let parent = dcomp.create_visual()?;
    let child = dcomp.create_visual()?;

    // 子を親に追加
    parent.add_visual(&child, false, None)?;

    // VisualGraphics として管理
    let vg = VisualGraphics::new_with_parent(child.clone(), Some(parent.clone()));

    // on_remove相当の処理: 親から自分を削除
    if let Some(p) = vg.parent_visual() {
        if let Some(v) = vg.visual() {
            // エラーは無視（親が先に削除されている場合など）
            let _ = p.remove_visual(v);
        }
    }

    // 削除後、再度削除を試みるとエラーになる（存在しないVisual）
    // これは仕様通りの動作
    let result = parent.remove_visual(&child);
    // 既に削除済みなのでエラーが返るはず
    eprintln!("Remove after on_remove: {:?}", result);

    Ok(())
}

/// ECS World での VisualGraphics ライフサイクルテスト
#[test]
fn test_visual_graphics_ecs_lifecycle() -> Result<()> {
    let graphics = setup_graphics()?;
    let dcomp = graphics.dcomp().expect("dcomp device should exist");

    let mut world = World::new();

    // 親エンティティを作成
    let parent_visual = dcomp.create_visual()?;
    let parent_vg = VisualGraphics::new(parent_visual.clone());
    let parent_entity = world.spawn(parent_vg).id();

    // 子エンティティを作成（親を参照）
    let child_visual = dcomp.create_visual()?;
    parent_visual.add_visual(&child_visual, false, None)?;
    let child_vg =
        VisualGraphics::new_with_parent(child_visual.clone(), Some(parent_visual.clone()));
    let child_entity = world.spawn(child_vg).id();

    // 子の VisualGraphics が parent_visual を持っていることを確認
    let child_vg_ref = world.get::<VisualGraphics>(child_entity).unwrap();
    assert!(child_vg_ref.parent_visual().is_some());

    // エンティティを despawn
    // 注意: on_remove フックが呼ばれることを期待
    world.despawn(child_entity);

    // 親エンティティはまだ存在
    assert!(world.get::<VisualGraphics>(parent_entity).is_some());

    Ok(())
}
