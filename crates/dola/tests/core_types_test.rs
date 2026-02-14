//! Unit tests for core data types: DolaDocument, AnimationVariableDef, DynamicValue, DolaError
//! Tasks 2.5, 3.4, 4.4, 5.7, 6.3

use dola::*;
use std::collections::BTreeMap;

// =============================================================
// Task 2.5: DolaDocument / AnimationVariableDef / DynamicValue serde round-trip
// =============================================================

mod document_tests {
    use super::*;

    #[test]
    fn minimal_document_json_roundtrip() {
        let doc = DolaDocument {
            schema_version: "1.0".to_string(),
            variable: BTreeMap::new(),
            transition: BTreeMap::new(),
            storyboard: BTreeMap::new(),
        };
        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: DolaDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, deserialized);
    }

    #[test]
    fn document_with_variables_json_roundtrip() {
        let mut variable = BTreeMap::new();
        variable.insert(
            "opacity".to_string(),
            AnimationVariableDef::Float {
                initial: 0.0,
                min: Some(0.0),
                max: Some(1.0),
            },
        );
        variable.insert(
            "count".to_string(),
            AnimationVariableDef::Integer {
                initial: 0,
                min: Some(0),
                max: Some(100),
                typewriter: None,
            },
        );
        variable.insert(
            "bg".to_string(),
            AnimationVariableDef::Object {
                initial: DynamicValue::Map({
                    let mut m = BTreeMap::new();
                    m.insert(
                        "path".to_string(),
                        DynamicValue::String("default.png".to_string()),
                    );
                    m
                }),
            },
        );

        let doc = DolaDocument {
            schema_version: "1.0".to_string(),
            variable,
            transition: BTreeMap::new(),
            storyboard: BTreeMap::new(),
        };
        let json = serde_json::to_string_pretty(&doc).unwrap();
        let deserialized: DolaDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, deserialized);
    }
}

mod variable_tests {
    use super::*;

    #[test]
    fn float_variable_json_roundtrip() {
        let var = AnimationVariableDef::Float {
            initial: 0.5,
            min: Some(0.0),
            max: Some(1.0),
        };
        let json = serde_json::to_string(&var).unwrap();
        assert!(json.contains(r#""type":"f64""#));
        let deserialized: AnimationVariableDef = serde_json::from_str(&json).unwrap();
        assert_eq!(var, deserialized);
    }

    #[test]
    fn integer_variable_json_roundtrip() {
        let var = AnimationVariableDef::Integer {
            initial: 42,
            min: Some(0),
            max: Some(100),
            typewriter: None,
        };
        let json = serde_json::to_string(&var).unwrap();
        assert!(json.contains(r#""type":"i64""#));
        let deserialized: AnimationVariableDef = serde_json::from_str(&json).unwrap();
        assert_eq!(var, deserialized);
    }

    #[test]
    fn integer_variable_with_typewriter_json_roundtrip() {
        let var = AnimationVariableDef::Integer {
            initial: 0,
            min: Some(0),
            max: None,
            typewriter: Some("こんにちは世界".to_string()),
        };
        let json = serde_json::to_string(&var).unwrap();
        let deserialized: AnimationVariableDef = serde_json::from_str(&json).unwrap();
        assert_eq!(var, deserialized);
    }

    #[test]
    fn object_variable_json_roundtrip() {
        let var = AnimationVariableDef::Object {
            initial: DynamicValue::Map({
                let mut m = BTreeMap::new();
                m.insert(
                    "path".to_string(),
                    DynamicValue::String("image.png".to_string()),
                );
                m
            }),
        };
        let json = serde_json::to_string(&var).unwrap();
        assert!(json.contains(r#""type":"object""#));
        let deserialized: AnimationVariableDef = serde_json::from_str(&json).unwrap();
        assert_eq!(var, deserialized);
    }
}

mod dynamic_value_tests {
    use super::*;

    #[test]
    fn null_roundtrip() {
        let val = DynamicValue::Null;
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "null");
        let deserialized: DynamicValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, deserialized);
    }

    #[test]
    fn bool_roundtrip() {
        let val = DynamicValue::Bool(true);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "true");
        let deserialized: DynamicValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, deserialized);
    }

    #[test]
    fn integer_roundtrip() {
        let val = DynamicValue::Integer(42);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "42");
        let deserialized: DynamicValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, deserialized);
    }

