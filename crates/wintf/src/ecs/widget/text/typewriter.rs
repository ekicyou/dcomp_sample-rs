//! Typewriter コンポーネント定義
//!
//! - Typewriter: ウィジェット論理コンポーネント（永続、スタイル設定）
//! - TypewriterTalk: 1回のトーク論理情報（再生中のみ存在）
//! - TypewriterLayoutCache: 描画リソース（システムが自動生成）

use crate::ecs::widget::text::typewriter_ir::{
    TimelineItem, TypewriterEventKind, TypewriterTimeline, TypewriterToken,
};
use crate::ecs::Visual;
use bevy_ecs::component::Component;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::world::DeferredWorld;
use tracing::trace;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use windows::Win32::Graphics::DirectWrite::IDWriteTextLayout;

// re-export TextDirection from label
pub use crate::ecs::widget::text::label::TextDirection;

// ============================================================
// Typewriter - ウィジェット論理コンポーネント（永続）
// ============================================================

/// テキスト色の型エイリアス
pub type Color = D2D1_COLOR_F;

/// ウィジェット論理コンポーネント（永続）
/// メモリ戦略: SparseSet（動的追加/削除）
#[derive(Component)]
#[component(storage = "SparseSet", on_add = on_typewriter_add, on_remove = on_typewriter_remove)]
pub struct Typewriter {
    // === スタイル設定（Label互換） ===
    pub font_family: String,
    pub font_size: f32,
    /// フォアグラウンド色（テキスト色）
    pub foreground: Color,
    /// バックグラウンド色（None: 透明）
    pub background: Option<Color>,
    pub direction: TextDirection,

    // === デフォルト設定 ===
    /// デフォルト文字間ウェイト（秒）
    pub default_char_wait: f64,
}

impl Default for Typewriter {
    fn default() -> Self {
        Self {
            font_family: "メイリオ".to_string(),
            font_size: 16.0,
            foreground: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            background: None,
            direction: TextDirection::default(),
            default_char_wait: 0.05, // 50ms
        }
    }
}

/// Typewriter追加時のフック: VisualコンポーネントとTyperwriterTalk（空）を自動挿入
fn on_typewriter_add(mut world: DeferredWorld, hook: HookContext) {
    let entity = hook.entity;
    let needs_visual = world.get::<Visual>(entity).is_none();
    let needs_talk = world.get::<TypewriterTalk>(entity).is_none();

    if needs_visual || needs_talk {
        let mut cmds = world.commands();
        let mut entity_cmds = cmds.entity(entity);
        
        if needs_visual {
            entity_cmds.insert(Visual::default());
        }
        if needs_talk {
            // 空のトークを登録（背景描画のため）
            entity_cmds.insert(TypewriterTalk::new(vec![], 0.0));
        }
    }
}

/// Typewriter削除時のフック
fn on_typewriter_remove(_world: DeferredWorld, hook: HookContext) {
    trace!(entity = ?hook.entity, "[Typewriter] Removed");
}

// ============================================================
// TypewriterTalk - 1回のトーク論理情報（再生中のみ存在）
// ============================================================

/// 再生状態
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TypewriterState {
    #[default]
    Playing,
    Paused,
    Completed,
}

/// 1回のトーク論理情報（再生中のみ存在）
/// トーク完了・クリア時に remove される
/// メモリ戦略: SparseSet（動的追加/削除）
///
/// COMリソース（TextLayout）は TypewriterLayoutCache が保持。
/// このコンポーネントは論理情報のみを保持する。
#[derive(Component, Clone)]
#[component(storage = "SparseSet", on_remove = on_typewriter_talk_remove)]
pub struct TypewriterTalk {
    /// Stage 1 IR トークン列
    tokens: Vec<TypewriterToken>,

    // === 再生状態 ===
    state: TypewriterState,
    /// 再生開始時刻
    start_time: f64,
    /// 一時停止時の経過時間
    paused_elapsed: f64,
    /// 現在の表示クラスタ数
    visible_cluster_count: u32,
    /// 進行度（0.0〜1.0）
    progress: f32,
    /// 次に処理するタイムライン項目インデックス
    next_item_index: usize,
}

impl TypewriterTalk {
    /// トークン列と開始時刻から TypewriterTalk を生成
    pub fn new(tokens: Vec<TypewriterToken>, start_time: f64) -> Self {
        Self {
            tokens,
            state: TypewriterState::Playing,
            start_time,
            paused_elapsed: 0.0,
            visible_cluster_count: 0,
            progress: 0.0,
            next_item_index: 0,
        }
    }

    // === 操作 API ===

    /// 一時停止
    pub fn pause(&mut self, current_time: f64) {
        if self.state == TypewriterState::Playing {
            self.paused_elapsed = current_time - self.start_time;
            self.state = TypewriterState::Paused;
        }
    }

    /// 再開
    pub fn resume(&mut self, current_time: f64) {
        if self.state == TypewriterState::Paused {
            self.start_time = current_time - self.paused_elapsed;
            self.state = TypewriterState::Playing;
        }
    }

    /// 全文即時表示（LayoutCache がある場合のみ有効）
    pub fn skip(&mut self, total_cluster_count: u32) {
        self.visible_cluster_count = total_cluster_count;
        self.progress = 1.0;
        self.state = TypewriterState::Completed;
    }

