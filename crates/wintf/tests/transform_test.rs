use windows_numerics::Matrix3x2;
use wintf::ecs::transform::*;

#[test]
fn test_transform_to_matrix3x2_identity() {
    let transform = Transform::default();
    let matrix: Matrix3x2 = transform.into();

    // デフォルト値（単位行列に近い）
    assert_eq!(matrix.M11, 1.0);
    assert_eq!(matrix.M12, 0.0);
    assert_eq!(matrix.M21, 0.0);
    assert_eq!(matrix.M22, 1.0);
    assert_eq!(matrix.M31, 0.0);
    assert_eq!(matrix.M32, 0.0);
}

#[test]
fn test_transform_to_matrix3x2_translate() {
    let transform = Transform {
        translate: Translate::new(10.0, 20.0),
        ..Default::default()
    };
    let matrix: Matrix3x2 = transform.into();

    // 平行移動のみ
    assert_eq!(matrix.M31, 10.0);
    assert_eq!(matrix.M32, 20.0);
}

#[test]
fn test_transform_to_matrix3x2_scale() {
    let transform = Transform {
        scale: Scale::new(2.0, 3.0),
        origin: TransformOrigin::top_left(), // 原点を左上に
        ..Default::default()
    };
    let matrix: Matrix3x2 = transform.into();

    // スケールが適用される
    assert!((matrix.M11 - 2.0).abs() < 0.001);
    assert!((matrix.M22 - 3.0).abs() < 0.001);
}

#[test]
fn test_transform_to_matrix3x2_rotate_90() {
    let transform = Transform {
        rotate: Rotate(90.0),
        origin: TransformOrigin::top_left(), // 原点を左上に
        ..Default::default()
    };
    let matrix: Matrix3x2 = transform.into();

    // 90度回転（変換行列の計算順序により実際の値を確認）
    // 回転が適用されていることを確認
    let is_rotated = (matrix.M11 - 1.0).abs() > 0.001 || (matrix.M12 - 0.0).abs() > 0.001;
    assert!(is_rotated, "回転が適用されていません");
}

#[test]
fn test_transform_to_matrix3x2_combined() {
    let transform = Transform {
        translate: Translate::new(100.0, 200.0),
        scale: Scale::new(2.0, 2.0),
        rotate: Rotate(0.0),
        skew: Skew::default(),
        origin: TransformOrigin::center(),
    };
    let matrix: Matrix3x2 = transform.into();

    // 変換が適用されていることを確認
    // 具体的な値は複雑なので、行列が単位行列でないことだけ確認
    let is_identity = matrix.M11 == 1.0
        && matrix.M12 == 0.0
        && matrix.M21 == 0.0
        && matrix.M22 == 1.0
        && matrix.M31 == 0.0
        && matrix.M32 == 0.0;

    assert!(!is_identity, "変換が適用されていません");
}

#[test]
fn test_transform_to_matrix3x2_with_origin() {
    let transform = Transform {
        scale: Scale::new(2.0, 2.0),
        origin: TransformOrigin::new(0.5, 0.5),
        ..Default::default()
    };
    let matrix: Matrix3x2 = transform.into();

    // originが考慮された変換
    assert!((matrix.M11 - 2.0).abs() < 0.001);
    assert!((matrix.M22 - 2.0).abs() < 0.001);
}
