//! Integration tests — JSON/TOML/YAML round-trip and E2E tests
//! Tasks 11.2, 11.3, 11.4, 12.1, 12.2

use dola::*;
use std::collections::BTreeMap;

/// 完全な DolaDocument を構築（テスト用）
fn build_complete_document() -> DolaDocument {
    DolaDocumentBuilder::new("1.0")
        // 3変数
        .variable(
            "opacity",
            AnimationVariableDef::Float {
                initial: 0.0,
                min: Some(0.0),
                max: Some(1.0),
            },
        )
        .variable(
            "char_count",
            AnimationVariableDef::Integer {
                initial: 0,
                min: Some(0),
                max: None,
                typewriter: Some("こんにちは世界".to_string()),
            },
        )
        .variable(
            "bg_image",
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
        )
        // 2トランジション
        .transition(
            "fade_in",
            TransitionDef {
                from: None,
                to: Some(TransitionValue::Scalar(1.0)),
                relative_to: None,
                easing: Some(EasingFunction::Named(EasingName::QuadraticInOut)),
                delay: 0.0,
                duration: Some(1.5),
            },
        )
        .transition(
            "typewrite",
            TransitionDef {
                from: None,
                to: Some(TransitionValue::Scalar(7.0)),
                relative_to: None,
                easing: Some(EasingFunction::Named(EasingName::Linear)),
                delay: 0.0,
                duration: Some(3.0),
            },
        )
        // SB1: greeting — 3つの配置パターン
        .storyboard(
            "greeting",
            StoryboardBuilder::new()
                .time_scale(1.0)
                // Entry 1: 前エントリ連結
                .entry(StoryboardEntry {
                    variable: Some("opacity".to_string()),
                    transition: Some(TransitionRef::Named("fade_in".to_string())),
                    at: None,
                    between: None,
                    keyframe: Some("visible".to_string()),
                })
                // Entry 2: KF起点 (at = "visible")
                .entry(StoryboardEntry {
                    variable: Some("char_count".to_string()),
                    transition: Some(TransitionRef::Named("typewrite".to_string())),
                    at: Some(KeyframeRef::Single("visible".to_string())),
                    between: None,
                    keyframe: Some("text_done".to_string()),
                })
                // Entry 3: Object型インライントランジション
                .entry(StoryboardEntry {
                    variable: Some("bg_image".to_string()),
                    transition: Some(TransitionRef::Inline(TransitionDef {
                        from: None,
                        to: Some(TransitionValue::Dynamic(DynamicValue::Map({
                            let mut m = BTreeMap::new();
                            m.insert(
                                "path".to_string(),
                                DynamicValue::String("smile.png".to_string()),
                            );
                            m
                        }))),
                        relative_to: None,
                        easing: None,
                        delay: 0.0,
                        duration: None,
                    })),
                    at: Some(KeyframeRef::Single("text_done".to_string())),
                    between: None,
                    keyframe: None,
                })
                .build(),
        )
        // SB2: sync_test — KF間 + 純粋KF
        .storyboard(
            "sync_test",
            StoryboardBuilder::new()
                // Entry 1: 純粋KF
                .entry(StoryboardEntry {
                    variable: None,
                    transition: None,
                    at: None,
                    between: None,
                    keyframe: Some("marker_a".to_string()),
                })
                // Entry 2: 前エントリ連結
                .entry(StoryboardEntry {
                    variable: Some("opacity".to_string()),
                    transition: Some(TransitionRef::Inline(TransitionDef {
                        from: Some(TransitionValue::Scalar(0.0)),
                        to: Some(TransitionValue::Scalar(1.0)),
                        relative_to: None,
                        easing: Some(EasingFunction::Named(EasingName::Linear)),
                        delay: 0.0,
                        duration: Some(2.0),
                    })),
                    at: None,
                    between: None,
                    keyframe: Some("marker_b".to_string()),
                })
                // Entry 3: KF間 (between)
                .entry(StoryboardEntry {
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
                        from: "marker_a".to_string(),
                        to: "marker_b".to_string(),
                    }),
                    keyframe: None,
                })
                .build(),
        )
        .build()
        .unwrap()
}

