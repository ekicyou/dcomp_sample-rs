use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;
use windows_numerics::{Matrix3x2, Vector2};
use wintf::ecs::{
    transform_rect_axis_aligned, Arrangement, D2DRectExt, GlobalArrangement, LayoutScale, Offset,
    Size,
};

// Task 6.1: Size構造体とArrangement.local_bounds()のテスト

#[test]
fn test_size_default() {
    let size = Size::default();
    assert_eq!(size.width, 0.0);
    assert_eq!(size.height, 0.0);
}

#[test]
fn test_size_copy_clone() {
    let size1 = Size {
        width: 100.0,
        height: 50.0,
    };
    let size2 = size1; // Copy
    let size3 = size1.clone(); // Clone

    assert_eq!(size1, size2);
    assert_eq!(size1, size3);
}

#[test]
fn test_arrangement_local_bounds_positive_size() {
    let arrangement = Arrangement {
        offset: Offset { x: 10.0, y: 20.0 },
        scale: LayoutScale { x: 1.0, y: 1.0 },
        size: Size {
            width: 100.0,
            height: 50.0,
        },
    };

    let bounds = arrangement.local_bounds();
    assert_eq!(bounds.left, 10.0);
    assert_eq!(bounds.top, 20.0);
    assert_eq!(bounds.right, 110.0); // 10 + 100
    assert_eq!(bounds.bottom, 70.0); // 20 + 50
}

#[test]
fn test_arrangement_local_bounds_zero_size() {
    let arrangement = Arrangement {
        offset: Offset { x: 10.0, y: 20.0 },
        scale: LayoutScale { x: 1.0, y: 1.0 },
        size: Size {
            width: 0.0,
            height: 0.0,
        },
    };

    let bounds = arrangement.local_bounds();
    assert_eq!(bounds.left, 10.0);
    assert_eq!(bounds.top, 20.0);
    assert_eq!(bounds.right, 10.0); // 10 + 0
    assert_eq!(bounds.bottom, 20.0); // 20 + 0
}

#[test]
fn test_arrangement_local_bounds_negative_size() {
    // 負のサイズでも計算は動作する（警告ログのみ）
    let arrangement = Arrangement {
        offset: Offset { x: 10.0, y: 20.0 },
        scale: LayoutScale { x: 1.0, y: 1.0 },
        size: Size {
            width: -10.0,
            height: -5.0,
        },
    };

    let bounds = arrangement.local_bounds();
    assert_eq!(bounds.left, 10.0);
    assert_eq!(bounds.top, 20.0);
    assert_eq!(bounds.right, 0.0); // 10 + (-10)
    assert_eq!(bounds.bottom, 15.0); // 20 + (-5)
}

// Task 6.3: D2DRectExt拡張トレイトのテスト

#[test]
fn test_d2drect_from_offset_size() {
    let offset = Offset { x: 10.0, y: 20.0 };
    let size = Size {
        width: 100.0,
        height: 50.0,
    };

    let rect = D2D_RECT_F::from_offset_size(offset, size);
    assert_eq!(rect.left, 10.0);
    assert_eq!(rect.top, 20.0);
    assert_eq!(rect.right, 110.0);
    assert_eq!(rect.bottom, 70.0);
}

#[test]
fn test_d2drect_width_height() {
    let rect = D2D_RECT_F {
        left: 10.0,
        top: 20.0,
        right: 110.0,
        bottom: 70.0,
    };

    assert_eq!(rect.width(), 100.0);
    assert_eq!(rect.height(), 50.0);
}

#[test]
fn test_d2drect_offset() {
    let rect = D2D_RECT_F {
        left: 10.0,
        top: 20.0,
        right: 110.0,
        bottom: 70.0,
    };

    let offset = rect.offset();
    assert_eq!(offset.X, 10.0);
    assert_eq!(offset.Y, 20.0);
}

#[test]
fn test_d2drect_size() {
    let rect = D2D_RECT_F {
        left: 10.0,
        top: 20.0,
        right: 110.0,
        bottom: 70.0,
    };

    let size = rect.size();
    assert_eq!(size.X, 100.0);
    assert_eq!(size.Y, 50.0);
}

