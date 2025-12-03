//! 高レベルレイアウトAPI
//!
//! taffyの型を直接使わず、wintf独自の型を提供する。
//! これらの型はCopy, Send, Syncを満たし、bevy_ecsのComponentとして使用可能。

use bevy_ecs::prelude::*;

// ===== Dimension型: Auto, Length, Percentをサポート =====

/// レイアウト寸法を表す型（Auto, ピクセル、パーセント）
///
/// # パーセント値の扱い
///
/// パーセント値は**0.0～100.0**の範囲で指定します。
/// これはCSS、HTML、WPF/XAMLなどの一般的なUI記法に合わせた直感的な表記です。
///
/// # 例
///
/// ```ignore
/// use wintf::ecs::layout::Dimension;
///
/// // 100%（親要素全体）
/// let full = Dimension::Percent(100.0);
///
/// // 50%（親要素の半分）
/// let half = Dimension::Percent(50.0);
///
/// // 固定200ピクセル
/// let fixed = Dimension::Px(200.0);
///
/// // 自動サイズ
/// let auto = Dimension::Auto;
/// ```
///
/// # 内部実装の注意
///
/// 内部的にTaffyレイアウトエンジンは0.0-1.0の範囲（正規化値）を使用しますが、
/// wintfのAPIでは0.0-100.0を採用しています。変換は自動的に行われます。
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum Dimension {
    /// 自動サイズ
    Auto,
    /// ピクセル値
    Px(f32),
    /// パーセント値（0.0～100.0）
    ///
    /// 例: `Percent(100.0)` = 100%、`Percent(50.0)` = 50%
    Percent(f32),
}

impl Dimension {
    /// ピクセル値を生成
    pub const fn length(val: f32) -> Self {
        Self::Px(val)
    }

    /// パーセント値を生成
    pub const fn percent(val: f32) -> Self {
        Self::Percent(val)
    }

    /// Auto値を生成
    pub const fn auto() -> Self {
        Self::Auto
    }

    /// ゼロ値を生成
    pub const fn zero() -> Self {
        Self::Px(0.0)
    }
}

/// TaffyZeroトレイト: ZERO定数を提供
pub trait TaffyZero {
    const ZERO: Self;
}

impl TaffyZero for Dimension {
    const ZERO: Self = Self::Px(0.0);
}

/// TaffyAutoトレイト: AUTO定数を提供
pub trait TaffyAuto {
    const AUTO: Self;
}

impl TaffyAuto for Dimension {
    const AUTO: Self = Self::Auto;
}

impl Default for Dimension {
    fn default() -> Self {
        Self::Auto
    }
}

// taffy::Dimensionへの変換
impl From<Dimension> for taffy::Dimension {
    fn from(val: Dimension) -> Self {
        match val {
            Dimension::Auto => taffy::Dimension::auto(),
            Dimension::Px(v) => taffy::Dimension::length(v),
            // Taffyはパーセントを0.0-1.0の範囲で扱うため、100で割る
            Dimension::Percent(v) => taffy::Dimension::percent(v / 100.0),
        }
    }
}

// taffy::Dimensionからの変換
impl From<taffy::Dimension> for Dimension {
    fn from(_val: taffy::Dimension) -> Self {
        // taffy::DimensionはCompactLengthを使用しているため、
        // 内部表現から値を取り出す必要がある
        // ここでは簡易的にデフォルト値を使用
        // TODO: 正確な変換が必要な場合は実装を追加
        Self::Auto
    }
}

// ===== LengthPercentageAuto型: Auto, Length, Percentをサポート =====

/// 長さ/パーセント/自動を表す型（マージンなどで使用）
///
/// マージンなどで使用される寸法型。パーセント値は**0.0～100.0**の範囲で指定します。
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum LengthPercentageAuto {
    /// 自動値
    Auto,
    /// ピクセル値
    Px(f32),
    /// パーセント値（0.0～100.0）
    Percent(f32),
}

impl LengthPercentageAuto {
    pub const fn length(val: f32) -> Self {
        Self::Px(val)
    }

