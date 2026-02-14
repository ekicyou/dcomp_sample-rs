// TODO: Implement EasingFunction, EasingName, ParametricEasing
use serde::{Deserialize, Serialize};

/// イージング関数（名前付き or パラメトリック）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EasingFunction {
    /// 名前付きイージング（文字列としてシリアライズ）
    Named(EasingName),
    /// パラメトリックイージング（オブジェクトとしてシリアライズ）
    Parametric(ParametricEasing),
}

/// 名前付きイージング（interpolation::EaseFunction 準拠 + Linear）
/// Rust バリアント名は PascalCase、シリアライズ形式は snake_case
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EasingName {
    Linear,
    QuadraticIn,
    QuadraticOut,
    QuadraticInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    QuarticIn,
    QuarticOut,
    QuarticInOut,
    QuinticIn,
    QuinticOut,
    QuinticInOut,
    SineIn,
    SineOut,
    SineInOut,
    CircularIn,
    CircularOut,
    CircularInOut,
    ExponentialIn,
    ExponentialOut,
    ExponentialInOut,
    ElasticIn,
    ElasticOut,
    ElasticInOut,
    BackIn,
    BackOut,
    BackInOut,
    BounceIn,
    BounceOut,
    BounceInOut,
}

/// パラメトリックイージング（内部タグ "type" で判別）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParametricEasing {
    /// 二次ベジェ補間（interpolation::quad_bez 準拠）
    QuadraticBezier { x0: f64, x1: f64, x2: f64 },
    /// 三次ベジェ補間（interpolation::cub_bez 準拠）
    CubicBezier { x0: f64, x1: f64, x2: f64, x3: f64 },
}
