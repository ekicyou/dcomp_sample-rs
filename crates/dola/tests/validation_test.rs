//! Validation tests for V1-V13
//! Tasks 7.6, 8.6

use dola::*;
use std::collections::BTreeMap;

/// ヘルパー: 最小の有効なドキュメントを作成
fn minimal_valid_doc() -> DolaDocument {
    DolaDocument {
        schema_version: "1.0".to_string(),
        variable: BTreeMap::new(),
        transition: BTreeMap::new(),
        storyboard: BTreeMap::new(),
    }
}

/// ヘルパー: f64変数付きドキュメント
fn doc_with_float_var(name: &str, initial: f64, min: Option<f64>, max: Option<f64>) -> DolaDocument {
    let mut variable = BTreeMap::new();
    variable.insert(
        name.to_string(),
        AnimationVariableDef::Float { initial, min, max },
    );
    DolaDocument {
        schema_version: "1.0".to_string(),
        variable,
        transition: BTreeMap::new(),
        storyboard: BTreeMap::new(),
    }
}

// =============================================================
// V1: スキーマバージョン検証
// =============================================================

mod v1_tests {
    use super::*;

    #[test]
    fn schema_version_1_0_ok() {
        let doc = minimal_valid_doc();
        assert!(doc.validate().is_ok());
    }

    #[test]
    fn schema_version_mismatch() {
        let doc = DolaDocument {
            schema_version: "2.0".to_string(),
            ..minimal_valid_doc()
        };
        let errors = doc.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            DolaError::SchemaVersionMismatch { expected, found }
            if expected == "1.0" && found == "2.0"
        )));
    }
}

// =============================================================
// V2: キーフレーム名重複検出
// =============================================================

mod v2_tests {
    use super::*;

    #[test]
    fn duplicate_keyframe_detected() {
        let mut doc = minimal_valid_doc();
        let mut variable = BTreeMap::new();
        variable.insert(
            "x".to_string(),
            AnimationVariableDef::Float {
                initial: 0.0,
                min: None,
                max: None,
            },
        );
        doc.variable = variable;

        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![
                    StoryboardEntry {
                        variable: Some("x".to_string()),
                        transition: Some(TransitionRef::Inline(TransitionDef {
                            from: None,
                            to: Some(TransitionValue::Scalar(1.0)),
                            relative_to: None,
                            easing: None,
                            delay: 0.0,
                            duration: Some(1.0),
                        })),
                        at: None,
                        between: None,
                        keyframe: Some("visible".to_string()),
                    },
                    StoryboardEntry {
                        variable: Some("x".to_string()),
                        transition: Some(TransitionRef::Inline(TransitionDef {
                            from: None,
                            to: Some(TransitionValue::Scalar(2.0)),
                            relative_to: None,
                            easing: None,
                            delay: 0.0,
                            duration: Some(1.0),
                        })),
                        at: None,
                        between: None,
                        keyframe: Some("visible".to_string()), // duplicate!
                    },
                ],
            },
        );
        doc.storyboard = storyboard;

        let errors = doc.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            DolaError::DuplicateKeyframe { storyboard, name }
            if storyboard == "sb1" && name == "visible"
        )));
    }
}

// =============================================================
// V3: 予約キーフレーム名 "start" 使用禁止
// =============================================================

mod v3_tests {
    use super::*;

    #[test]
    fn reserved_keyframe_start_rejected() {
        let mut doc = minimal_valid_doc();
        let mut variable = BTreeMap::new();
        variable.insert(
            "x".to_string(),
            AnimationVariableDef::Float {
                initial: 0.0,
                min: None,
                max: None,
            },
        );
        doc.variable = variable;

        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![StoryboardEntry {
                    variable: Some("x".to_string()),
                    transition: Some(TransitionRef::Inline(TransitionDef {
                        from: None,
                        to: Some(TransitionValue::Scalar(1.0)),
                        relative_to: None,
                        easing: None,
                        delay: 0.0,
                        duration: Some(1.0),
                    })),
                    at: None,
                    between: None,
                    keyframe: Some("start".to_string()), // reserved!
                }],
            },
        );
        doc.storyboard = storyboard;

        let errors = doc.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, DolaError::ReservedKeyframeName { name } if name == "start")));
    }
}

// =============================================================
// V4: 未定義変数参照
// =============================================================

