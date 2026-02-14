// TODO: Implement Validation
use std::collections::BTreeSet;

use crate::document::DolaDocument;
use crate::error::DolaError;
use crate::storyboard::{KeyframeNames, KeyframeRef};
use crate::transition::{TransitionRef, TransitionValue};
use crate::variable::AnimationVariableDef;

/// 期待されるスキーマバージョン
const EXPECTED_SCHEMA_VERSION: &str = "1.0";

/// DolaDocument のバリデーション
pub trait Validate {
    /// ドキュメント全体を検証し、すべてのエラーを収集して返す
    fn validate(&self) -> Result<(), Vec<DolaError>>;
}

impl Validate for DolaDocument {
    fn validate(&self) -> Result<(), Vec<DolaError>> {
        let mut errors = Vec::new();

        // V1: スキーマバージョン検証
        validate_schema_version(self, &mut errors);

        // V12: 変数初期値の値域検証
        validate_variable_ranges(self, &mut errors);

        // Storyboard ごとの検証
        for (sb_name, sb) in &self.storyboard {
            // V2: キーフレーム名重複検出
            validate_duplicate_keyframes(sb_name, sb, &mut errors);

            // V3: 予約キーフレーム名検証
            validate_reserved_keyframe_names(sb_name, sb, &mut errors);

            // V6: キーフレーム参照検証（前方参照許可 + 暗黙的KF追跡）
            validate_keyframe_references(sb_name, sb, &mut errors);

            for (entry_idx, entry) in sb.entry.iter().enumerate() {
                // V4: 変数参照存在確認
                if let Some(ref var_name) = entry.variable {
                    if !self.variable.contains_key(var_name) {
                        errors.push(DolaError::UndefinedVariable {
                            storyboard: sb_name.clone(),
                            entry_index: entry_idx,
                            name: var_name.clone(),
                        });
                    }
                }

                // V5: トランジション名前参照存在確認
                if let Some(TransitionRef::Named(ref trans_name)) = entry.transition {
                    if !self.transition.contains_key(trans_name) {
                        errors.push(DolaError::UndefinedTransition {
                            storyboard: sb_name.clone(),
                            entry_index: entry_idx,
                            name: trans_name.clone(),
                        });
                    }
                }

                // V7: transition あり → variable 必須
                if entry.transition.is_some() && entry.variable.is_none() {
                    errors.push(DolaError::InvalidEntry {
                        storyboard: sb_name.clone(),
                        entry_index: entry_idx,
                        reason: "transition requires variable".to_string(),
                    });
                }

                // V8: at と between は排他
                if entry.at.is_some() && entry.between.is_some() {
                    errors.push(DolaError::InvalidEntry {
                        storyboard: sb_name.clone(),
                        entry_index: entry_idx,
                        reason: "at and between are mutually exclusive".to_string(),
                    });
                }

                // V9: 純粋KFエントリ（variable/transition なし）→ keyframe 必須
                if entry.variable.is_none() && entry.transition.is_none() && entry.keyframe.is_none()
                {
                    errors.push(DolaError::InvalidEntry {
                        storyboard: sb_name.clone(),
                        entry_index: entry_idx,
                        reason: "entry without variable/transition must have keyframe".to_string(),
                    });
                }

                // V10, V11, V13: トランジション内容の検証
                let resolved_transition = match &entry.transition {
                    Some(TransitionRef::Inline(def)) => Some(def),
                    Some(TransitionRef::Named(name)) => self.transition.get(name),
                    None => None,
                };

                if let Some(trans_def) = resolved_transition {
                    // V11: to と relative_to 排他
                    if trans_def.to.is_some() && trans_def.relative_to.is_some() {
                        errors.push(DolaError::MutuallyExclusive {
                            storyboard: sb_name.clone(),
                            entry_index: entry_idx,
                        });
                    }

                    // V10, V13: 変数型に基づくトランジション制約
                    if let Some(ref var_name) = entry.variable {
                        if let Some(var_def) = self.variable.get(var_name) {
                            validate_transition_type_constraints(
                                sb_name,
                                entry_idx,
                                var_name,
                                var_def,
                                trans_def,
                                &mut errors,
                            );
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// V1: スキーマバージョン検証
fn validate_schema_version(doc: &DolaDocument, errors: &mut Vec<DolaError>) {
    if doc.schema_version != EXPECTED_SCHEMA_VERSION {
        errors.push(DolaError::SchemaVersionMismatch {
            expected: EXPECTED_SCHEMA_VERSION.to_string(),
            found: doc.schema_version.clone(),
        });
    }
}

/// V12: 変数初期値の値域検証
fn validate_variable_ranges(doc: &DolaDocument, errors: &mut Vec<DolaError>) {
    for (var_name, var_def) in &doc.variable {
        match var_def {
            AnimationVariableDef::Float { initial, min, max } => {
                if let Some(min_val) = min {
                    if *initial < *min_val {
                        errors.push(DolaError::ValueOutOfRange {
                            variable: var_name.clone(),
                            field: "initial".to_string(),
                            value: *initial,
                            min: *min_val,
                            max: max.unwrap_or(f64::INFINITY),
                        });
                    }
                }
                if let Some(max_val) = max {
                    if *initial > *max_val {
                        errors.push(DolaError::ValueOutOfRange {
                            variable: var_name.clone(),
                            field: "initial".to_string(),
                            value: *initial,
                            min: min.unwrap_or(f64::NEG_INFINITY),
                            max: *max_val,
                        });
                    }
                }
            }
            AnimationVariableDef::Integer { initial, min, max, .. } => {
                if let Some(min_val) = min {
                    if *initial < *min_val {
                        errors.push(DolaError::ValueOutOfRange {
                            variable: var_name.clone(),
                            field: "initial".to_string(),
                            value: *initial as f64,
                            min: *min_val as f64,
                            max: max.unwrap_or(i64::MAX) as f64,
                        });
                    }
                }
                if let Some(max_val) = max {
                    if *initial > *max_val {
                        errors.push(DolaError::ValueOutOfRange {
                            variable: var_name.clone(),
                            field: "initial".to_string(),
                            value: *initial as f64,
                            min: min.unwrap_or(i64::MIN) as f64,
                            max: *max_val as f64,
                        });
                    }
                }
            }
            AnimationVariableDef::Object { .. } => {
                // Object 型には値域検証なし
            }
        }
    }
}

/// V2: キーフレーム名重複検出（明示的 keyframe フィールドのみ対象）
fn validate_duplicate_keyframes(
    sb_name: &str,
    sb: &crate::storyboard::Storyboard,
    errors: &mut Vec<DolaError>,
) {
    let mut seen = BTreeSet::new();
    for entry in &sb.entry {
        if let Some(ref kf_name) = entry.keyframe {
            if !seen.insert(kf_name.clone()) {
                errors.push(DolaError::DuplicateKeyframe {
                    storyboard: sb_name.to_string(),
                    name: kf_name.clone(),
                });
            }
        }
    }
}

/// V3: 予約キーフレーム名検証（"start" 使用禁止）
fn validate_reserved_keyframe_names(
    _sb_name: &str,
    sb: &crate::storyboard::Storyboard,
    errors: &mut Vec<DolaError>,
) {
    for entry in &sb.entry {
        if let Some(ref kf_name) = entry.keyframe {
            if kf_name == "start" {
                errors.push(DolaError::ReservedKeyframeName {
                    name: kf_name.clone(),
                });
            }
        }
    }
}

/// V6: キーフレーム参照検証（前方参照許可 + 暗黙的KF追跡）
fn validate_keyframe_references(
    sb_name: &str,
    sb: &crate::storyboard::Storyboard,
    errors: &mut Vec<DolaError>,
) {
    // 第1パス: 全キーフレーム名を収集（明示的 + 暗黙的 + "start" 予約）
    let mut known_keyframes = BTreeSet::new();
    known_keyframes.insert("start".to_string());

    for (idx, entry) in sb.entry.iter().enumerate() {
        if let Some(ref kf_name) = entry.keyframe {
            known_keyframes.insert(kf_name.clone());
        } else {
            // 暗黙的KF: __implicit_{index}
            known_keyframes.insert(format!("__implicit_{}", idx));
        }
    }

    // 第2パス: at/between の参照先が存在するか検証
    for entry in &sb.entry {
        if let Some(ref kf_ref) = entry.at {
            let names = collect_keyframe_names_from_ref(kf_ref);
            for name in names {
                if !known_keyframes.contains(&name) {
                    errors.push(DolaError::UndefinedKeyframe {
                        storyboard: sb_name.to_string(),
                        name,
                    });
                }
            }
        }
        if let Some(ref between) = entry.between {
            if !known_keyframes.contains(&between.from) {
                errors.push(DolaError::UndefinedKeyframe {
                    storyboard: sb_name.to_string(),
                    name: between.from.clone(),
                });
            }
            if !known_keyframes.contains(&between.to) {
                errors.push(DolaError::UndefinedKeyframe {
                    storyboard: sb_name.to_string(),
                    name: between.to.clone(),
                });
            }
        }
    }
}

/// KeyframeRef からキーフレーム名を収集
fn collect_keyframe_names_from_ref(kf_ref: &KeyframeRef) -> Vec<String> {
    match kf_ref {
        KeyframeRef::Single(name) => vec![name.clone()],
        KeyframeRef::Multiple(names) => names.clone(),
        KeyframeRef::WithOffset { keyframes, .. } => match keyframes {
            KeyframeNames::Single(name) => vec![name.clone()],
            KeyframeNames::Multiple(names) => names.clone(),
        },
    }
}

/// V10, V12(transition), V13: 変数型に基づくトランジション制約
fn validate_transition_type_constraints(
    sb_name: &str,
    entry_idx: usize,
    var_name: &str,
    var_def: &AnimationVariableDef,
    trans_def: &crate::transition::TransitionDef,
    errors: &mut Vec<DolaError>,
) {
    match var_def {
        AnimationVariableDef::Object { .. } => {
            // V10: Object 型 → to のみ許可、from/relative_to/easing 不可、to は Dynamic のみ
            if trans_def.from.is_some() {
                errors.push(DolaError::ObjectTransitionViolation {
                    storyboard: sb_name.to_string(),
                    entry_index: entry_idx,
                    field: "from".to_string(),
                });
            }
            if trans_def.relative_to.is_some() {
                errors.push(DolaError::ObjectTransitionViolation {
                    storyboard: sb_name.to_string(),
                    entry_index: entry_idx,
                    field: "relative_to".to_string(),
                });
            }
            if trans_def.easing.is_some() {
                errors.push(DolaError::ObjectTransitionViolation {
                    storyboard: sb_name.to_string(),
                    entry_index: entry_idx,
                    field: "easing".to_string(),
                });
            }
            if let Some(ref to) = trans_def.to {
                if matches!(to, TransitionValue::Scalar(_)) {
                    errors.push(DolaError::TypeMismatch {
                        storyboard: sb_name.to_string(),
                        entry_index: entry_idx,
                        reason: "Object variable requires Dynamic transition value".to_string(),
                    });
                }
            }
        }
        AnimationVariableDef::Float { .. } | AnimationVariableDef::Integer { .. } => {
            // V13: f64/i64 変数 → from/to は Scalar のみ
            if let Some(ref from) = trans_def.from {
                if matches!(from, TransitionValue::Dynamic(_)) {
                    errors.push(DolaError::TypeMismatch {
                        storyboard: sb_name.to_string(),
                        entry_index: entry_idx,
                        reason: "Numeric variable requires Scalar transition value for 'from'"
                            .to_string(),
                    });
                }
            }
            if let Some(ref to) = trans_def.to {
                if matches!(to, TransitionValue::Dynamic(_)) {
                    errors.push(DolaError::TypeMismatch {
                        storyboard: sb_name.to_string(),
                        entry_index: entry_idx,
                        reason: "Numeric variable requires Scalar transition value for 'to'"
                            .to_string(),
                    });
                }
            }

            // V12: トランジション from/to の値域検証
            let (min_f, max_f) = match var_def {
                AnimationVariableDef::Float { min, max, .. } => {
                    (min.unwrap_or(f64::NEG_INFINITY), max.unwrap_or(f64::INFINITY))
                }
                AnimationVariableDef::Integer { min, max, .. } => {
                    (
                        min.map(|v| v as f64).unwrap_or(f64::NEG_INFINITY),
                        max.map(|v| v as f64).unwrap_or(f64::INFINITY),
                    )
                }
                _ => unreachable!(),
            };

            if let Some(TransitionValue::Scalar(val)) = &trans_def.from {
                if *val < min_f || *val > max_f {
                    errors.push(DolaError::ValueOutOfRange {
                        variable: var_name.to_string(),
                        field: "from".to_string(),
                        value: *val,
                        min: min_f,
                        max: max_f,
                    });
                }
            }
            if let Some(TransitionValue::Scalar(val)) = &trans_def.to {
                if *val < min_f || *val > max_f {
                    errors.push(DolaError::ValueOutOfRange {
                        variable: var_name.to_string(),
                        field: "to".to_string(),
                        value: *val,
                        min: min_f,
                        max: max_f,
                    });
                }
            }
        }
    }
}
