//! Typewriter IR (Intermediate Representation) 型定義
//!
//! 2段階IR設計:
//! - Stage 1 IR (TypewriterToken): 外部インターフェース（areka-P0-script-engine と共有）
//! - Stage 2 IR (TimelineItem): 内部タイムライン（グリフ単位に分解済み）

use bevy_ecs::prelude::*;

// ============================================================
// Stage 1 IR - 外部インターフェース
// ============================================================

/// Stage 1 IR - 外部インターフェース
/// Script Engine から受け取る形式
#[derive(Debug, Clone)]
pub enum TypewriterToken {
    /// 表示するテキスト
    Text(String),
    /// ウェイト（f64秒単位、Windows Animation API互換）
    Wait(f64),
    /// イベント発火（対象エンティティの TypewriterEvent を設定）
    FireEvent {
        target: Entity,
        event: TypewriterEventKind,
    },
}

/// イベント通知用 enum Component
/// Changed<TypewriterEvent> で検出、処理後に None へ戻す（set パターン）
/// メモリ戦略: SparseSet（動的変更）
#[derive(Component, Debug, Clone, Default, PartialEq)]
#[component(storage = "SparseSet")]
pub enum TypewriterEvent {
    #[default]
    None,
    /// 表示完了
    Complete,
    /// 一時停止
    Paused,
    /// 再開
    Resumed,
}

/// FireEvent で使用するイベント種別
/// TypewriterEvent との違い: Component ではない純粋なデータ型
#[derive(Debug, Clone, PartialEq)]
pub enum TypewriterEventKind {
    /// 表示完了
    Complete,
    /// 一時停止
    Paused,
    /// 再開
    Resumed,
}

impl From<TypewriterEventKind> for TypewriterEvent {
    fn from(kind: TypewriterEventKind) -> Self {
        match kind {
            TypewriterEventKind::Complete => TypewriterEvent::Complete,
            TypewriterEventKind::Paused => TypewriterEvent::Paused,
            TypewriterEventKind::Resumed => TypewriterEvent::Resumed,
        }
    }
}

// ============================================================
// Stage 2 IR - 内部タイムライン
// ============================================================

/// Stage 2 IR - 内部タイムライン
/// DirectWriteでグリフ単位に分解後の形式
#[derive(Debug)]
pub enum TimelineItem {
    /// グリフ表示（TextLayout内のクラスタ番号）
    Glyph {
        /// クラスタインデックス
        cluster_index: u32,
        /// デフォルトウェイト後の累積時刻
        show_at: f64,
    },
    /// ウェイト
    Wait {
        /// ウェイト秒数
        duration: f64,
        /// ウェイト開始時刻
        start_at: f64,
    },
    /// イベント発火
    FireEvent {
        /// 対象エンティティ
        target: Entity,
        /// 発火するイベント
        event: TypewriterEventKind,
        /// 発火時刻
        fire_at: f64,
    },
}

/// Typewriter タイムライン全体
pub struct TypewriterTimeline {
    /// 全文テキスト
    pub full_text: String,
    /// タイムライン項目
    pub items: Vec<TimelineItem>,
    /// 総再生時間
    pub total_duration: f64,
    /// 総クラスタ数
    pub total_cluster_count: u32,
}

impl TypewriterTimeline {
    /// 空のタイムラインを作成
    pub fn empty() -> Self {
        Self {
            full_text: String::new(),
            items: Vec::new(),
            total_duration: 0.0,
            total_cluster_count: 0,
        }
    }
}