mod v4_tests {
    use super::*;

    #[test]
    fn undefined_variable_detected() {
        let mut doc = minimal_valid_doc();
        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![StoryboardEntry {
                    variable: Some("undefined_var".to_string()),
                    transition: Some(TransitionRef::Inline(TransitionDef {
                        from: None,
                        to: Some(TransitionValue::Scalar(1.0)),
                        relative_to: None,
                        easing: None,
                        delay: 0.0,
                        duration: Some(1.0),
                    })),
                    at: None,
                    between: None,
                    keyframe: None,
                }],
            },
        );
        doc.storyboard = storyboard;

        let errors = doc.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            DolaError::UndefinedVariable { name, .. }
            if name == "undefined_var"
        )));
    }
}

// =============================================================
// V5: 未定義トランジション参照
// =============================================================

mod v5_tests {
    use super::*;

    #[test]
    fn undefined_transition_detected() {
        let mut doc = minimal_valid_doc();
        let mut variable = BTreeMap::new();
        variable.insert(
            "x".to_string(),
            AnimationVariableDef::Float {
                initial: 0.0,
                min: None,
                max: None,
            },
        );
        doc.variable = variable;

        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![StoryboardEntry {
                    variable: Some("x".to_string()),
                    transition: Some(TransitionRef::Named("undefined_trans".to_string())),
                    at: None,
                    between: None,
                    keyframe: None,
                }],
            },
        );
        doc.storyboard = storyboard;

        let errors = doc.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            DolaError::UndefinedTransition { name, .. }
            if name == "undefined_trans"
        )));
    }
}

// =============================================================
// V6: キーフレーム参照検証（前方参照許可 + 暗黙的KF追跡）
// =============================================================

mod v6_tests {
    use super::*;

    #[test]
    fn forward_reference_ok() {
        // entry[0] で at="kf_from_entry_1" を参照, entry[1] で keyframe="kf_from_entry_1" を定義
        let mut doc = minimal_valid_doc();
        let mut variable = BTreeMap::new();
        variable.insert(
            "x".to_string(),
            AnimationVariableDef::Float {
                initial: 0.0,
                min: None,
                max: None,
            },
        );
        doc.variable = variable;

        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![
                    StoryboardEntry {
                        variable: Some("x".to_string()),
                        transition: Some(TransitionRef::Inline(TransitionDef {
                            from: None,
                            to: Some(TransitionValue::Scalar(1.0)),
                            relative_to: None,
                            easing: None,
                            delay: 0.0,
                            duration: Some(1.0),
                        })),
                        at: Some(KeyframeRef::Single("kf_from_entry_1".to_string())),
                        between: None,
                        keyframe: None,
                    },
                    StoryboardEntry {
                        variable: Some("x".to_string()),
                        transition: Some(TransitionRef::Inline(TransitionDef {
                            from: None,
                            to: Some(TransitionValue::Scalar(2.0)),
                            relative_to: None,
                            easing: None,
                            delay: 0.0,
                            duration: Some(1.0),
                        })),
                        at: None,
                        between: None,
                        keyframe: Some("kf_from_entry_1".to_string()),
                    },
                ],
            },
        );
        doc.storyboard = storyboard;

        assert!(doc.validate().is_ok());
    }

    #[test]
    fn undefined_keyframe_reference_detected() {
        let mut doc = minimal_valid_doc();
        let mut variable = BTreeMap::new();
        variable.insert(
            "x".to_string(),
            AnimationVariableDef::Float {
                initial: 0.0,
                min: None,
                max: None,
            },
        );
        doc.variable = variable;

        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![StoryboardEntry {
                    variable: Some("x".to_string()),
                    transition: Some(TransitionRef::Inline(TransitionDef {
                        from: None,
                        to: Some(TransitionValue::Scalar(1.0)),
                        relative_to: None,
                        easing: None,
                        delay: 0.0,
                        duration: Some(1.0),
                    })),
                    at: Some(KeyframeRef::Single("nonexistent".to_string())),
                    between: None,
                    keyframe: None,
                }],
            },
        );
        doc.storyboard = storyboard;

        let errors = doc.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            DolaError::UndefinedKeyframe { name, .. }
            if name == "nonexistent"
        )));
    }

    #[test]
    fn implicit_keyframe_forward_reference_ok() {
        // entry[0] で at="__implicit_1" を参照、entry[1] は keyframe 省略 → 暗黙的KF __implicit_1
        let mut doc = minimal_valid_doc();
        let mut variable = BTreeMap::new();
        variable.insert(
            "x".to_string(),
            AnimationVariableDef::Float {
                initial: 0.0,
                min: None,
                max: None,
            },
        );
        doc.variable = variable;

        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![
                    StoryboardEntry {
                        variable: Some("x".to_string()),
                        transition: Some(TransitionRef::Inline(TransitionDef {
                            from: None,
                            to: Some(TransitionValue::Scalar(1.0)),
                            relative_to: None,
                            easing: None,
                            delay: 0.0,
                            duration: Some(1.0),
                        })),
                        at: Some(KeyframeRef::Single("__implicit_1".to_string())),
                        between: None,
                        keyframe: None,
                    },
                    StoryboardEntry {
                        variable: Some("x".to_string()),
                        transition: Some(TransitionRef::Inline(TransitionDef {
                            from: None,
                            to: Some(TransitionValue::Scalar(2.0)),
                            relative_to: None,
                            easing: None,
                            delay: 0.0,
                            duration: Some(1.0),
                        })),
                        at: None,
                        between: None,
                        keyframe: None, // implicit KF: __implicit_1
                    },
                ],
            },
        );
        doc.storyboard = storyboard;

        assert!(doc.validate().is_ok());
    }

    #[test]
    fn at_start_reserved_keyframe_ok() {
        // at = "start" は予約KFなのでOK
        let mut doc = minimal_valid_doc();
        let mut variable = BTreeMap::new();
        variable.insert(
            "x".to_string(),
            AnimationVariableDef::Float {
                initial: 0.0,
                min: None,
                max: None,
            },
        );
        doc.variable = variable;

        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![StoryboardEntry {
                    variable: Some("x".to_string()),
                    transition: Some(TransitionRef::Inline(TransitionDef {
                        from: None,
                        to: Some(TransitionValue::Scalar(1.0)),
                        relative_to: None,
                        easing: None,
                        delay: 0.0,
                        duration: Some(1.0),
                    })),
                    at: Some(KeyframeRef::Single("start".to_string())),
                    between: None,
                    keyframe: None,
                }],
            },
        );
        doc.storyboard = storyboard;

        assert!(doc.validate().is_ok());
    }
}