    // === 状態取得 ===

    /// トークン列を取得
    pub fn tokens(&self) -> &[TypewriterToken] {
        &self.tokens
    }

    /// 再生状態を取得
    pub fn state(&self) -> TypewriterState {
        self.state
    }

    /// 進行度を取得 (0.0〜1.0)
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// 表示クラスタ数を取得
    pub fn visible_cluster_count(&self) -> u32 {
        self.visible_cluster_count
    }

    /// 完了しているかどうか
    pub fn is_completed(&self) -> bool {
        self.state == TypewriterState::Completed
    }

    /// 開始時刻を取得
    pub fn start_time(&self) -> f64 {
        self.start_time
    }

    // === 内部更新（TypewriterLayoutCache と連携） ===

    /// 現在時刻に基づいて状態を更新
    ///
    /// # Arguments
    /// * `current_time` - 現在時刻
    /// * `timeline` - Stage 2 IR タイムライン（LayoutCache から取得）
    ///
    /// # Returns
    /// 発火すべきイベントのリスト
    pub fn update(
        &mut self,
        current_time: f64,
        timeline: &TypewriterTimeline,
    ) -> Vec<(bevy_ecs::entity::Entity, TypewriterEventKind)> {
        if self.state != TypewriterState::Playing {
            return Vec::new();
        }

        let elapsed = current_time - self.start_time;
        let mut events_to_fire = Vec::new();

        // タイムラインを走査して表示状態を更新
        while self.next_item_index < timeline.items.len() {
            let item = &timeline.items[self.next_item_index];
            match item {
                TimelineItem::Glyph { show_at, .. } => {
                    if elapsed >= *show_at {
                        self.visible_cluster_count += 1;
                        self.next_item_index += 1;
                    } else {
                        break;
                    }
                }
                TimelineItem::Wait { start_at, duration } => {
                    if elapsed >= *start_at + *duration {
                        self.next_item_index += 1;
                    } else {
                        break;
                    }
                }
                TimelineItem::FireEvent {
                    target,
                    event,
                    fire_at,
                } => {
                    if elapsed >= *fire_at {
                        events_to_fire.push((*target, event.clone()));
                        self.next_item_index += 1;
                    } else {
                        break;
                    }
                }
            }
        }

        // 進行度を更新
        if timeline.total_cluster_count > 0 {
            self.progress = self.visible_cluster_count as f32 / timeline.total_cluster_count as f32;
        } else {
            self.progress = 1.0;
        }

        // 全クラスタ表示完了で Completed に遷移
        if self.visible_cluster_count >= timeline.total_cluster_count {
            self.state = TypewriterState::Completed;
        }

        events_to_fire
    }
}

fn on_typewriter_talk_remove(_world: DeferredWorld, hook: HookContext) {
    trace!(entity = ?hook.entity, "[TypewriterTalk] Removed");
}

// ============================================================
// TypewriterLayoutCache - 描画リソース（システムが自動生成）
// ============================================================

/// Typewriter 描画リソースキャッシュ
///
/// TypewriterTalk 追加時に描画システムが自動生成する。
/// COMリソース（IDWriteTextLayout）と Stage 2 IR を保持。
#[derive(Component)]
#[component(storage = "SparseSet", on_remove = on_layout_cache_remove)]
pub struct TypewriterLayoutCache {
    /// TextLayout（描画に使用）
    text_layout: IDWriteTextLayout,
    /// Stage 2 IR タイムライン
    timeline: TypewriterTimeline,
}

unsafe impl Send for TypewriterLayoutCache {}
unsafe impl Sync for TypewriterLayoutCache {}

impl TypewriterLayoutCache {
    /// 新規作成
    pub fn new(text_layout: IDWriteTextLayout, timeline: TypewriterTimeline) -> Self {
        Self {
            text_layout,
            timeline,
        }
    }

    /// TextLayout参照
    pub fn text_layout(&self) -> &IDWriteTextLayout {
        &self.text_layout
    }

    /// タイムライン参照
    pub fn timeline(&self) -> &TypewriterTimeline {
        &self.timeline
    }
}

impl std::fmt::Debug for TypewriterLayoutCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypewriterLayoutCache")
            .field("timeline", &self.timeline)
            .finish_non_exhaustive()
    }
}

fn on_layout_cache_remove(_world: DeferredWorld, hook: HookContext) {
    trace!(entity = ?hook.entity, "[TypewriterLayoutCache] Removed - COM resources released");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typewriter_default() {
        let tw = Typewriter::default();
        assert_eq!(tw.font_family, "メイリオ");
        assert_eq!(tw.font_size, 16.0);
        assert!((tw.default_char_wait - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_typewriter_state_default() {
        let state = TypewriterState::default();
        assert_eq!(state, TypewriterState::Playing);
    }

    #[test]
    fn test_typewriter_state_transitions() {
        assert_eq!(TypewriterState::Playing, TypewriterState::Playing);
        assert_eq!(TypewriterState::Paused, TypewriterState::Paused);
        assert_eq!(TypewriterState::Completed, TypewriterState::Completed);
        assert_ne!(TypewriterState::Playing, TypewriterState::Paused);
    }
}