#[test]
fn test_d2drect_set_offset() {
    let mut rect = D2D_RECT_F {
        left: 10.0,
        top: 20.0,
        right: 110.0,
        bottom: 70.0,
    };

    rect.set_offset(Vector2 { X: 30.0, Y: 40.0 });
    assert_eq!(rect.left, 30.0);
    assert_eq!(rect.top, 40.0);
    assert_eq!(rect.right, 130.0); // 幅100を維持
    assert_eq!(rect.bottom, 90.0); // 高さ50を維持
}

#[test]
fn test_d2drect_set_size() {
    let mut rect = D2D_RECT_F {
        left: 10.0,
        top: 20.0,
        right: 110.0,
        bottom: 70.0,
    };

    rect.set_size(Vector2 { X: 200.0, Y: 100.0 });
    assert_eq!(rect.left, 10.0); // 左上を維持
    assert_eq!(rect.top, 20.0);
    assert_eq!(rect.right, 210.0); // 10 + 200
    assert_eq!(rect.bottom, 120.0); // 20 + 100
}

#[test]
fn test_d2drect_contains() {
    let rect = D2D_RECT_F {
        left: 10.0,
        top: 20.0,
        right: 110.0,
        bottom: 70.0,
    };

    assert!(rect.contains(10.0, 20.0)); // 左上
    assert!(rect.contains(110.0, 70.0)); // 右下
    assert!(rect.contains(50.0, 40.0)); // 中央
    assert!(!rect.contains(5.0, 20.0)); // 左外
    assert!(!rect.contains(10.0, 15.0)); // 上外
    assert!(!rect.contains(115.0, 40.0)); // 右外
    assert!(!rect.contains(50.0, 75.0)); // 下外
}

#[test]
fn test_d2drect_union() {
    let rect1 = D2D_RECT_F {
        left: 10.0,
        top: 20.0,
        right: 50.0,
        bottom: 60.0,
    };

    let rect2 = D2D_RECT_F {
        left: 30.0,
        top: 40.0,
        right: 70.0,
        bottom: 80.0,
    };

    let union = rect1.union(&rect2);
    assert_eq!(union.left, 10.0); // min(10, 30)
    assert_eq!(union.top, 20.0); // min(20, 40)
    assert_eq!(union.right, 70.0); // max(50, 70)
    assert_eq!(union.bottom, 80.0); // max(60, 80)
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "Invalid rect: left > right")]
fn test_d2drect_validate_invalid_horizontal() {
    let rect = D2D_RECT_F {
        left: 100.0,
        top: 20.0,
        right: 50.0, // left > right
        bottom: 70.0,
    };

    rect.validate();
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "Invalid rect: top > bottom")]
fn test_d2drect_validate_invalid_vertical() {
    let rect = D2D_RECT_F {
        left: 10.0,
        top: 100.0,
        right: 50.0,
        bottom: 70.0, // top > bottom
    };

    rect.validate();
}

// Task 6.2: transform_rect_axis_aligned関数のテスト

#[test]
fn test_transform_rect_identity() {
    let rect = D2D_RECT_F {
        left: 10.0,
        top: 20.0,
        right: 110.0,
        bottom: 70.0,
    };

    let matrix = Matrix3x2::identity();
    let result = transform_rect_axis_aligned(&rect, &matrix);

    assert_eq!(result.left, rect.left);
    assert_eq!(result.top, rect.top);
    assert_eq!(result.right, rect.right);
    assert_eq!(result.bottom, rect.bottom);
}

#[test]
fn test_transform_rect_translation_only() {
    let rect = D2D_RECT_F {
        left: 10.0,
        top: 20.0,
        right: 110.0,
        bottom: 70.0,
    };

    let matrix = Matrix3x2::translation(5.0, 10.0);
    let result = transform_rect_axis_aligned(&rect, &matrix);

    assert_eq!(result.left, 15.0); // 10 + 5
    assert_eq!(result.top, 30.0); // 20 + 10
    assert_eq!(result.right, 115.0); // 110 + 5
    assert_eq!(result.bottom, 80.0); // 70 + 10
}

