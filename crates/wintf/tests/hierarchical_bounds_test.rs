// Task 7.1: 階層的バウンディングボックス計算の統合テスト

use windows_numerics::Matrix3x2;
use wintf::ecs::{Arrangement, GlobalArrangement, LayoutScale, Offset, Size};

/// 3階層のWidgetツリーでの階層的bounds計算をテスト
/// GlobalArrangementのtrait実装を通じて正しく計算されることを検証
#[test]
fn test_hierarchical_bounds_calculation() {
    // 親ウィジェット（レベル0）
    // Position: (10, 20), Scale: (2.0, 2.0), Size: (100, 50)
    let parent_arr = Arrangement {
        offset: Offset { x: 10.0, y: 20.0 },
        scale: LayoutScale { x: 2.0, y: 2.0 },
        size: Size {
            width: 100.0,
            height: 50.0,
        },
    };
    let parent_global: GlobalArrangement = parent_arr.into();

    // 親のbounds検証
    // local_bounds = (0,0,100,50) ← 原点基準
    // transform = scale(2,2) + translate(20,40) (scaleされたoffset: 10*2=20, 20*2=40)
    // bounds = (0,0,100,50) * scale(2,2) + translate(20,40) = (20,40,220,140)
    assert_eq!(parent_global.bounds.left, 20.0);
    assert_eq!(parent_global.bounds.top, 40.0);
    assert_eq!(parent_global.bounds.right, 220.0);
    assert_eq!(parent_global.bounds.bottom, 140.0);

    // 子ウィジェット（レベル1）
    // Position: (5, 10), Scale: (1.0, 1.0), Size: (30, 20)
    let child_arr = Arrangement {
        offset: Offset { x: 5.0, y: 10.0 },
        scale: LayoutScale { x: 1.0, y: 1.0 },
        size: Size {
            width: 30.0,
            height: 20.0,
        },
    };
    let child_global = parent_global * child_arr;

    // 子のbounds検証（親の変換適用後）
    // 新ロジック: 子のoffsetに親のスケールを適用
    // parent.bounds.left = 20 (10*2)
    // child.offset = (5, 10), parent.scale = (2, 2)
    // scaled_child_offset = (5*2, 10*2) = (10, 20)
    // child.bounds.left = 20 + 10 = 30
    // child.bounds.right = 30 + 30*2 = 90 (size 30 * parent_scale 2)
    assert_eq!(child_global.bounds.left, 30.0);
    assert_eq!(child_global.bounds.top, 60.0);
    assert_eq!(child_global.bounds.right, 90.0);
    assert_eq!(child_global.bounds.bottom, 100.0);

    // 孫ウィジェット（レベル2）
    // Position: (2, 3), Scale: (1.5, 1.5), Size: (10, 8)
    let grandchild_arr = Arrangement {
        offset: Offset { x: 2.0, y: 3.0 },
        scale: LayoutScale { x: 1.5, y: 1.5 },
        size: Size {
            width: 10.0,
            height: 8.0,
        },
    };
    let grandchild_global = child_global * grandchild_arr;

    // 孫のbounds検証（親と子の累積変換適用後）
    // 新ロジック: 孫のoffsetに親（child）のスケールを適用
    // child.bounds.left = 30
    // child.transform.M11 = 2 (累積スケール)
    // grandchild.offset = (2, 3)
    // scaled_offset = (2*2, 3*2) = (4, 6)
    // grandchild.bounds.left = 30 + 4 = 34
    // result.M11 = 2 * 1.5 = 3
    // grandchild.bounds.right = 34 + 10*3 = 64
    assert_eq!(grandchild_global.bounds.left, 34.0);
    assert_eq!(grandchild_global.bounds.top, 66.0);
    assert_eq!(grandchild_global.bounds.right, 64.0);
    assert_eq!(grandchild_global.bounds.bottom, 90.0);
}

/// From<Arrangement>実装が正しくboundsを設定することを確認
#[test]
fn test_global_arrangement_from_arrangement() {
    let arrangement = Arrangement {
        offset: Offset { x: 0.0, y: 0.0 },
        scale: LayoutScale { x: 1.0, y: 1.0 },
        size: Size {
            width: 50.0,
            height: 50.0,
        },
    };

    let global: GlobalArrangement = arrangement.into();

    // transform: identity
    assert_eq!(global.transform, Matrix3x2::identity());

    // bounds: left=0, top=0, right=50, bottom=50
    assert_eq!(global.bounds.left, 0.0);
    assert_eq!(global.bounds.top, 0.0);
    assert_eq!(global.bounds.right, 50.0);
    assert_eq!(global.bounds.bottom, 50.0);
}