// =============================================================
// V7: transition あり → variable 必須
// =============================================================

mod v7_tests {
    use super::*;

    #[test]
    fn transition_without_variable_error() {
        let mut doc = minimal_valid_doc();
        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![StoryboardEntry {
                    variable: None, // missing!
                    transition: Some(TransitionRef::Inline(TransitionDef {
                        from: None,
                        to: Some(TransitionValue::Scalar(1.0)),
                        relative_to: None,
                        easing: None,
                        delay: 0.0,
                        duration: Some(1.0),
                    })),
                    at: None,
                    between: None,
                    keyframe: None,
                }],
            },
        );
        doc.storyboard = storyboard;

        let errors = doc.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            DolaError::InvalidEntry { reason, .. }
            if reason.contains("transition requires variable")
        )));
    }
}

// =============================================================
// V8: at と between は排他
// =============================================================

mod v8_tests {
    use super::*;

    #[test]
    fn at_and_between_mutually_exclusive() {
        let mut doc = minimal_valid_doc();
        let mut variable = BTreeMap::new();
        variable.insert(
            "x".to_string(),
            AnimationVariableDef::Float {
                initial: 0.0,
                min: None,
                max: None,
            },
        );
        doc.variable = variable;

        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![
                    // Need a KF first
                    StoryboardEntry {
                        variable: Some("x".to_string()),
                        transition: Some(TransitionRef::Inline(TransitionDef {
                            from: None,
                            to: Some(TransitionValue::Scalar(1.0)),
                            relative_to: None,
                            easing: None,
                            delay: 0.0,
                            duration: Some(1.0),
                        })),
                        at: None,
                        between: None,
                        keyframe: Some("kf1".to_string()),
                    },
                    StoryboardEntry {
                        variable: Some("x".to_string()),
                        transition: Some(TransitionRef::Inline(TransitionDef {
                            from: None,
                            to: Some(TransitionValue::Scalar(1.0)),
                            relative_to: None,
                            easing: None,
                            delay: 0.0,
                            duration: Some(1.0),
                        })),
                        at: Some(KeyframeRef::Single("kf1".to_string())),
                        between: Some(BetweenKeyframes {
                            from: "start".to_string(),
                            to: "kf1".to_string(),
                        }),
                        keyframe: None,
                    },
                ],
            },
        );
        doc.storyboard = storyboard;

        let errors = doc.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            DolaError::InvalidEntry { reason, .. }
            if reason.contains("at and between are mutually exclusive")
        )));
    }
}