    pub const fn percent(val: f32) -> Self {
        Self::Percent(val)
    }

    pub const fn auto() -> Self {
        Self::Auto
    }

    pub const fn zero() -> Self {
        Self::Px(0.0)
    }
}

impl TaffyZero for LengthPercentageAuto {
    const ZERO: Self = Self::Px(0.0);
}

impl TaffyAuto for LengthPercentageAuto {
    const AUTO: Self = Self::Auto;
}

impl Default for LengthPercentageAuto {
    fn default() -> Self {
        Self::Auto
    }
}

impl From<LengthPercentageAuto> for taffy::LengthPercentageAuto {
    fn from(val: LengthPercentageAuto) -> Self {
        match val {
            LengthPercentageAuto::Auto => taffy::LengthPercentageAuto::auto(),
            LengthPercentageAuto::Px(v) => taffy::LengthPercentageAuto::length(v),
            LengthPercentageAuto::Percent(v) => taffy::LengthPercentageAuto::percent(v),
        }
    }
}

// ===== LengthPercentage型: Length, Percentをサポート =====

/// 長さ/パーセントを表す型（パディングなどで使用、Autoなし）
///
/// パディングなどで使用される寸法型。パーセント値は**0.0～100.0**の範囲で指定します。
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum LengthPercentage {
    /// ピクセル値
    Px(f32),
    /// パーセント値（0.0～100.0）
    Percent(f32),
}

impl LengthPercentage {
    pub const fn length(val: f32) -> Self {
        Self::Px(val)
    }

    pub const fn percent(val: f32) -> Self {
        Self::Percent(val)
    }

    pub const fn zero() -> Self {
        Self::Px(0.0)
    }
}

impl TaffyZero for LengthPercentage {
    const ZERO: Self = Self::Px(0.0);
}

impl Default for LengthPercentage {
    fn default() -> Self {
        Self::Px(0.0)
    }
}

impl From<LengthPercentage> for taffy::LengthPercentage {
    fn from(val: LengthPercentage) -> Self {
        match val {
            LengthPercentage::Px(v) => taffy::LengthPercentage::length(v),
            LengthPercentage::Percent(v) => taffy::LengthPercentage::percent(v),
        }
    }
}

// ===== Rect型: 矩形の4辺を表すジェネリック型 =====

/// 矩形の4辺（left, right, top, bottom）を表すジェネリック型
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct Rect<T> {
    pub left: T,
    pub right: T,
    pub top: T,
    pub bottom: T,
}

impl<T: TaffyZero> Rect<T> {
    pub const fn zero() -> Self
    where
        T: Copy,
    {
        Self {
            left: T::ZERO,
            right: T::ZERO,
            top: T::ZERO,
            bottom: T::ZERO,
        }
    }
}

impl<T: TaffyAuto> Rect<T> {
    pub const fn auto() -> Self
    where
        T: Copy,
    {
        Self {
            left: T::AUTO,
            right: T::AUTO,
            top: T::AUTO,
            bottom: T::AUTO,
        }
    }
}

impl<T: Default> Default for Rect<T> {
    fn default() -> Self {
        Self {
            left: T::default(),
            right: T::default(),
            top: T::default(),
            bottom: T::default(),
        }
    }
}

impl From<Rect<LengthPercentageAuto>> for taffy::Rect<taffy::LengthPercentageAuto> {
    fn from(val: Rect<LengthPercentageAuto>) -> Self {
        taffy::Rect {
            left: val.left.into(),
            right: val.right.into(),
            top: val.top.into(),
            bottom: val.bottom.into(),
        }
    }
}

impl From<Rect<LengthPercentage>> for taffy::Rect<taffy::LengthPercentage> {
    fn from(val: Rect<LengthPercentage>) -> Self {
        taffy::Rect {
            left: val.left.into(),
            right: val.right.into(),
            top: val.top.into(),
            bottom: val.bottom.into(),
        }
    }
}