/// Mul<Arrangement>実装が正しくboundsを計算することを確認
#[test]
fn test_global_arrangement_mul_arrangement() {
    let parent = Arrangement {
        offset: Offset { x: 10.0, y: 15.0 },
        scale: LayoutScale { x: 1.0, y: 1.0 },
        size: Size {
            width: 20.0,
            height: 25.0,
        },
    };
    let parent_global: GlobalArrangement = parent.into();

    let child = Arrangement {
        offset: Offset { x: 5.0, y: 10.0 },
        scale: LayoutScale { x: 1.0, y: 1.0 },
        size: Size {
            width: 10.0,
            height: 10.0,
        },
    };

    let child_global = parent_global * child;

    // transform: parent_transform * child_transform
    // parent: offset(10, 15), scale(1, 1)
    // child: offset(5, 10), scale(1, 1)
    // result: offset(10+5, 15+10) = (15, 25), scale(1, 1)
    assert_eq!(child_global.transform.M31, 15.0);
    assert_eq!(child_global.transform.M32, 25.0);

    // bounds: child.local_bounds() transformed by result_transform
    // child.local_bounds(): (0, 0, 10, 10) - 原点基準
    // transformed: (15, 25, 25, 35)
    assert_eq!(child_global.bounds.left, 15.0);
    assert_eq!(child_global.bounds.top, 25.0);
    assert_eq!(child_global.bounds.right, 25.0);
    assert_eq!(child_global.bounds.bottom, 35.0);
}

/// スケール変換のみのテスト
#[test]
fn test_scale_only_transformation() {
    let parent = Arrangement {
        offset: Offset { x: 0.0, y: 0.0 },
        scale: LayoutScale { x: 3.0, y: 2.0 },
        size: Size {
            width: 10.0,
            height: 10.0,
        },
    };
    let parent_global: GlobalArrangement = parent.into();

    let child = Arrangement {
        offset: Offset { x: 5.0, y: 10.0 },
        scale: LayoutScale { x: 1.0, y: 1.0 },
        size: Size {
            width: 10.0,
            height: 10.0,
        },
    };

    let child_global = parent_global * child;

    // 子のローカル: offset(5, 10), size(10, 10)
    // 親のscale(3.0, 2.0), offset(0, 0)
    // 子のoffsetに親のスケールを適用: (5*3, 10*2) = (15, 20)
    // child.local_bounds(): (0, 0, 10, 10) - 原点基準
    // result_transform: scale(3,2), translation(15, 20)
    // transformed: (15, 20, 45, 40)
    assert_eq!(child_global.bounds.left, 15.0);
    assert_eq!(child_global.bounds.top, 20.0);
    assert_eq!(child_global.bounds.right, 45.0);
    assert_eq!(child_global.bounds.bottom, 40.0);
}

/// 複雑なスケール階層のテスト
#[test]
fn test_complex_scale_hierarchy() {
    // ルート: scale(2.0, 2.0)
    let root = Arrangement {
        offset: Offset { x: 0.0, y: 0.0 },
        scale: LayoutScale { x: 2.0, y: 2.0 },
        size: Size {
            width: 100.0,
            height: 100.0,
        },
    };
    let root_global: GlobalArrangement = root.into();

    // レベル1: offset(10, 10), scale(1.5, 1.5)
    let level1 = Arrangement {
        offset: Offset { x: 10.0, y: 10.0 },
        scale: LayoutScale { x: 1.5, y: 1.5 },
        size: Size {
            width: 50.0,
            height: 50.0,
        },
    };
    let level1_global = root_global * level1;

    // 新ロジック: offsetに親スケールを適用
    // root: offset=0, scale=2, bounds = (0, 0, 200, 200)
    // level1.offset = (10, 10), parent_scale = 2
    // scaled_offset = (20, 20)
    // level1.bounds.left = 0 + 20 = 20
    // result.M11 = 2 * 1.5 = 3
    // level1.bounds.right = 20 + 50*3 = 170
    assert_eq!(level1_global.bounds.left, 20.0);
    assert_eq!(level1_global.bounds.top, 20.0);
    assert_eq!(level1_global.bounds.right, 170.0);
    assert_eq!(level1_global.bounds.bottom, 170.0);

    // レベル2: offset(5, 5), scale(1.0, 1.0), size(10, 10)
    let level2 = Arrangement {
        offset: Offset { x: 5.0, y: 5.0 },
        scale: LayoutScale { x: 1.0, y: 1.0 },
        size: Size {
            width: 10.0,
            height: 10.0,
        },
    };
    let level2_global = level1_global * level2;

    // level2.offset = (5, 5), parent_scale = 3 (累積)
    // scaled_offset = (15, 15)
    // level2.bounds.left = 20 + 15 = 35
    // result.M11 = 3 * 1 = 3
    // level2.bounds.right = 35 + 10*3 = 65
    assert_eq!(level2_global.bounds.left, 35.0);
    assert_eq!(level2_global.bounds.top, 35.0);
    assert_eq!(level2_global.bounds.right, 65.0);
    assert_eq!(level2_global.bounds.bottom, 65.0);
}
