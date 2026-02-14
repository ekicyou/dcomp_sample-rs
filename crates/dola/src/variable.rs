// TODO: Implement AnimationVariableDef
use serde::{Deserialize, Serialize};

use crate::value::DynamicValue;

/// アニメーション変数定義（内部タグ方式: "type" フィールドで判別）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnimationVariableDef {
    /// 連続値変数（座標・透明度・角度等）
    #[serde(rename = "f64")]
    Float {
        initial: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
    },
    /// 離散値変数（イージング対応: f64 で補間後 i64 に丸める）
    #[serde(rename = "i64")]
    Integer {
        initial: i64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<i64>,
        /// タイプライター文字列（設定時: initial=0, 終了値=文字列長）
        #[serde(default, skip_serializing_if = "Option::is_none")]
        typewriter: Option<String>,
    },
    /// オブジェクト型変数（補間なし、キーフレームで値切り替え）
    #[serde(rename = "object")]
    Object { initial: DynamicValue },
}
