//! Builder API tests
//! Tasks 9.2, 10.2

use dola::*;

// =============================================================
// Task 9.2: DolaDocumentBuilder tests
// =============================================================

mod document_builder_tests {
    use super::*;

    #[test]
    fn minimal_build_ok() {
        let doc = DolaDocumentBuilder::new("1.0").build().unwrap();
        assert_eq!(doc.schema_version, "1.0");
        assert!(doc.variable.is_empty());
        assert!(doc.transition.is_empty());
        assert!(doc.storyboard.is_empty());
    }

    #[test]
    fn build_with_all_components() {
        let doc = DolaDocumentBuilder::new("1.0")
            .variable(
                "opacity",
                AnimationVariableDef::Float {
                    initial: 0.0,
                    min: Some(0.0),
                    max: Some(1.0),
                },
            )
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
            .storyboard(
                "greeting",
                StoryboardBuilder::new()
                    .entry(StoryboardEntry {
                        variable: Some("opacity".to_string()),
                        transition: Some(TransitionRef::Named("fade_in".to_string())),
                        at: None,
                        between: None,
                        keyframe: Some("visible".to_string()),
                    })
                    .build(),
            )
            .build()
            .unwrap();

        // Serialize → deserialize → compare
        let json = serde_json::to_string_pretty(&doc).unwrap();
        let deserialized: DolaDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, deserialized);
    }

    #[test]
    fn build_with_invalid_data_returns_errors() {
        let result = DolaDocumentBuilder::new("1.0")
            .storyboard(
                "sb1",
                StoryboardBuilder::new()
                    .entry(StoryboardEntry {
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
                    })
                    .build(),
            )
            .build();

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, DolaError::UndefinedVariable { .. })));
    }
}

// =============================================================
// Task 10.2: StoryboardBuilder tests
// =============================================================

mod storyboard_builder_tests {
    use super::*;

    #[test]
    fn default_values() {
        let sb = StoryboardBuilder::new().build();
        assert_eq!(sb.time_scale, 1.0);
        assert_eq!(sb.loop_count, None);
        assert_eq!(sb.interruption_policy, InterruptionPolicy::Conclude);
        assert!(sb.entry.is_empty());
    }

    #[test]
    fn entry_added() {
        let sb = StoryboardBuilder::new()
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
                keyframe: Some("kf1".to_string()),
            })
            .build();

        assert_eq!(sb.entry.len(), 1);
        assert_eq!(sb.entry[0].variable, Some("x".to_string()));
    }

    #[test]
    fn custom_meta_fields() {
        let sb = StoryboardBuilder::new()
            .time_scale(2.0)
            .loop_count(3)
            .interruption_policy(InterruptionPolicy::Cancel)
            .build();

        assert_eq!(sb.time_scale, 2.0);
        assert_eq!(sb.loop_count, Some(3));
        assert_eq!(sb.interruption_policy, InterruptionPolicy::Cancel);
    }
}
