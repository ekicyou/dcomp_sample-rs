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

    // 親のbounds検証（ローカル座標系そのまま）
    // offset: (10, 20), size: (100, 50)
    // bounds: left=10, top=20, right=110, bottom=70
    assert_eq!(parent_global.bounds.left, 10.0);
    assert_eq!(parent_global.bounds.top, 20.0);
    assert_eq!(parent_global.bounds.right, 110.0);
    assert_eq!(parent_global.bounds.bottom, 70.0);

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
    // parent_transform: scale(2,2) * translation(10,20)
    // child_transform: scale(1,1) * translation(5,10) = translation(5,10)
    // result_transform: parent * child = scale(2,2) * translation(10,20) * translation(5,10)
    //                 = scale(2,2) * translation(15,30)
    // child.local_bounds(): (5, 10, 35, 30)
    // apply scale(2,2): (10, 20, 70, 60)
    // apply translation(15,30): (25, 50, 85, 90)
    assert_eq!(child_global.bounds.left, 25.0);
    assert_eq!(child_global.bounds.top, 50.0);
    assert_eq!(child_global.bounds.right, 85.0);
    assert_eq!(child_global.bounds.bottom, 90.0);

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
    // child_transform: scale(2,2), translation(15,30)
    // grandchild: offset(2,3), scale(1.5,1.5), size(10,8)
    // grandchild.local_bounds(): (2, 3, 12, 11)
    // result_transform: scale(3,3), translation(24.5,48)
    // transformed bounds: (30.5, 57, 60.5, 81)
    assert_eq!(grandchild_global.bounds.left, 30.5);
    assert_eq!(grandchild_global.bounds.top, 57.0);
    assert_eq!(grandchild_global.bounds.right, 60.5);
    assert_eq!(grandchild_global.bounds.bottom, 81.0);
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
    // child.local_bounds(): left=5, top=10, right=15, bottom=20
    // transformed: left=5+15=20, top=10+25=35, right=15+15=30, bottom=20+25=45
    assert_eq!(child_global.bounds.left, 20.0);
    assert_eq!(child_global.bounds.top, 35.0);
    assert_eq!(child_global.bounds.right, 30.0);
    assert_eq!(child_global.bounds.bottom, 45.0);
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
    // child.local_bounds(): (5, 10, 15, 20)
    // transformed: (20, 30, 50, 50)
    assert_eq!(child_global.bounds.left, 20.0);
    assert_eq!(child_global.bounds.top, 30.0);
    assert_eq!(child_global.bounds.right, 50.0);
    assert_eq!(child_global.bounds.bottom, 50.0);
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

    // レベル1のbounds: (40, 40, 190, 190)
    assert_eq!(level1_global.bounds.left, 40.0);
    assert_eq!(level1_global.bounds.top, 40.0);
    assert_eq!(level1_global.bounds.right, 190.0);
    assert_eq!(level1_global.bounds.bottom, 190.0);

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

    // レベル2のbounds: (30, 30, 60, 60)
    assert_eq!(level2_global.bounds.left, 30.0);
    assert_eq!(level2_global.bounds.top, 30.0);
    assert_eq!(level2_global.bounds.right, 60.0);
    assert_eq!(level2_global.bounds.bottom, 60.0);
}