    #[test]
    fn float_roundtrip() {
        let val = DynamicValue::Float(3.14);
        let json = serde_json::to_string(&val).unwrap();
        let deserialized: DynamicValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, deserialized);
    }

    #[test]
    fn string_roundtrip() {
        let val = DynamicValue::String("hello".to_string());
        let json = serde_json::to_string(&val).unwrap();
        let deserialized: DynamicValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, deserialized);
    }

    #[test]
    fn array_roundtrip() {
        let val = DynamicValue::Array(vec![
            DynamicValue::Integer(1),
            DynamicValue::String("two".to_string()),
            DynamicValue::Bool(false),
        ]);
        let json = serde_json::to_string(&val).unwrap();
        let deserialized: DynamicValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, deserialized);
    }

    #[test]
    fn map_roundtrip() {
        let mut m = BTreeMap::new();
        m.insert("key".to_string(), DynamicValue::String("value".to_string()));
        m.insert("num".to_string(), DynamicValue::Integer(99));
        let val = DynamicValue::Map(m);
        let json = serde_json::to_string(&val).unwrap();
        let deserialized: DynamicValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, deserialized);
    }

    #[test]
    fn btreemap_deterministic_order() {
        // Insert keys in reverse order, verify serialized output is alphabetical
        let mut m = BTreeMap::new();
        m.insert("z_key".to_string(), DynamicValue::Integer(1));
        m.insert("a_key".to_string(), DynamicValue::Integer(2));
        m.insert("m_key".to_string(), DynamicValue::Integer(3));
        let val = DynamicValue::Map(m);
        let json = serde_json::to_string(&val).unwrap();
        let a_pos = json.find("a_key").unwrap();
        let m_pos = json.find("m_key").unwrap();
        let z_pos = json.find("z_key").unwrap();
        assert!(a_pos < m_pos);
        assert!(m_pos < z_pos);
    }
}

// =============================================================
// Task 3.4: EasingFunction/EasingName/ParametricEasing serde round-trip
// =============================================================

mod easing_tests {
    use super::*;

    #[test]
    fn all_31_easing_names_json_roundtrip() {
        let names = vec![
            (EasingName::Linear, "linear"),
            (EasingName::QuadraticIn, "quadratic_in"),
            (EasingName::QuadraticOut, "quadratic_out"),
            (EasingName::QuadraticInOut, "quadratic_in_out"),
            (EasingName::CubicIn, "cubic_in"),
            (EasingName::CubicOut, "cubic_out"),
            (EasingName::CubicInOut, "cubic_in_out"),
            (EasingName::QuarticIn, "quartic_in"),
            (EasingName::QuarticOut, "quartic_out"),
            (EasingName::QuarticInOut, "quartic_in_out"),
            (EasingName::QuinticIn, "quintic_in"),
            (EasingName::QuinticOut, "quintic_out"),
            (EasingName::QuinticInOut, "quintic_in_out"),
            (EasingName::SineIn, "sine_in"),
            (EasingName::SineOut, "sine_out"),
            (EasingName::SineInOut, "sine_in_out"),
            (EasingName::CircularIn, "circular_in"),
            (EasingName::CircularOut, "circular_out"),
            (EasingName::CircularInOut, "circular_in_out"),
            (EasingName::ExponentialIn, "exponential_in"),
            (EasingName::ExponentialOut, "exponential_out"),
            (EasingName::ExponentialInOut, "exponential_in_out"),
            (EasingName::ElasticIn, "elastic_in"),
            (EasingName::ElasticOut, "elastic_out"),
            (EasingName::ElasticInOut, "elastic_in_out"),
            (EasingName::BackIn, "back_in"),
            (EasingName::BackOut, "back_out"),
            (EasingName::BackInOut, "back_in_out"),
            (EasingName::BounceIn, "bounce_in"),
            (EasingName::BounceOut, "bounce_out"),
            (EasingName::BounceInOut, "bounce_in_out"),
        ];

        assert_eq!(names.len(), 31, "Must have exactly 31 easing names");

        for (variant, expected_str) in &names {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(json, format!("\"{}\"", expected_str), "Failed for {:?}", variant);

            let deserialized: EasingName = serde_json::from_str(&json).unwrap();
            assert_eq!(*variant, deserialized);
        }
    }

