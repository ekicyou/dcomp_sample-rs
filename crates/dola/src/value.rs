// TODO: Implement DynamicValue
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// フォーマット非依存の動的値型（JSON/TOML/YAML 共通）
/// バリアント順序: Integer を Float より前に定義し、TOML の整数/浮動小数点区別を保持
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DynamicValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<DynamicValue>),
    Map(BTreeMap<String, DynamicValue>),
}
