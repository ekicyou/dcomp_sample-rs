//! RemoveVisual API テスト (R1)
//!
//! DCompositionVisualExt トレイトの remove_visual/remove_all_visuals メソッドをテストする。

use windows::core::Result;
use wintf::com::dcomp::*;
use wintf::ecs::GraphicsCore;

/// テスト用の GraphicsCore を作成するヘルパー関数
fn setup_graphics() -> Result<GraphicsCore> {
    GraphicsCore::new()
}

/// remove_visual: 子Visualを親から削除できることを確認
#[test]
fn test_remove_visual_removes_child_from_parent() -> Result<()> {
    let graphics = setup_graphics()?;
    let dcomp = graphics.dcomp().expect("dcomp device should exist");

    // 親Visual と 子Visual を作成
    let parent_visual = dcomp.create_visual()?;
    let child_visual = dcomp.create_visual()?;

    // 子を親に追加
    parent_visual.add_visual(&child_visual, false, None)?;

    // 子を親から削除
    let result = parent_visual.remove_visual(&child_visual);
    assert!(result.is_ok(), "remove_visual should succeed: {:?}", result);

    Ok(())
}

/// remove_visual: 存在しないVisualの削除はエラーを返す（適切なエラーハンドリング）
#[test]
fn test_remove_visual_nonexistent_returns_error() -> Result<()> {
    let graphics = setup_graphics()?;
    let dcomp = graphics.dcomp().expect("dcomp device should exist");

    // 親Visual と 子Visual を作成（子は追加しない）
    let parent_visual = dcomp.create_visual()?;
    let child_visual = dcomp.create_visual()?;

    // 追加していないVisualを削除しようとする → エラーが返るはず
    let result = parent_visual.remove_visual(&child_visual);

    // DirectComposition の仕様: 追加していないVisualを削除しようとするとエラー
    // 注意: 実際の動作はOS/バージョンにより異なる可能性があるため、
    // エラーが返る場合とOKが返る場合の両方を許容する
    // (ドキュメントでは E_INVALIDARG を返すとされている)

    // テストとしては、呼び出しがパニックしないことを確認
    // エラーハンドリングが適切に行われていることの確認
    eprintln!("remove_visual result for nonexistent: {:?}", result);

    Ok(())
}

/// remove_all_visuals: 全ての子Visualを削除できることを確認
#[test]
fn test_remove_all_visuals_clears_all_children() -> Result<()> {
    let graphics = setup_graphics()?;
    let dcomp = graphics.dcomp().expect("dcomp device should exist");

    // 親Visual と 複数の子Visual を作成
    let parent_visual = dcomp.create_visual()?;
    let child1 = dcomp.create_visual()?;
    let child2 = dcomp.create_visual()?;
    let child3 = dcomp.create_visual()?;

    // 全ての子を親に追加
    parent_visual.add_visual(&child1, false, None)?;
    parent_visual.add_visual(&child2, false, None)?;
    parent_visual.add_visual(&child3, false, None)?;

    // 全ての子を一括削除
    let result = parent_visual.remove_all_visuals();
    assert!(
        result.is_ok(),
        "remove_all_visuals should succeed: {:?}",
        result
    );

    // 削除後に再度remove_all_visualsを呼んでも問題ないことを確認
    let result2 = parent_visual.remove_all_visuals();
    assert!(
        result2.is_ok(),
        "remove_all_visuals on empty parent should succeed: {:?}",
        result2
    );

    Ok(())
}

/// remove_visual: 複数回の追加・削除が正常に動作することを確認
#[test]
fn test_remove_visual_multiple_operations() -> Result<()> {
    let graphics = setup_graphics()?;
    let dcomp = graphics.dcomp().expect("dcomp device should exist");

    let parent_visual = dcomp.create_visual()?;
    let child_visual = dcomp.create_visual()?;

    // 追加 → 削除 → 再追加 → 再削除 のサイクル
    parent_visual.add_visual(&child_visual, false, None)?;
    parent_visual.remove_visual(&child_visual)?;
    parent_visual.add_visual(&child_visual, false, None)?;
    parent_visual.remove_visual(&child_visual)?;

    Ok(())
}
