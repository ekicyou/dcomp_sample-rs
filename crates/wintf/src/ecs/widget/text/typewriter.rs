//! Typewriter コンポーネント定義
//!
//! - Typewriter: ウィジェット論理コンポーネント（永続）
//! - TypewriterTalk: 1回のトーク（再生中のみ存在、終了で解放）

use crate::com::dwrite::DWriteTextLayoutExt;
use crate::ecs::graphics::AnimationCore;
use crate::ecs::widget::text::label::TextDirection;
use crate::ecs::widget::text::typewriter_ir::{
    TimelineItem, TypewriterEventKind, TypewriterTimeline, TypewriterToken,
};
use crate::ecs::Visual;
use bevy_ecs::component::Component;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::world::DeferredWorld;
use tracing::trace;
use windows::core::Result;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use windows::Win32::Graphics::DirectWrite::IDWriteTextLayout;

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
    pub color: Color,
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
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            direction: TextDirection::default(),
            default_char_wait: 0.05, // 50ms
        }
    }
}

/// Typewriter追加時のフック: Visualコンポーネントを自動挿入
fn on_typewriter_add(mut world: DeferredWorld, hook: HookContext) {
    if world.get::<Visual>(hook.entity).is_some() {
        return;
    }
    world
        .commands()
        .entity(hook.entity)
        .insert(Visual::default());
}

/// Typewriter削除時のフック
fn on_typewriter_remove(_world: DeferredWorld, hook: DeferredHook) {
    trace!(entity = ?hook.entity, "[Typewriter] Removed");
}

// ============================================================
// TypewriterTalk - 1回のトーク（再生中のみ存在）
// ============================================================

/// 再生状態
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TypewriterState {
    #[default]
    Playing,
    Paused,
    Completed,
}

/// 1回のトーク（再生中のみ存在）
/// トーク完了・クリア時に remove される
/// メモリ戦略: SparseSet（動的追加/削除）
#[derive(Component)]
#[component(storage = "SparseSet", on_remove = on_typewriter_talk_remove)]
pub struct TypewriterTalk {
    // === リソース ===
    /// TextLayout（このトーク用、描画に使用）
    text_layout: IDWriteTextLayout,
    /// Stage 2 IR タイムライン
    timeline: TypewriterTimeline,

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
    /// Stage 1 IR から TypewriterTalk を生成
    ///
    /// # Arguments
    /// * `tokens` - Stage 1 IR トークン列
    /// * `typewriter` - Typewriter コンポーネント（スタイル設定取得用）
    /// * `text_layout` - 作成済みの TextLayout
    /// * `current_time` - 現在時刻（AnimationCore.get_time()から取得）
    pub fn new(
        tokens: Vec<TypewriterToken>,
        typewriter: &Typewriter,
        text_layout: IDWriteTextLayout,
        current_time: f64,
    ) -> Result<Self> {
        // Stage 1 → Stage 2 IR 変換
        let timeline = Self::convert_to_timeline(tokens, typewriter, &text_layout)?;

        Ok(Self {
            text_layout,
            timeline,
            state: TypewriterState::Playing,
            start_time: current_time,
            paused_elapsed: 0.0,
            visible_cluster_count: 0,
            progress: 0.0,
            next_item_index: 0,
        })
    }

    /// Stage 1 → Stage 2 IR 変換
    fn convert_to_timeline(
        tokens: Vec<TypewriterToken>,
        typewriter: &Typewriter,
        text_layout: &IDWriteTextLayout,
    ) -> Result<TypewriterTimeline> {
        let cluster_metrics = text_layout.get_cluster_metrics()?;
        let total_cluster_count = cluster_metrics.len() as u32;

        let mut full_text = String::new();
        let mut items = Vec::new();
        let mut current_time = 0.0;
        let mut cluster_index = 0u32;

        for token in tokens {
            match token {
                TypewriterToken::Text(text) => {
                    full_text.push_str(&text);

                    // テキスト内の各クラスタをタイムラインに追加
                    // クラスタ数はDirectWriteで決定されるため、文字数ではなくクラスタ単位で処理
                    let char_count = text.chars().count();
                    for _ in 0..char_count {
                        if cluster_index < total_cluster_count {
                            // デフォルトウェイトを加算
                            current_time += typewriter.default_char_wait;
                            items.push(TimelineItem::Glyph {
                                cluster_index,
                                show_at: current_time,
                            });
                            cluster_index += 1;
                        }
                    }
                }
                TypewriterToken::Wait(duration) => {
                    items.push(TimelineItem::Wait {
                        duration,
                        start_at: current_time,
                    });
                    current_time += duration;
                }
                TypewriterToken::FireEvent { target, event } => {
                    items.push(TimelineItem::FireEvent {
                        target,
                        event,
                        fire_at: current_time,
                    });
                }
            }
        }

        Ok(TypewriterTimeline {
            full_text,
            items,
            total_duration: current_time,
            total_cluster_count,
        })
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

    /// 全文即時表示
    pub fn skip(&mut self) {
        self.visible_cluster_count = self.timeline.total_cluster_count;
        self.progress = 1.0;
        self.state = TypewriterState::Completed;
    }

    // === 状態取得 ===

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

    /// TextLayout参照
    pub fn text_layout(&self) -> &IDWriteTextLayout {
        &self.text_layout
    }

    /// タイムライン参照
    pub fn timeline(&self) -> &TypewriterTimeline {
        &self.timeline
    }

    // === 内部更新 ===

    /// 現在時刻に基づいて状態を更新
    ///
    /// # Returns
    /// 発火すべきイベントのリスト
    pub fn update(&mut self, current_time: f64) -> Vec<(bevy_ecs::entity::Entity, TypewriterEventKind)> {
        if self.state != TypewriterState::Playing {
            return Vec::new();
        }

        let elapsed = current_time - self.start_time;
        let mut events_to_fire = Vec::new();

        // タイムラインを走査して表示状態を更新
        while self.next_item_index < self.timeline.items.len() {
            let item = &self.timeline.items[self.next_item_index];
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
        if self.timeline.total_cluster_count > 0 {
            self.progress =
                self.visible_cluster_count as f32 / self.timeline.total_cluster_count as f32;
        } else {
            self.progress = 1.0;
        }

        // 全クラスタ表示完了で Completed に遷移
        if self.visible_cluster_count >= self.timeline.total_cluster_count {
            self.state = TypewriterState::Completed;
        }

        events_to_fire
    }
}

fn on_typewriter_talk_remove(_world: DeferredWorld, hook: DeferredHook) {
    trace!(entity = ?hook.entity, "[TypewriterTalk] Removed - resources released");
}
