// TODO: Implement DolaDocumentBuilder, StoryboardBuilder
use crate::document::DolaDocument;
use crate::error::DolaError;
use crate::storyboard::{InterruptionPolicy, Storyboard, StoryboardEntry};
use crate::transition::TransitionDef;
use crate::validate::Validate;
use crate::variable::AnimationVariableDef;
use std::collections::BTreeMap;

/// DolaDocument ビルダー
pub struct DolaDocumentBuilder {
    schema_version: String,
    variable: BTreeMap<String, AnimationVariableDef>,
    transition: BTreeMap<String, TransitionDef>,
    storyboard: BTreeMap<String, Storyboard>,
}

impl DolaDocumentBuilder {
    /// 新しいビルダーを作成
    pub fn new(schema_version: impl Into<String>) -> Self {
        Self {
            schema_version: schema_version.into(),
            variable: BTreeMap::new(),
            transition: BTreeMap::new(),
            storyboard: BTreeMap::new(),
        }
    }

    /// アニメーション変数を追加
    pub fn variable(mut self, name: impl Into<String>, def: AnimationVariableDef) -> Self {
        self.variable.insert(name.into(), def);
        self
    }

    /// トランジションテンプレートを追加
    pub fn transition(mut self, name: impl Into<String>, def: TransitionDef) -> Self {
        self.transition.insert(name.into(), def);
        self
    }

    /// ストーリーボードを追加
    pub fn storyboard(mut self, name: impl Into<String>, sb: Storyboard) -> Self {
        self.storyboard.insert(name.into(), sb);
        self
    }

    /// ドキュメントを構築し、自動的にバリデーションを実行
    pub fn build(self) -> Result<DolaDocument, Vec<DolaError>> {
        let doc = DolaDocument {
            schema_version: self.schema_version,
            variable: self.variable,
            transition: self.transition,
            storyboard: self.storyboard,
        };
        doc.validate()?;
        Ok(doc)
    }
}

/// Storyboard ビルダー
pub struct StoryboardBuilder {
    time_scale: f64,
    loop_count: Option<u32>,
    interruption_policy: InterruptionPolicy,
    entry: Vec<StoryboardEntry>,
}

impl StoryboardBuilder {
    /// 新しいビルダーを作成
    pub fn new() -> Self {
        Self {
            time_scale: 1.0,
            loop_count: None,
            interruption_policy: InterruptionPolicy::Conclude,
            entry: Vec::new(),
        }
    }

    /// 再生速度倍率を設定
    pub fn time_scale(mut self, scale: f64) -> Self {
        self.time_scale = scale;
        self
    }

    /// ループ回数を設定
    pub fn loop_count(mut self, count: u32) -> Self {
        self.loop_count = Some(count);
        self
    }

    /// 割り込み終了戦略を設定
    pub fn interruption_policy(mut self, policy: InterruptionPolicy) -> Self {
        self.interruption_policy = policy;
        self
    }

    /// エントリを追加
    pub fn entry(mut self, entry: StoryboardEntry) -> Self {
        self.entry.push(entry);
        self
    }

    /// ストーリーボードを構築
    pub fn build(self) -> Storyboard {
        Storyboard {
            time_scale: self.time_scale,
            loop_count: self.loop_count,
            interruption_policy: self.interruption_policy,
            entry: self.entry,
        }
    }
}

impl Default for StoryboardBuilder {
    fn default() -> Self {
        Self::new()
    }
}