// ===== 高レベルレイアウトコンポーネント（値オブジェクト） =====
// 注: これらの型は以前はComponentでしたが、BoxStyle統合後は値オブジェクトとして使用します。
// Componentとしての利用は廃止され、BoxStyleを使用してください。

/// ボックスサイズ（値オブジェクト）
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BoxSize {
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
}

/// ボックスマージン（値オブジェクト）
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BoxMargin(pub Rect<LengthPercentageAuto>);

/// ボックスパディング（値オブジェクト）
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BoxPadding(pub Rect<LengthPercentage>);

/// ボックス配置タイプ（値オブジェクト）
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BoxPosition {
    /// 相対配置（通常のフロー内配置）
    #[default]
    Relative,
    /// 絶対配置（親要素基準の座標指定）
    Absolute,
}

/// 絶対配置のインセット座標（値オブジェクト）
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BoxInset(pub Rect<LengthPercentageAuto>);

/// Flexコンテナ（値オブジェクト）
///
/// 注: BoxStyleのflex_direction, justify_content, align_itemsを直接使用することを推奨。
/// この型は後方互換性のために維持されています。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlexContainer {
    pub direction: taffy::FlexDirection,
    pub justify_content: Option<taffy::JustifyContent>,
    pub align_items: Option<taffy::AlignItems>,
}

impl Default for FlexContainer {
    fn default() -> Self {
        Self {
            direction: taffy::FlexDirection::Row,
            justify_content: None,
            align_items: None,
        }
    }
}

/// Flexアイテム（値オブジェクト）
///
/// 注: BoxStyleのflex_grow, flex_shrink, flex_basis, align_selfを直接使用することを推奨。
/// この型は後方互換性のために維持されています。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlexItem {
    pub grow: f32,
    pub shrink: f32,
    pub basis: Dimension,
    pub align_self: Option<taffy::AlignSelf>,
}

impl Default for FlexItem {
    fn default() -> Self {
        Self {
            grow: 0.0,
            shrink: 1.0,
            basis: Dimension::Auto,
            align_self: None,
        }
    }
}

// taffy の Flex 関連型を re-export（テストと外部利用のため）
pub use taffy::{AlignContent, AlignItems, AlignSelf, FlexDirection, JustifyContent};

// ===== BoxStyle統合コンポーネント =====

/// 統合レイアウトスタイルコンポーネント
///
/// 全レイアウトプロパティを統合したユーザー向けコンポーネント。
/// `TaffyStyle`と1:1対応し、`build_taffy_styles_system`で変換される。
///
/// # 設計意図
///
/// - Box系5種（size, margin, padding, position, inset）をOption型でネスト構造として含める
/// - Flex系7種（flex_direction, justify_content, align_items, flex_grow, flex_shrink, flex_basis, align_self）
///   をフラットなOption型フィールドとして含める（taffyのStyle構造体と同様のフラット設計）
/// - `None`フィールドはtaffyデフォルト値にマッピング
///
/// # 使用例
///
/// ```rust,ignore
/// use wintf::ecs::layout::*;
///
/// // Flexコンテナーとして使用
/// commands.spawn((
///     BoxStyle {
///         size: Some(BoxSize {
///             width: Some(Dimension::Percent(100.0)),
///             height: Some(Dimension::Percent(100.0)),
///         }),
///         flex_direction: Some(FlexDirection::Row),
///         justify_content: Some(JustifyContent::SpaceEvenly),
///         align_items: Some(AlignItems::Center),
///         ..Default::default()
///     },
/// ));
///
/// // Flexアイテムとして使用
/// commands.spawn((
///     BoxStyle {
///         size: Some(BoxSize {
///             width: Some(Dimension::Px(200.0)),
///             height: Some(Dimension::Px(100.0)),
///         }),
///         flex_grow: Some(1.0),
///         flex_shrink: Some(1.0),
///         ..Default::default()
///     },
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct BoxStyle {
    // === Box系プロパティ（ネスト構造） ===
    /// サイズ（width, height）
    pub size: Option<BoxSize>,
    /// 最小サイズ（min_width, min_height）
    pub min_size: Option<BoxSize>,
    /// 最大サイズ（max_width, max_height）
    pub max_size: Option<BoxSize>,
    /// マージン（外側余白）
    pub margin: Option<BoxMargin>,
    /// パディング（内側余白）
    pub padding: Option<BoxPadding>,
    /// 配置タイプ（Relative/Absolute）
    pub position: Option<BoxPosition>,
    /// インセット（絶対配置時の座標）
    pub inset: Option<BoxInset>,

    // === Flex系プロパティ（フラット構造） ===
    /// Flexコンテナーの主軸方向
    pub flex_direction: Option<FlexDirection>,
    /// 主軸方向の子要素配置
    pub justify_content: Option<JustifyContent>,
    /// 交差軸方向の子要素配置
    pub align_items: Option<AlignItems>,
    /// Flexアイテムの伸長率（デフォルト: 0.0）
    /// 注: Noneの場合はtaffyデフォルト値(0.0)を適用
    pub flex_grow: Option<f32>,
    /// Flexアイテムの収縮率（デフォルト: 1.0）
    /// 注: Noneの場合はtaffyデフォルト値(1.0)を適用
    pub flex_shrink: Option<f32>,
    /// Flexアイテムの基準サイズ
    pub flex_basis: Option<Dimension>,
    /// 自身の交差軸配置（親のalign_itemsを上書き）
    pub align_self: Option<AlignSelf>,
}

