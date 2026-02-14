// TODO: Implement TransitionDef, TransitionRef, TransitionValue
use serde::{Deserialize, Serialize};

use crate::easing::EasingFunction;
use crate::value::DynamicValue;

/// トランジションの開始値・終了値を表す型
/// serde 動作: #[serde(untagged)] により Scalar(f64) を先に試行。
/// 数値は Scalar、オブジェクト構造は Dynamic にマッピング。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransitionValue {
    /// スカラー値（f64/i64 変数向け）
    Scalar(f64),
    /// オブジェクト値（Object 型変数向け、補間なし）
    Dynamic(DynamicValue),
}

/// トランジション定義
///
/// 不変条件:
/// - to と relative_to は排他（同時指定不可。V11）
/// - f64/i64 型変数: from/to は TransitionValue::Scalar のみ（V13）。relative_to 使用可
/// - Object 型変数: to（TransitionValue::Dynamic）のみ。from/relative_to/easing は不可（V10）
/// - 総時間 = delay + duration（duration 省略時は即時 = delay 後即座に切り替え）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransitionDef {
    /// 開始値（省略時は配置時点の変数の現在値）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<TransitionValue>,
    /// 終了値（relative_to と排他）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<TransitionValue>,
    /// 相対終了値（開始値からのオフセット。f64 のみ。to と排他）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relative_to: Option<f64>,
    /// イージング種別（f64/i64 のみ。Object には適用不可）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub easing: Option<EasingFunction>,
    /// トランジション前待機時間（f64秒、デフォルト 0）
    #[serde(default)]
    pub delay: f64,
    /// 遷移持続時間（f64秒、省略時は即時遷移）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
}

/// トランジション参照（ハイブリッド: 名前文字列 or インライン定義）
/// serde 動作: 文字列→Named、オブジェクト→Inline を順番に試行
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransitionRef {
    /// 名前付きテンプレートへの参照
    Named(String),
    /// インライントランジション定義
    Inline(TransitionDef),
}