    #[test]
    fn parametric_quadratic_bezier_json_roundtrip() {
        let easing = ParametricEasing::QuadraticBezier {
            x0: 0.0,
            x1: 0.5,
            x2: 1.0,
        };
        let json = serde_json::to_string(&easing).unwrap();
        assert!(json.contains(r#""type":"quadratic_bezier""#));
        let deserialized: ParametricEasing = serde_json::from_str(&json).unwrap();
        assert_eq!(easing, deserialized);
    }

    #[test]
    fn parametric_cubic_bezier_json_roundtrip() {
        let easing = ParametricEasing::CubicBezier {
            x0: 0.0,
            x1: 0.42,
            x2: 0.58,
            x3: 1.0,
        };
        let json = serde_json::to_string(&easing).unwrap();
        assert!(json.contains(r#""type":"cubic_bezier""#));
        let deserialized: ParametricEasing = serde_json::from_str(&json).unwrap();
        assert_eq!(easing, deserialized);
    }

    #[test]
    fn easing_function_named_untagged_deserialize() {
        // 文字列 → Named
        let json = r#""linear""#;
        let ef: EasingFunction = serde_json::from_str(json).unwrap();
        assert_eq!(ef, EasingFunction::Named(EasingName::Linear));
    }

    #[test]
    fn easing_function_parametric_untagged_deserialize() {
        // オブジェクト → Parametric
        let json = r#"{"type":"cubic_bezier","x0":0.0,"x1":0.42,"x2":0.58,"x3":1.0}"#;
        let ef: EasingFunction = serde_json::from_str(json).unwrap();
        assert_eq!(
            ef,
            EasingFunction::Parametric(ParametricEasing::CubicBezier {
                x0: 0.0,
                x1: 0.42,
                x2: 0.58,
                x3: 1.0,
            })
        );
    }
}

// =============================================================
// Task 4.4: TransitionValue/TransitionDef/TransitionRef serde round-trip
// =============================================================

mod transition_tests {
    use super::*;

    #[test]
    fn transition_value_scalar_json_roundtrip() {
        let val = TransitionValue::Scalar(5.0);
        let json = serde_json::to_string(&val).unwrap();
        let deserialized: TransitionValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, deserialized);
    }

    #[test]
    fn transition_value_dynamic_json_roundtrip() {
        let val = TransitionValue::Dynamic(DynamicValue::Map({
            let mut m = BTreeMap::new();
            m.insert(
                "path".to_string(),
                DynamicValue::String("img.png".to_string()),
            );
            m
        }));
        let json = serde_json::to_string(&val).unwrap();
        let deserialized: TransitionValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, deserialized);
    }

    #[test]
    fn transition_def_full_fields_json_roundtrip() {
        let def = TransitionDef {
            from: Some(TransitionValue::Scalar(0.0)),
            to: Some(TransitionValue::Scalar(1.0)),
            relative_to: None,
            easing: Some(EasingFunction::Named(EasingName::QuadraticInOut)),
            delay: 0.5,
            duration: Some(2.0),
        };
        let json = serde_json::to_string(&def).unwrap();
        let deserialized: TransitionDef = serde_json::from_str(&json).unwrap();
        assert_eq!(def, deserialized);
    }

    #[test]
    fn transition_def_relative_to_json_roundtrip() {
        let def = TransitionDef {
            from: None,
            to: None,
            relative_to: Some(50.0),
            easing: Some(EasingFunction::Named(EasingName::Linear)),
            delay: 0.0,
            duration: Some(1.0),
        };
        let json = serde_json::to_string(&def).unwrap();
        let deserialized: TransitionDef = serde_json::from_str(&json).unwrap();
        assert_eq!(def, deserialized);
    }

    #[test]
    fn transition_def_delay_default_json() {
        // delay 省略時はデフォルト 0.0
        let json = r#"{"to":1.0,"duration":1.0}"#;
        let def: TransitionDef = serde_json::from_str(json).unwrap();
        assert_eq!(def.delay, 0.0);
    }

    #[test]
    fn transition_ref_named_json_roundtrip() {
        let tref = TransitionRef::Named("fade_in".to_string());
        let json = serde_json::to_string(&tref).unwrap();
        assert_eq!(json, r#""fade_in""#);
        let deserialized: TransitionRef = serde_json::from_str(&json).unwrap();
        assert_eq!(tref, deserialized);
    }

    #[test]
    fn transition_ref_inline_json_roundtrip() {
        let tref = TransitionRef::Inline(TransitionDef {
            from: None,
            to: Some(TransitionValue::Scalar(1.0)),
            relative_to: None,
            easing: None,
            delay: 0.0,
            duration: Some(1.5),
        });
        let json = serde_json::to_string(&tref).unwrap();
        let deserialized: TransitionRef = serde_json::from_str(&json).unwrap();
        assert_eq!(tref, deserialized);
    }
}

// =============================================================
// Task 5.7: Storyboard/StoryboardEntry/KeyframeRef serde round-trip
// =============================================================

mod storyboard_tests {
    use super::*;

    #[test]
    fn keyframe_ref_single_json_roundtrip() {
        let kf = KeyframeRef::Single("visible".to_string());
        let json = serde_json::to_string(&kf).unwrap();
        assert_eq!(json, r#""visible""#);
        let deserialized: KeyframeRef = serde_json::from_str(&json).unwrap();
        assert_eq!(kf, deserialized);
    }

    #[test]
    fn keyframe_ref_multiple_json_roundtrip() {
        let kf = KeyframeRef::Multiple(vec!["a".to_string(), "b".to_string()]);
        let json = serde_json::to_string(&kf).unwrap();
        let deserialized: KeyframeRef = serde_json::from_str(&json).unwrap();
        assert_eq!(kf, deserialized);
    }

    #[test]
    fn keyframe_ref_with_offset_single_json_roundtrip() {
        let kf = KeyframeRef::WithOffset {
            keyframes: KeyframeNames::Single("visible".to_string()),
            offset: 0.5,
        };
        let json = serde_json::to_string(&kf).unwrap();
        let deserialized: KeyframeRef = serde_json::from_str(&json).unwrap();
        assert_eq!(kf, deserialized);
    }

    #[test]
    fn keyframe_ref_with_offset_multiple_json_roundtrip() {
        let kf = KeyframeRef::WithOffset {
            keyframes: KeyframeNames::Multiple(vec!["a".to_string(), "b".to_string()]),
            offset: 1.0,
        };
        let json = serde_json::to_string(&kf).unwrap();
        let deserialized: KeyframeRef = serde_json::from_str(&json).unwrap();
        assert_eq!(kf, deserialized);
    }

    #[test]
    fn storyboard_entry_chain_pattern_json_roundtrip() {
        // 前エントリ連結: variable + transition のみ
        let entry = StoryboardEntry {
            variable: Some("opacity".to_string()),
            transition: Some(TransitionRef::Named("fade_in".to_string())),
            at: None,
            between: None,
            keyframe: Some("visible".to_string()),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: StoryboardEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn storyboard_entry_at_pattern_json_roundtrip() {
        // KF起点
        let entry = StoryboardEntry {
            variable: Some("char_count".to_string()),
            transition: Some(TransitionRef::Named("typewrite".to_string())),
            at: Some(KeyframeRef::Single("visible".to_string())),
            between: None,
            keyframe: Some("text_done".to_string()),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: StoryboardEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn storyboard_entry_between_pattern_json_roundtrip() {
        // KF間
        let entry = StoryboardEntry {
            variable: Some("opacity".to_string()),
            transition: Some(TransitionRef::Inline(TransitionDef {
                from: None,
                to: Some(TransitionValue::Scalar(0.0)),
                relative_to: None,
                easing: Some(EasingFunction::Named(EasingName::Linear)),
                delay: 0.0,
                duration: None,
            })),
            at: None,
            between: Some(BetweenKeyframes {
                from: "visible".to_string(),
                to: "text_done".to_string(),
            }),
            keyframe: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: StoryboardEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn storyboard_entry_pure_keyframe_json_roundtrip() {
        // 純粋KF
        let entry = StoryboardEntry {
            variable: None,
            transition: None,
            at: None,
            between: None,
            keyframe: Some("sync_point".to_string()),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: StoryboardEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn storyboard_default_values_json() {
        // time_scale=1.0, interruption_policy=Conclude がデフォルト
        let json = r#"{"entry":[]}"#;
        let sb: Storyboard = serde_json::from_str(json).unwrap();
        assert_eq!(sb.time_scale, 1.0);
        assert_eq!(sb.loop_count, None);
        assert_eq!(
            sb.interruption_policy,
            dola::InterruptionPolicy::Conclude
        );
        assert!(sb.entry.is_empty());
    }

    #[test]
    fn interruption_policy_all_variants_json_roundtrip() {
        use dola::InterruptionPolicy;
        let variants = vec![
            (InterruptionPolicy::Cancel, "\"cancel\""),
            (InterruptionPolicy::Conclude, "\"conclude\""),
            (InterruptionPolicy::Trim, "\"trim\""),
            (InterruptionPolicy::Compress, "\"compress\""),
            (InterruptionPolicy::Never, "\"never\""),
        ];
        for (variant, expected) in &variants {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected, "Failed for {:?}", variant);
            let deserialized: InterruptionPolicy = serde_json::from_str(&json).unwrap();
            assert_eq!(*variant, deserialized);
        }
    }
}

// =============================================================
// Task 6.3: PlaybackState/ScheduleRequest serde round-trip
// =============================================================

mod playback_tests {
    use super::*;

    #[test]
    fn playback_state_all_variants_json_roundtrip() {
        let variants = vec![
            PlaybackState::Idle,
            PlaybackState::Playing,
            PlaybackState::Paused,
            PlaybackState::Completed,
            PlaybackState::Cancelled,
        ];
        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: PlaybackState = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn schedule_request_json_roundtrip() {
        let req = ScheduleRequest {
            storyboard: "greeting".to_string(),
            start_time: 1.5,
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: ScheduleRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, deserialized);
    }
}
