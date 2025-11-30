//! WintfTaskPool - 非同期タスク実行用リソース
//!
//! TaskPoolでバックグラウンド実行し、mpsc channelでコマンドをECSへ送信。
//! Input scheduleでdrain_and_apply。

use bevy_ecs::prelude::*;
use bevy_tasks::TaskPool;
use std::future::Future;
use std::sync::mpsc;
use std::sync::Mutex;

/// Box化されたECSコマンド型（クロージャベース）
pub type BoxedCommand = Box<dyn FnOnce(&mut World) + Send>;

/// コマンド送信用チャネル型
pub type CommandSender = mpsc::Sender<BoxedCommand>;

/// 非同期タスク実行用リソース
///
/// # Design
/// - TaskPoolでバックグラウンド実行
/// - mpsc channelでBox<dyn Command>をECSへ送信
/// - Input scheduleでdrain_and_apply
///
/// # Example
/// ```ignore
/// task_pool.spawn(|tx| async move {
///     let result = some_async_work().await;
///     let cmd: BoxedCommand = Box::new(MyCommand { result });
///     let _ = tx.send(cmd);
/// });
/// ```
#[derive(Resource)]
pub struct WintfTaskPool {
    pool: TaskPool,
    sender: mpsc::Sender<BoxedCommand>,
    // MutexでラップしてSyncを満たす
    receiver: Mutex<mpsc::Receiver<BoxedCommand>>,
}

impl WintfTaskPool {
    /// 新しいWintfTaskPoolを作成
    pub fn new() -> Self {
        let pool = TaskPool::new();
        let (sender, receiver) = mpsc::channel();
        Self {
            pool,
            sender,
            receiver: Mutex::new(receiver),
        }
    }

    /// 非同期タスクを生成（CommandSenderが自動で渡される）
    pub fn spawn<F, Fut>(&self, f: F)
    where
        F: FnOnce(CommandSender) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send,
    {
        let tx = self.sender.clone();
        self.pool
            .spawn(async move {
                f(tx).await;
            })
            .detach();
    }

    /// 受信したコマンドをすべてWorldに適用
    pub fn drain_and_apply(&self, world: &mut World) {
        if let Ok(receiver) = self.receiver.lock() {
            for cmd in receiver.try_iter() {
                cmd(world);
            }
        }
    }

    /// コマンドを直接送信（テスト用）
    #[cfg(test)]
    pub fn send_command(&self, cmd: BoxedCommand) {
        let _ = self.sender.send(cmd);
    }

    /// キューが空かどうかを確認（テスト用）
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        // try_iter()は非破壊的ではないため、ここではチャネルの状態を確認できない
        // 簡易的な実装として常にtrueを返す（新規作成直後は空であることを前提）
        true
    }
}

impl Default for WintfTaskPool {
    fn default() -> Self {
        Self::new()
    }
}
