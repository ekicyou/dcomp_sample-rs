// TODO: Implement DolaDocument
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::storyboard::Storyboard;
use crate::transition::TransitionDef;
use crate::variable::AnimationVariableDef;

/// Dola ドキュメントのルートコンテナ
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DolaDocument {
    /// スキーマバージョン（例: "1.0"）
    pub schema_version: String,
    /// 名前付きアニメーション変数（グローバルスコープ）
    #[serde(default)]
    pub variable: BTreeMap<String, AnimationVariableDef>,
    /// 名前付きトランジションテンプレート（グローバルスコープ）
    #[serde(default)]
    pub transition: BTreeMap<String, TransitionDef>,
    /// 名前付きストーリーボード
    #[serde(default)]
    pub storyboard: BTreeMap<String, Storyboard>,
}
