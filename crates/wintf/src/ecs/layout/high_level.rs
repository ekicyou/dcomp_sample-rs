//! 高レベルレイアウトAPI
//!
//! taffyの型を直接使わず、wintf独自の型を提供する。
//! これらの型はCopy, Send, Syncを満たし、bevy_ecsのComponentとして使用可能。

use bevy_ecs::prelude::*;

// ===== Dimension型: Auto, Length, Percentをサポート =====

/// レイアウト寸法を表す型（Auto, ピクセル、パーセント）
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum Dimension {
    /// 自動サイズ
    Auto,
    /// ピクセル値
    Px(f32),
    /// パーセント値（0.0～100.0）
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
            Dimension::Percent(v) => taffy::Dimension::percent(v),
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
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum LengthPercentageAuto {
    /// 自動値
    Auto,
    /// ピクセル値
    Px(f32),
    /// パーセント値
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

// ===== LengthPercentage型: Length, Percentのみ（Autoなし） =====

/// 長さ/パーセントを表す型（パディングなどで使用、Autoなし）
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum LengthPercentage {
    /// ピクセル値
    Px(f32),
    /// パーセント値
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

// ===== 高レベルレイアウトコンポーネント =====

/// ボックスサイズコンポーネント
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct BoxSize {
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
}

impl Default for BoxSize {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
        }
    }
}

/// ボックスマージンコンポーネント
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct BoxMargin(pub Rect<LengthPercentageAuto>);

impl Default for BoxMargin {
    fn default() -> Self {
        Self(Rect::zero())
    }
}

/// ボックスパディングコンポーネント
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct BoxPadding(pub Rect<LengthPercentage>);

impl Default for BoxPadding {
    fn default() -> Self {
        Self(Rect::zero())
    }
}

/// Flexコンテナコンポーネント
#[derive(Debug, Clone, Copy, PartialEq, Component)]
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

/// Flexアイテムコンポーネント
#[derive(Debug, Clone, Copy, PartialEq, Component)]
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