impl BoxStyle {
    /// 新しいBoxStyleを作成
    pub fn new() -> Self {
        Self::default()
    }
}

/// BoxStyleからtaffy::Styleへの変換
impl From<&BoxStyle> for taffy::Style {
    fn from(style: &BoxStyle) -> Self {
        let mut taffy_style = taffy::Style::default();

        // Box系プロパティ変換
        if let Some(size) = &style.size {
            if let Some(w) = size.width {
                taffy_style.size.width = w.into();
            }
            if let Some(h) = size.height {
                taffy_style.size.height = h.into();
            }
        }
        if let Some(min_size) = &style.min_size {
            if let Some(w) = min_size.width {
                taffy_style.min_size.width = w.into();
            }
            if let Some(h) = min_size.height {
                taffy_style.min_size.height = h.into();
            }
        }
        if let Some(max_size) = &style.max_size {
            if let Some(w) = max_size.width {
                taffy_style.max_size.width = w.into();
            }
            if let Some(h) = max_size.height {
                taffy_style.max_size.height = h.into();
            }
        }
        if let Some(margin) = &style.margin {
            taffy_style.margin = margin.0.into();
        }
        if let Some(padding) = &style.padding {
            taffy_style.padding = padding.0.into();
        }
        if let Some(position) = &style.position {
            taffy_style.position = match position {
                BoxPosition::Relative => taffy::Position::Relative,
                BoxPosition::Absolute => taffy::Position::Absolute,
            };
        }
        if let Some(inset) = &style.inset {
            taffy_style.inset = inset.0.into();
        }

        // Flex系プロパティ変換
        // コンテナープロパティ設定時にdisplay: Flexを自動設定
        if style.flex_direction.is_some()
            || style.justify_content.is_some()
            || style.align_items.is_some()
        {
            taffy_style.display = taffy::Display::Flex;
        }
        if let Some(dir) = style.flex_direction {
            taffy_style.flex_direction = dir;
        }
        if let Some(jc) = style.justify_content {
            taffy_style.justify_content = Some(jc);
        }
        if let Some(ai) = style.align_items {
            taffy_style.align_items = Some(ai);
        }

        // flex_grow/flex_shrinkはNone時にtaffyデフォルト値を適用
        taffy_style.flex_grow = style.flex_grow.unwrap_or(0.0);
        taffy_style.flex_shrink = style.flex_shrink.unwrap_or(1.0);

        if let Some(basis) = style.flex_basis {
            taffy_style.flex_basis = basis.into();
        }
        if let Some(align_self) = style.align_self {
            taffy_style.align_self = Some(align_self);
        }

        taffy_style
    }
}