#[test]
fn test_transform_rect_scale_only() {
    let rect = D2D_RECT_F {
        left: 10.0,
        top: 20.0,
        right: 110.0,
        bottom: 70.0,
    };

    let matrix = Matrix3x2::scale(2.0, 2.0);
    let result = transform_rect_axis_aligned(&rect, &matrix);

    assert_eq!(result.left, 20.0); // 10 * 2
    assert_eq!(result.top, 40.0); // 20 * 2
    assert_eq!(result.right, 220.0); // 110 * 2
    assert_eq!(result.bottom, 140.0); // 70 * 2
}

#[test]
fn test_transform_rect_translation_and_scale() {
    let rect = D2D_RECT_F {
        left: 10.0,
        top: 20.0,
        right: 110.0,
        bottom: 70.0,
    };

    // スケール -> 平行移動の順
    let scale = Matrix3x2::scale(2.0, 2.0);
    let translation = Matrix3x2::translation(5.0, 10.0);
    let matrix = scale * translation;

    let result = transform_rect_axis_aligned(&rect, &matrix);

    // (10, 20) -> scale -> (20, 40) -> translate -> (25, 50)
    // (110, 70) -> scale -> (220, 140) -> translate -> (225, 150)
    assert_eq!(result.left, 25.0);
    assert_eq!(result.top, 50.0);
    assert_eq!(result.right, 225.0);
    assert_eq!(result.bottom, 150.0);
}

// Task 6.4: GlobalArrangementのtrait実装テスト

#[test]
fn test_global_arrangement_from_arrangement() {
    let arrangement = Arrangement {
        offset: Offset { x: 10.0, y: 20.0 },
        scale: LayoutScale { x: 2.0, y: 2.0 },
        size: Size {
            width: 100.0,
            height: 50.0,
        },
    };

    let global: GlobalArrangement = arrangement.into();

    // transform検証
    assert_eq!(global.transform.M11, 2.0); // scale x
    assert_eq!(global.transform.M22, 2.0); // scale y
    assert_eq!(global.transform.M31, 10.0); // offset x
    assert_eq!(global.transform.M32, 20.0); // offset y

    // bounds検証（local_bounds()と同じ）
    assert_eq!(global.bounds.left, 10.0);
    assert_eq!(global.bounds.top, 20.0);
    assert_eq!(global.bounds.right, 110.0);
    assert_eq!(global.bounds.bottom, 70.0);
}

#[test]
fn test_global_arrangement_mul_arrangement() {
    let parent = GlobalArrangement {
        transform: Matrix3x2::translation(10.0, 20.0),
        bounds: D2D_RECT_F {
            left: 10.0,
            top: 20.0,
            right: 50.0,
            bottom: 60.0,
        },
    };

    let child = Arrangement {
        offset: Offset { x: 5.0, y: 7.0 },
        scale: LayoutScale { x: 1.0, y: 1.0 },
        size: Size {
            width: 30.0,
            height: 20.0,
        },
    };

    let result = parent * child;

    // transform検証: 親と子の累積
    assert_eq!(result.transform.M31, 15.0); // 10 + 5
    assert_eq!(result.transform.M32, 27.0); // 20 + 7

    // bounds検証: 子のlocal_boundsが結果transformで変換される
    // 子のlocal_bounds: (5, 7, 35, 27)
    // 結果transform: translate(10,20) * translate(5,7) = translate(15,27)
    // bounds変換: (5,7,35,27) + (15,27) = (20,34,50,54)
    assert_eq!(result.bounds.left, 20.0); // 5 + 15
    assert_eq!(result.bounds.top, 34.0); // 7 + 27
    assert_eq!(result.bounds.right, 50.0); // 35 + 15
    assert_eq!(result.bounds.bottom, 54.0); // 27 + 27
}
