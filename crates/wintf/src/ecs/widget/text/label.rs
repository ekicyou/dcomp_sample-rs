use crate::ecs::graphics::GraphicsCommandList;
use bevy_ecs::change_detection::DetectChangesMut;
use bevy_ecs::component::Component;
use bevy_ecs::world::DeferredWorld;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use windows::Win32::Graphics::DirectWrite::IDWriteTextLayout;

/// 色の型エイリアス（shapes/rectangleと共通）
pub type Color = D2D1_COLOR_F;

/// Labelコンポーネント: テキスト表示ウィジット
///
/// # フィールド
/// - `text`: 表示するテキスト (UTF-8)
/// - `font_family`: フォントファミリー名 (例: "メイリオ", "Arial")
/// - `font_size`: フォントサイズ (pt単位, 範囲: 8.0～72.0)
/// - `color`: テキスト色 (RGBA, 各成分 0.0～1.0)
#[derive(Component)]
#[component(storage = "SparseSet", on_remove = on_label_remove)]
pub struct Label {
    pub text: String,
    pub font_family: String,
    pub font_size: f32,
    pub color: Color,
}

impl Default for Label {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_family: "メイリオ".to_string(),
            font_size: 16.0,
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }, // 黒色
        }
    }
}

/// Labelコンポーネント削除時のフック
/// GraphicsCommandListをクリアしてChanged検出に対応
fn on_label_remove(mut world: DeferredWorld, hook: bevy_ecs::lifecycle::HookContext) {
    let entity = hook.entity;
    // GraphicsCommandListを取得して中身をクリア(Changed検出のため)
    if let Some(mut cmd_list) = world.get_mut::<GraphicsCommandList>(entity) {
        cmd_list.set_if_neq(GraphicsCommandList::empty());
    }
}

/// TextLayoutコンポーネント: IDWriteTextLayoutのキャッシュ
///
/// Labelコンポーネントから生成されたTextLayoutを保持。
/// Labelが変更されない限り、再生成せず再利用される。
#[derive(Component)]
#[component(storage = "SparseSet", on_remove = on_text_layout_remove)]
pub struct TextLayoutResource {
    layout: Option<IDWriteTextLayout>,
}

impl TextLayoutResource {
    /// 新しいTextLayoutコンポーネントを作成
    pub fn new(layout: IDWriteTextLayout) -> Self {
        Self {
            layout: Some(layout),
        }
    }

    /// TextLayoutを取得
    pub fn get(&self) -> Option<&IDWriteTextLayout> {
        self.layout.as_ref()
    }

    /// 空のTextLayoutを作成
    pub fn empty() -> Self {
        Self { layout: None }
    }
}

/// TextLayoutコンポーネント削除時のフック
/// COMオブジェクトはDropで自動解放されるため、ログ出力のみ
fn on_text_layout_remove(_world: DeferredWorld, hook: bevy_ecs::lifecycle::HookContext) {
    #[cfg(debug_assertions)]
    println!("[TextLayoutResource] Removed from Entity={:?}", hook.entity);
}