// =============================================================
// V9: 純粋KFエントリ（variable/transition なし）→ keyframe 必須
// =============================================================

mod v9_tests {
    use super::*;

    #[test]
    fn empty_entry_without_keyframe_error() {
        let mut doc = minimal_valid_doc();
        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![StoryboardEntry {
                    variable: None,
                    transition: None,
                    at: None,
                    between: None,
                    keyframe: None, // missing!
                }],
            },
        );
        doc.storyboard = storyboard;

        let errors = doc.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            DolaError::InvalidEntry { reason, .. }
            if reason.contains("must have keyframe")
        )));
    }

    #[test]
    fn pure_keyframe_entry_ok() {
        let mut doc = minimal_valid_doc();
        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![StoryboardEntry {
                    variable: None,
                    transition: None,
                    at: None,
                    between: None,
                    keyframe: Some("sync_point".to_string()),
                }],
            },
        );
        doc.storyboard = storyboard;

        assert!(doc.validate().is_ok());
    }
}

// =============================================================
// V10: Object型トランジション制限
// =============================================================

mod v10_tests {
    use super::*;

    #[test]
    fn object_with_from_error() {
        let mut doc = minimal_valid_doc();
        let mut variable = BTreeMap::new();
        variable.insert(
            "bg".to_string(),
            AnimationVariableDef::Object {
                initial: DynamicValue::Null,
            },
        );
        doc.variable = variable;

        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![StoryboardEntry {
                    variable: Some("bg".to_string()),
                    transition: Some(TransitionRef::Inline(TransitionDef {
                        from: Some(TransitionValue::Dynamic(DynamicValue::Null)), // not allowed!
                        to: Some(TransitionValue::Dynamic(DynamicValue::String(
                            "img.png".to_string(),
                        ))),
                        relative_to: None,
                        easing: None,
                        delay: 0.0,
                        duration: None,
                    })),
                    at: None,
                    between: None,
                    keyframe: None,
                }],
            },
        );
        doc.storyboard = storyboard;

        let errors = doc.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            DolaError::ObjectTransitionViolation { field, .. }
            if field == "from"
        )));
    }

    #[test]
    fn object_with_scalar_to_error() {
        let mut doc = minimal_valid_doc();
        let mut variable = BTreeMap::new();
        variable.insert(
            "bg".to_string(),
            AnimationVariableDef::Object {
                initial: DynamicValue::Null,
            },
        );
        doc.variable = variable;

        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![StoryboardEntry {
                    variable: Some("bg".to_string()),
                    transition: Some(TransitionRef::Inline(TransitionDef {
                        from: None,
                        to: Some(TransitionValue::Scalar(1.0)), // Object variable + Scalar = error
                        relative_to: None,
                        easing: None,
                        delay: 0.0,
                        duration: None,
                    })),
                    at: None,
                    between: None,
                    keyframe: None,
                }],
            },
        );
        doc.storyboard = storyboard;

        let errors = doc.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            DolaError::TypeMismatch { reason, .. }
            if reason.contains("Object variable requires Dynamic")
        )));
    }
}

// =============================================================
// V11: to/relative_to 排他
// =============================================================

mod v11_tests {
    use super::*;

    #[test]
    fn to_and_relative_to_mutually_exclusive() {
        let mut doc = minimal_valid_doc();
        let mut variable = BTreeMap::new();
        variable.insert(
            "x".to_string(),
            AnimationVariableDef::Float {
                initial: 0.0,
                min: None,
                max: None,
            },
        );
        doc.variable = variable;

        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![StoryboardEntry {
                    variable: Some("x".to_string()),
                    transition: Some(TransitionRef::Inline(TransitionDef {
                        from: None,
                        to: Some(TransitionValue::Scalar(1.0)),
                        relative_to: Some(50.0), // both specified!
                        easing: None,
                        delay: 0.0,
                        duration: Some(1.0),
                    })),
                    at: None,
                    between: None,
                    keyframe: None,
                }],
            },
        );
        doc.storyboard = storyboard;

        let errors = doc.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, DolaError::MutuallyExclusive { .. })));
    }
}