// =============================================================
// Task 11.2: JSON round-trip
// =============================================================

mod json_integration_tests {
    use super::*;

    #[test]
    fn complete_document_json_roundtrip() {
        let doc = build_complete_document();
        let json = serde_json::to_string_pretty(&doc).unwrap();
        let deserialized: DolaDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, deserialized);
    }

    #[test]
    fn easing_name_snake_case_in_json() {
        let doc = build_complete_document();
        let json = serde_json::to_string_pretty(&doc).unwrap();
        assert!(json.contains("quadratic_in_out"));
    }

    #[test]
    fn interruption_policy_snake_case_in_json() {
        let sb = StoryboardBuilder::new()
            .interruption_policy(InterruptionPolicy::Conclude)
            .entry(StoryboardEntry {
                variable: None,
                transition: None,
                at: None,
                between: None,
                keyframe: Some("kf".to_string()),
            })
            .build();
        let json = serde_json::to_string(&sb).unwrap();
        assert!(json.contains("conclude"));
    }
}

// =============================================================
// Task 11.3: TOML round-trip (feature "toml")
// =============================================================

#[cfg(feature = "toml")]
mod toml_integration_tests {
    use super::*;

    #[test]
    fn complete_document_toml_roundtrip() {
        let doc = build_complete_document();
        let toml_str = toml::to_string_pretty(&doc).unwrap();
        let deserialized: DolaDocument = toml::from_str(&toml_str).unwrap();
        assert_eq!(doc, deserialized);
    }

    #[test]
    fn btreemap_key_order_deterministic_toml() {
        let doc = build_complete_document();
        let toml1 = toml::to_string_pretty(&doc).unwrap();
        let toml2 = toml::to_string_pretty(&doc).unwrap();
        assert_eq!(toml1, toml2, "BTreeMap key order must be deterministic");
    }
}

// =============================================================
// Task 11.4: YAML round-trip (feature "yaml")
// =============================================================

#[cfg(feature = "yaml")]
mod yaml_integration_tests {
    use super::*;

    #[test]
    fn complete_document_yaml_roundtrip() {
        let doc = build_complete_document();
        let yaml_str = serde_yaml::to_string(&doc).unwrap();
        let deserialized: DolaDocument = serde_yaml::from_str(&yaml_str).unwrap();
        assert_eq!(doc, deserialized);
    }
}

// =============================================================
// Task 12.1: E2E — 全配置パターン統合
// =============================================================

mod e2e_tests {
    use super::*;

    #[test]
    fn builder_validate_serialize_deserialize_revalidate() {
        // Builder API → build (validate) → serialize → deserialize → validate again
        let doc = build_complete_document();
        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: DolaDocument = serde_json::from_str(&json).unwrap();
        // Re-validate
        assert!(deserialized.validate().is_ok());
        assert_eq!(doc, deserialized);
    }

    #[test]
    fn implicit_keyframe_chain_pattern() {
        // Test that omitting keyframe still allows chain pattern (via implicit KFs)
        let doc = DolaDocumentBuilder::new("1.0")
            .variable(
                "x",
                AnimationVariableDef::Float {
                    initial: 0.0,
                    min: None,
                    max: None,
                },
            )
            .storyboard(
                "chain",
                StoryboardBuilder::new()
                    .entry(StoryboardEntry {
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
                        keyframe: None, // implicit KF
                    })
                    .entry(StoryboardEntry {
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
                        keyframe: None, // implicit KF
                    })
                    .build(),
            )
            .build()
            .unwrap();

        assert!(doc.validate().is_ok());
    }
}

