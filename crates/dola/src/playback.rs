// TODO: Implement PlaybackState, ScheduleRequest
use serde::{Deserialize, Serialize};

/// 再生状態列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    /// 待機（未開始）
    Idle,
    /// 再生中
    Playing,
    /// 一時停止
    Paused,
    /// 完了
    Completed,
    /// キャンセル
    Cancelled,
}

/// スケジューリング指示
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduleRequest {
    /// 対象ストーリーボード名
    pub storyboard: String,
    /// 開始時刻（f64秒、相対時間）
    pub start_time: f64,
}