// =============================================================
// V12: 値域検証
// =============================================================

mod v12_tests {
    use super::*;

    #[test]
    fn initial_above_max_error() {
        let doc = doc_with_float_var("x", 1.5, Some(0.0), Some(1.0));
        let errors = doc.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            DolaError::ValueOutOfRange { variable, field, .. }
            if variable == "x" && field == "initial"
        )));
    }

    #[test]
    fn initial_below_min_error() {
        let doc = doc_with_float_var("x", -1.0, Some(0.0), Some(1.0));
        let errors = doc.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            DolaError::ValueOutOfRange { variable, field, .. }
            if variable == "x" && field == "initial"
        )));
    }

    #[test]
    fn transition_to_out_of_range_error() {
        let mut doc = doc_with_float_var("x", 0.5, Some(0.0), Some(100.0));
        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![StoryboardEntry {
                    variable: Some("x".to_string()),
                    transition: Some(TransitionRef::Inline(TransitionDef {
                        from: None,
                        to: Some(TransitionValue::Scalar(200.0)), // out of range!
                        relative_to: None,
                        easing: None,
                        delay: 0.0,
                        duration: Some(1.0),
                    })),
                    at: None,
                    between: None,
                    keyframe: None,
                }],
            },
        );
        doc.storyboard = storyboard;

        let errors = doc.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            DolaError::ValueOutOfRange { variable, field, .. }
            if variable == "x" && field == "to"
        )));
    }

    #[test]
    fn i64_variable_initial_out_of_range() {
        let mut doc = minimal_valid_doc();
        let mut variable = BTreeMap::new();
        variable.insert(
            "count".to_string(),
            AnimationVariableDef::Integer {
                initial: 200,
                min: Some(0),
                max: Some(100),
                typewriter: None,
            },
        );
        doc.variable = variable;

        let errors = doc.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            DolaError::ValueOutOfRange { variable, .. }
            if variable == "count"
        )));
    }
}

// =============================================================
// V13: 変数型とトランジション値型の整合性
// =============================================================

mod v13_tests {
    use super::*;

    #[test]
    fn float_variable_with_dynamic_to_error() {
        let mut doc = doc_with_float_var("x", 0.0, None, None);
        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![StoryboardEntry {
                    variable: Some("x".to_string()),
                    transition: Some(TransitionRef::Inline(TransitionDef {
                        from: None,
                        to: Some(TransitionValue::Dynamic(DynamicValue::String(
                            "bad".to_string(),
                        ))),
                        relative_to: None,
                        easing: None,
                        delay: 0.0,
                        duration: Some(1.0),
                    })),
                    at: None,
                    between: None,
                    keyframe: None,
                }],
            },
        );
        doc.storyboard = storyboard;

        let errors = doc.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            DolaError::TypeMismatch { reason, .. }
            if reason.contains("Numeric variable requires Scalar")
        )));
    }

    #[test]
    fn object_variable_with_scalar_to_error() {
        let mut doc = minimal_valid_doc();
        let mut variable = BTreeMap::new();
        variable.insert(
            "bg".to_string(),
            AnimationVariableDef::Object {
                initial: DynamicValue::Null,
            },
        );
        doc.variable = variable;

        let mut storyboard = BTreeMap::new();
        storyboard.insert(
            "sb1".to_string(),
            Storyboard {
                time_scale: 1.0,
                loop_count: None,
                interruption_policy: InterruptionPolicy::Conclude,
                entry: vec![StoryboardEntry {
                    variable: Some("bg".to_string()),
                    transition: Some(TransitionRef::Inline(TransitionDef {
                        from: None,
                        to: Some(TransitionValue::Scalar(1.0)), // Object + Scalar = error
                        relative_to: None,
                        easing: None,
                        delay: 0.0,
                        duration: None,
                    })),
                    at: None,
                    between: None,
                    keyframe: None,
                }],
            },
        );
        doc.storyboard = storyboard;

        let errors = doc.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            DolaError::TypeMismatch { reason, .. }
            if reason.contains("Object variable requires Dynamic")
        )));
    }
}