// =============================================================
// Task 12.2: エッジケーステスト
// =============================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn empty_storyboard_ok() {
        let doc = DolaDocumentBuilder::new("1.0")
            .storyboard("empty", StoryboardBuilder::new().build())
            .build()
            .unwrap();
        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: DolaDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, deserialized);
    }

    #[test]
    fn pure_keyframe_only_storyboard() {
        let doc = DolaDocumentBuilder::new("1.0")
            .storyboard(
                "kf_only",
                StoryboardBuilder::new()
                    .entry(StoryboardEntry {
                        variable: None,
                        transition: None,
                        at: None,
                        between: None,
                        keyframe: Some("marker".to_string()),
                    })
                    .build(),
            )
            .build()
            .unwrap();
        assert!(doc.validate().is_ok());
    }

    #[test]
    fn typewriter_variable_with_transition() {
        let doc = DolaDocumentBuilder::new("1.0")
            .variable(
                "chars",
                AnimationVariableDef::Integer {
                    initial: 0,
                    min: Some(0),
                    max: None,
                    typewriter: Some("こんにちは".to_string()),
                },
            )
            .storyboard(
                "type_sb",
                StoryboardBuilder::new()
                    .entry(StoryboardEntry {
                        variable: Some("chars".to_string()),
                        transition: Some(TransitionRef::Inline(TransitionDef {
                            from: None,
                            to: Some(TransitionValue::Scalar(5.0)),
                            relative_to: None,
                            easing: Some(EasingFunction::Named(EasingName::Linear)),
                            delay: 0.0,
                            duration: Some(3.0),
                        })),
                        at: None,
                        between: None,
                        keyframe: None,
                    })
                    .build(),
            )
            .build()
            .unwrap();
        assert!(doc.validate().is_ok());
    }

    #[test]
    fn bezier_easing_inline_transition() {
        let doc = DolaDocumentBuilder::new("1.0")
            .variable(
                "x",
                AnimationVariableDef::Float {
                    initial: 0.0,
                    min: None,
                    max: None,
                },
            )
            .storyboard(
                "bezier_sb",
                StoryboardBuilder::new()
                    .entry(StoryboardEntry {
                        variable: Some("x".to_string()),
                        transition: Some(TransitionRef::Inline(TransitionDef {
                            from: None,
                            to: Some(TransitionValue::Scalar(100.0)),
                            relative_to: None,
                            easing: Some(EasingFunction::Parametric(
                                ParametricEasing::CubicBezier {
                                    x0: 0.0,
                                    x1: 0.42,
                                    x2: 0.58,
                                    x3: 1.0,
                                },
                            )),
                            delay: 0.0,
                            duration: Some(2.0),
                        })),
                        at: None,
                        between: None,
                        keyframe: None,
                    })
                    .build(),
            )
            .build()
            .unwrap();

        let json = serde_json::to_string_pretty(&doc).unwrap();
        assert!(json.contains("cubic_bezier"));
        let deserialized: DolaDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, deserialized);
    }

    #[test]
    fn delay_only_instant_transition() {
        let doc = DolaDocumentBuilder::new("1.0")
            .variable(
                "x",
                AnimationVariableDef::Float {
                    initial: 0.0,
                    min: None,
                    max: None,
                },
            )
            .storyboard(
                "delay_sb",
                StoryboardBuilder::new()
                    .entry(StoryboardEntry {
                        variable: Some("x".to_string()),
                        transition: Some(TransitionRef::Inline(TransitionDef {
                            from: None,
                            to: Some(TransitionValue::Scalar(1.0)),
                            relative_to: None,
                            easing: None,
                            delay: 2.0,
                            duration: None, // instant transition after delay
                        })),
                        at: None,
                        between: None,
                        keyframe: None,
                    })
                    .build(),
            )
            .build()
            .unwrap();
        assert!(doc.validate().is_ok());
    }

    #[test]
    fn at_start_keyword() {
        let doc = DolaDocumentBuilder::new("1.0")
            .variable(
                "x",
                AnimationVariableDef::Float {
                    initial: 0.0,
                    min: None,
                    max: None,
                },
            )
            .storyboard(
                "start_sb",
                StoryboardBuilder::new()
                    .entry(StoryboardEntry {
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
                    })
                    .build(),
            )
            .build()
            .unwrap();
        assert!(doc.validate().is_ok());
    }

    #[test]
    fn multiple_keyframe_wait() {
        let doc = DolaDocumentBuilder::new("1.0")
            .variable(
                "x",
                AnimationVariableDef::Float {
                    initial: 0.0,
                    min: None,
                    max: None,
                },
            )
            .storyboard(
                "multi_kf",
                StoryboardBuilder::new()
                    .entry(StoryboardEntry {
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
                        keyframe: Some("a".to_string()),
                    })
                    .entry(StoryboardEntry {
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
                        keyframe: Some("b".to_string()),
                    })
                    .entry(StoryboardEntry {
                        variable: Some("x".to_string()),
                        transition: Some(TransitionRef::Inline(TransitionDef {
                            from: None,
                            to: Some(TransitionValue::Scalar(3.0)),
                            relative_to: None,
                            easing: None,
                            delay: 0.0,
                            duration: Some(1.0),
                        })),
                        at: Some(KeyframeRef::Multiple(vec![
                            "a".to_string(),
                            "b".to_string(),
                        ])),
                        between: None,
                        keyframe: None,
                    })
                    .build(),
            )
            .build()
            .unwrap();
        assert!(doc.validate().is_ok());
    }

    #[test]
    fn keyframe_ref_with_offset() {
        let doc = DolaDocumentBuilder::new("1.0")
            .variable(
                "x",
                AnimationVariableDef::Float {
                    initial: 0.0,
                    min: None,
                    max: None,
                },
            )
            .storyboard(
                "offset_sb",
                StoryboardBuilder::new()
                    .entry(StoryboardEntry {
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
                    })
                    .entry(StoryboardEntry {
                        variable: Some("x".to_string()),
                        transition: Some(TransitionRef::Inline(TransitionDef {
                            from: None,
                            to: Some(TransitionValue::Scalar(2.0)),
                            relative_to: None,
                            easing: None,
                            delay: 0.0,
                            duration: Some(1.0),
                        })),
                        at: Some(KeyframeRef::WithOffset {
                            keyframes: KeyframeNames::Single("visible".to_string()),
                            offset: 0.5,
                        }),
                        between: None,
                        keyframe: None,
                    })
                    .build(),
            )
            .build()
            .unwrap();
        assert!(doc.validate().is_ok());
    }

    #[test]
    fn object_transition_dynamic_value() {
        let doc = DolaDocumentBuilder::new("1.0")
            .variable(
                "bg",
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
            )
            .storyboard(
                "obj_sb",
                StoryboardBuilder::new()
                    .entry(StoryboardEntry {
                        variable: Some("bg".to_string()),
                        transition: Some(TransitionRef::Inline(TransitionDef {
                            from: None,
                            to: Some(TransitionValue::Dynamic(DynamicValue::Map({
                                let mut m = BTreeMap::new();
                                m.insert(
                                    "path".to_string(),
                                    DynamicValue::String("image.png".to_string()),
                                );
                                m
                            }))),
                            relative_to: None,
                            easing: None,
                            delay: 0.0,
                            duration: None,
                        })),
                        at: None,
                        between: None,
                        keyframe: None,
                    })
                    .build(),
            )
            .build()
            .unwrap();
        assert!(doc.validate().is_ok());
    }

    #[test]
    fn value_out_of_range_v12_error() {
        let result = DolaDocumentBuilder::new("1.0")
            .variable(
                "x",
                AnimationVariableDef::Float {
                    initial: 1.5,
                    min: Some(0.0),
                    max: Some(1.0),
                },
            )
            .build();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, DolaError::ValueOutOfRange { .. })));
    }

    #[test]
    fn type_mismatch_v13_error() {
        let result = DolaDocumentBuilder::new("1.0")
            .variable(
                "x",
                AnimationVariableDef::Float {
                    initial: 0.0,
                    min: None,
                    max: None,
                },
            )
            .storyboard(
                "sb",
                StoryboardBuilder::new()
                    .entry(StoryboardEntry {
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
                    })
                    .build(),
            )
            .build();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, DolaError::TypeMismatch { .. })));
    }
}

// =============================================================
// Task 11.1: Feature gates 動作検証（コンパイル時チェック）
// =============================================================

#[test]
fn feature_json_enabled_by_default() {
    // serde_json が利用可能であることを確認（defaultフィーチャーにjson含む）
    let doc = DolaDocument {
        schema_version: "1.0".to_string(),
        variable: BTreeMap::new(),
        transition: BTreeMap::new(),
        storyboard: BTreeMap::new(),
    };
    let _json = serde_json::to_string(&doc).unwrap();
}
