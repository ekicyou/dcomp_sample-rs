use bevy_ecs::component::Component;
use windows::Win32::Graphics::Direct2D::ID2D1CommandList;

/// Direct2D描画命令リスト
#[derive(Component, Debug, Clone, PartialEq)]
pub struct GraphicsCommandList {
    command_list: Option<ID2D1CommandList>,
}

impl GraphicsCommandList {
    /// 新しいCommandListコンポーネントを作成
    pub fn new(command_list: ID2D1CommandList) -> Self {
        Self {
            command_list: Some(command_list),
        }
    }

    /// 空のCommandListコンポーネントを作成
    pub fn empty() -> Self {
        Self { command_list: None }
    }

    /// CommandListへの参照を取得
    pub fn command_list(&self) -> Option<&ID2D1CommandList> {
        self.command_list.as_ref()
    }
}

// スレッド間送信を可能にする（windows-rsのスマートポインタはSend+Sync）
unsafe impl Send for GraphicsCommandList {}
unsafe impl Sync for GraphicsCommandList {}
