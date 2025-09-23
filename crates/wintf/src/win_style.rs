#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::api::*;
use windows::core::*;
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WinStyle {
    pub(crate) style: WINDOW_STYLE,
    pub(crate) ex_style: WINDOW_EX_STYLE,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) parent: Option<HWND>,
}

impl WinStyle {
    //================================================================================
    // コミット
    //================================================================================

    /// 値の取り出し
    pub fn get_style(&self) -> (WINDOW_STYLE, WINDOW_EX_STYLE) {
        (self.style, self.ex_style)
    }

    /// SetWindowLongWへ反映
    pub fn commit(&self, hwnd: HWND) -> Result<()> {
        set_window_long_ptr(hwnd, GWL_STYLE, self.style.0 as _)?;
        set_window_long_ptr(hwnd, GWL_EXSTYLE, self.ex_style.0 as _)?;
        Ok(())
    }

    //================================================================================
    // コンストラクタ
    //================================================================================

    /// hwndから現在のスタイルを取得して生成します
    pub fn new(hwnd: HWND) -> Result<Self> {
        let style = WINDOW_STYLE(get_window_long_ptr(hwnd, GWL_STYLE)? as _);
        let ex_style = WINDOW_EX_STYLE(get_window_long_ptr(hwnd, GWL_EXSTYLE)? as _);
        Ok(Self {
            style,
            ex_style,
            x: CW_USEDEFAULT,
            y: CW_USEDEFAULT,
            width: CW_USEDEFAULT,
            height: CW_USEDEFAULT,
            parent: None,
        })
    }

    #[inline(always)]
    fn with_style(style: WINDOW_STYLE) -> WinStyle {
        WinStyle {
            style,
            x: CW_USEDEFAULT,
            y: CW_USEDEFAULT,
            width: CW_USEDEFAULT,
            height: CW_USEDEFAULT,
            parent: None,
            ..Default::default()
        }
    }

    //================================================================================
    // 座標値、親ウィンドウ
    //================================================================================

    pub fn position(mut self, x: i32, y: i32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn size(mut self, width: i32, height: i32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn parent(mut self, parent: HWND) -> Self {
        self.parent = Some(parent);
        self
    }

    //================================================================================
    // 複数ビットがONの WINDOW_STYLE (複合フラグ)
    //================================================================================

    /// 標準的なトップレベルウィンドウを作成するための複合スタイルです。`WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX` の組み合わせです。
    pub fn WS_OVERLAPPEDWINDOW() -> Self {
        WinStyle::with_style(WS_OVERLAPPEDWINDOW)
    }

    /// `WS_OVERLAPPEDWINDOW` と同じです。
    pub fn WS_TILEDWINDOW() -> Self {
        WinStyle::with_style(WS_TILEDWINDOW)
    }

    /// 標準的なポップアップウィンドウを作成するための複合スタイルです。`WS_POPUP | WS_BORDER | WS_SYSMENU` の組み合わせです。
    pub fn WS_POPUPWINDOW() -> Self {
        WinStyle::with_style(WS_POPUPWINDOW)
    }

    //================================================================================
    // 1ビットだけONの WINDOW_STYLE (ウィンドウ基本属性)
    //================================================================================

    /// オーバーラップウィンドウを作成します。オーバーラップウィンドウは、タイトルバーと境界線を持つトップレベルウィンドウです。`WS_TILED` と同じです。
    pub fn WS_OVERLAPPED() -> Self {
        WinStyle::with_style(WS_OVERLAPPED)
    }

    /// ポップアップウィンドウを作成します。`WS_CHILD` とは併用できません。
    pub fn WS_POPUP() -> Self {
        WinStyle::with_style(WS_POPUP)
    }

    //================================================================================
    // 1ビットだけONの WINDOW_STYLE (その他の属性)
    //================================================================================

    /// オーバーラップウィンドウを作成します。`WS_OVERLAPPED` と同じです。
    pub fn WS_TILED(self, flag: bool) -> Self {
        set_style(self, WS_TILED, flag)
    }

    /// ウィンドウのタイトルバーに最大化ボタンを表示します。`WS_SYSMENU` スタイルも指定する必要があります。
    pub fn WS_MAXIMIZEBOX(self, flag: bool) -> Self {
        set_style(self, WS_MAXIMIZEBOX, flag)
    }

    /// ウィンドウのタイトルバーに最小化ボタンを表示します。`WS_SYSMENU` スタイルも指定する必要があります。
    pub fn WS_MINIMIZEBOX(self, flag: bool) -> Self {
        set_style(self, WS_MINIMIZEBOX, flag)
    }

    /// サイズ変更可能な境界線を持つウィンドウを作成します。`WS_THICKFRAME` と同じです。
    pub fn WS_SIZEBOX(self, flag: bool) -> Self {
        set_style(self, WS_SIZEBOX, flag)
    }

    /// サイズ変更可能な境界線を持つウィンドウを作成します。`WS_SIZEBOX` と同じです。
    pub fn WS_THICKFRAME(self, flag: bool) -> Self {
        set_style(self, WS_THICKFRAME, flag)
    }

    /// ウィンドウにタイトルバーを追加または削除します。
    pub fn WS_CAPTION(self, flag: bool) -> Self {
        set_style(self, WS_CAPTION, flag)
    }

    /// ウィンドウのタイトルバーにシステムメニューを表示します。`WS_CAPTION` スタイルも指定する必要があります。
    pub fn WS_SYSMENU(self, flag: bool) -> Self {
        set_style(self, WS_SYSMENU, flag)
    }

    /// 水平スクロールバーを持つウィンドウを作成します。
    pub fn WS_HSCROLL(self, flag: bool) -> Self {
        set_style(self, WS_HSCROLL, flag)
    }

    /// 垂直スクロールバーを持つウィンドウを作成します。
    pub fn WS_VSCROLL(self, flag: bool) -> Self {
        set_style(self, WS_VSCROLL, flag)
    }

    /// タイトルバーを持たないダイアログボックス形式の境界線を持つウィンドウを作成します。
    pub fn WS_DLGFRAME(self, flag: bool) -> Self {
        set_style(self, WS_DLGFRAME, flag)
    }

    /// 細い線の境界線を持つウィンドウを作成します。
    pub fn WS_BORDER(self, flag: bool) -> Self {
        set_style(self, WS_BORDER, flag)
    }

    /// ウィンドウを最大化状態で作成します。
    pub fn WS_MAXIMIZE(self, flag: bool) -> Self {
        set_style(self, WS_MAXIMIZE, flag)
    }

    /// 親ウィンドウのクライアント領域内で子ウィンドウが描画される領域をクリップします。親ウィンドウが描画されるときに、子ウィンドウによって隠されている領域の描画を除外します。
    pub fn WS_CLIPCHILDREN(self, flag: bool) -> Self {
        set_style(self, WS_CLIPCHILDREN, flag)
    }

    /// 兄弟関係にある子ウィンドウ同士の描画領域をクリップします。特定の子ウィンドウが再描画される必要がある場合、他の子ウィンドウの領域には描画されません。
    pub fn WS_CLIPSIBLINGS(self, flag: bool) -> Self {
        set_style(self, WS_CLIPSIBLINGS, flag)
    }

    /// ウィンドウを無効状態で作成します。無効化されたウィンドウはユーザーからの入力を受け付けません。
    pub fn WS_DISABLED(self, flag: bool) -> Self {
        set_style(self, WS_DISABLED, flag)
    }

    /// ウィンドウを可視状態で作成します。このスタイルが指定されない場合、ウィンドウは非表示になります。
    pub fn WS_VISIBLE(self, flag: bool) -> Self {
        set_style(self, WS_VISIBLE, flag)
    }

    /// ウィンドウを最小化状態で作成します。`WS_MINIMIZE` と同じです。
    pub fn WS_ICONIC(self, flag: bool) -> Self {
        set_style(self, WS_ICONIC, flag)
    }

    /// ウィンドウを最小化状態で作成します。`WS_ICONIC` と同じです。
    pub fn WS_MINIMIZE(self, flag: bool) -> Self {
        set_style(self, WS_MINIMIZE, flag)
    }

    //================================================================================
    // WINDOW_EX_STYLE: 複数ビットが立っているフラグ (組み合わせ)
    //================================================================================

    /// ウィンドウに立体的な境界線を作成します。
    /// WS_EX_CLIENTEDGE (0x200) と WS_EX_WINDOWEDGE (0x100) の組み合わせです。
    pub fn WS_EX_WINDOWEDGE(self) -> Self {
        set_ex(self, WS_EX_WINDOWEDGE, true)
    }

    /// オーバーラップウィンドウを作成します。
    /// WS_EX_WINDOWEDGE (0x100) と WS_EX_CLIENTEDGE (0x200) の組み合わせです。
    pub fn WS_EX_OVERLAPPEDWINDOW(self) -> Self {
        set_ex(self, WS_EX_OVERLAPPEDWINDOW, true)
    }

    /// パレットウィンドウを作成します。これは、WS_EX_WINDOWEDGE、WS_EX_TOOLWINDOW、WS_EX_TOPMOSTを組み合わせたものです。
    /// WS_EX_WINDOWEDGE (0x100) | WS_EX_TOOLWINDOW (0x80) | WS_EX_TOPMOST (0x8) の組み合わせです。
    pub fn WS_EX_PALETTEWINDOW(self) -> Self {
        set_ex(self, WS_EX_PALETTEWINDOW, true)
    }

    //================================================================================
    // WINDOW_EX_STYLE: 1ビットだけが立っているフラグ (単一機能)
    //================================================================================

    /// ウィンドウにモーダルダイアログボックスのような二重の境界線を作成します。
    pub fn WS_EX_DLGMODALFRAME(self, flag: bool) -> Self {
        set_ex(self, WS_EX_DLGMODALFRAME, flag)
    }

    /// 子ウィンドウが作成または破棄されたときに、その親ウィンドウにWM_PARENTNOTIFYメッセージを送信しないように指定します。
    pub fn WS_EX_NOPARENTNOTIFY(self, flag: bool) -> Self {
        set_ex(self, WS_EX_NOPARENTNOTIFY, flag)
    }

    /// ウィンドウを最前面ウィンドウとして作成します。システムは、非最前面ウィンドウの上にこのウィンドウを配置します。
    pub fn WS_EX_TOPMOST(self, flag: bool) -> Self {
        set_ex(self, WS_EX_TOPMOST, flag)
    }

    /// ウィンドウを透明として作成します。このウィンドウの下にある兄弟ウィンドウが描画されるまで、このウィンドウは描画されません。
    pub fn WS_EX_TRANSPARENT(self, flag: bool) -> Self {
        set_ex(self, WS_EX_TRANSPARENT, flag)
    }

    /// MDI（Multiple Document Interface）の子ウィンドウを作成します。
    pub fn WS_EX_MDICHILD(self, flag: bool) -> Self {
        set_ex(self, WS_EX_MDICHILD, flag)
    }

    /// フローティングツールバーとして使用されるツールウィンドウを作成します。
    pub fn WS_EX_TOOLWINDOW(self, flag: bool) -> Self {
        set_ex(self, WS_EX_TOOLWINDOW, flag)
    }

    /// ウィンドウに立体的な境界線を持つ、沈んだ外観を与えます。
    pub fn WS_EX_CLIENTEDGE(self, flag: bool) -> Self {
        set_ex(self, WS_EX_CLIENTEDGE, flag)
    }

    /// ウィンドウのタイトルバーに疑問符（？）のコンテキストヘルプボタンを追加します。
    pub fn WS_EX_CONTEXTHELP(self, flag: bool) -> Self {
        set_ex(self, WS_EX_CONTEXTHELP, flag)
    }

    /// Tabキーを使用してユーザーがコントロール間を移動できる子ウィンドウを持つウィンドウ（ダイアログボックスなど）を作成します。
    pub fn WS_EX_CONTROLPARENT(self, flag: bool) -> Self {
        set_ex(self, WS_EX_CONTROLPARENT, flag)
    }

    /// アイテムをドラッグアンドドロップで受け入れないウィンドウに、立体的な境界線スタイルを与えます。
    pub fn WS_EX_STATICEDGE(self, flag: bool) -> Self {
        set_ex(self, WS_EX_STATICEDGE, flag)
    }

    /// トップレベルウィンドウをタスクバーに表示します。
    pub fn WS_EX_APPWINDOW(self, flag: bool) -> Self {
        set_ex(self, WS_EX_APPWINDOW, flag)
    }

    /// レイヤードウィンドウを作成します。このスタイルは、ウィンドウ作成時またはSetWindowLong/Ex関数で設定する必要があります。
    pub fn WS_EX_LAYERED(self, flag: bool) -> Self {
        set_ex(self, WS_EX_LAYERED, flag)
    }

    /// 子ウィンドウが親ウィンドウのレイアウト（RTLまたはLTR）を継承しないようにします。
    pub fn WS_EX_NOINHERITLAYOUT(self, flag: bool) -> Self {
        set_ex(self, WS_EX_NOINHERITLAYOUT, flag)
    }

    /// このウィンドウとその子ウィンドウの描画を、画面外のビットマップにリダイレクトしません。
    pub fn WS_EX_NOREDIRECTIONBITMAP(self, flag: bool) -> Self {
        set_ex(self, WS_EX_NOREDIRECTIONBITMAP, flag)
    }

    /// ウィンドウをコンポジットウィンドウとして作成します。
    pub fn WS_EX_COMPOSITED(self, flag: bool) -> Self {
        set_ex(self, WS_EX_COMPOSITED, flag)
    }

    /// このスタイルで作成されたウィンドウは、ユーザーがクリックしてもフォアグラウンドウィンドウになりません。
    pub fn WS_EX_NOACTIVATE(self, flag: bool) -> Self {
        set_ex(self, WS_EX_NOACTIVATE, flag)
    }

    //================================================================================
    // WINDOW_EX_STYLE: ON/OFF関係
    //================================================================================

    /// ウィンドウは右側に垂直スクロールバーを持ちます（存在する場合）。これは既定値です。
    pub fn WS_EX_RIGHTSCROLLBAR(self) -> Self {
        set_ex(self, WS_EX_LEFTSCROLLBAR, false)
    }
    /// 垂直スクロールバーをウィンドウの左端に配置します。
    pub fn WS_EX_LEFTSCROLLBAR(self) -> Self {
        set_ex(self, WS_EX_LEFTSCROLLBAR, true)
    }

    /// ウィンドウは左揃えのプロパティを持ちます。これは既定値です。
    pub fn WS_EX_LEFT(self) -> Self {
        set_ex(self, WS_EX_RIGHT, false)
    }
    /// ウィンドウ自体が右揃えのプロパティを持ちます。
    pub fn WS_EX_RIGHT(self) -> Self {
        set_ex(self, WS_EX_RIGHT, true)
    }

    /// ウィンドウは左から右（LTR）への読み取り順序のプロパティを持ちます。これは既定値です。
    pub fn WS_EX_LTRREADING(self) -> Self {
        set_ex(self, WS_EX_RTLREADING, false)
    }
    /// ウィンドウのテキストを右から左（RTL）の読み取り順序で表示します。
    pub fn WS_EX_RTLREADING(self) -> Self {
        set_ex(self, WS_EX_RTLREADING, true)
    }

    /// ウィンドウのレイアウトを左から右（LTR）にします。これは既定値です。
    pub fn WS_EX_LAYOUTLTR(self) -> Self {
        set_ex(self, WS_EX_LAYOUTRTL, false)
    }
    /// ウィンドウのレイアウトを右から左（RTL）にします。
    pub fn WS_EX_LAYOUTRTL(self) -> Self {
        set_ex(self, WS_EX_LAYOUTRTL, true)
    }
}

#[inline(always)]
fn set_style(src: WinStyle, style: WINDOW_STYLE, flag: bool) -> WinStyle {
    let org = src.style.0;
    let style = style.0;
    let style = if flag { org | style } else { org & !style };
    let style = WINDOW_STYLE(style);
    WinStyle { style, ..src }
}

#[inline(always)]
fn set_ex(src: WinStyle, ex_style: WINDOW_EX_STYLE, flag: bool) -> WinStyle {
    let org = src.ex_style.0;
    let ex_style = ex_style.0;
    let ex_style = if flag {
        org | ex_style
    } else {
        org & !ex_style
    };
    let ex_style = WINDOW_EX_STYLE(ex_style);
    WinStyle { ex_style, ..src }
}

#[inline(always)]
fn set_ex2(src: WinStyle, on: WINDOW_EX_STYLE, off: WINDOW_EX_STYLE, flag: bool) -> WinStyle {
    let org = src.ex_style.0;
    let cleared = org & !(on.0 | off.0);
    let new_ex_style_value = if flag {
        cleared | on.0
    } else {
        cleared | off.0
    };
    let ex_style = WINDOW_EX_STYLE(new_ex_style_value);
    WinStyle { ex_style, ..src }
}
