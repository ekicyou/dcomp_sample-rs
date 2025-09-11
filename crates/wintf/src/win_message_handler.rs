#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::win_state::*;
use ambassador::*;
use windows::Win32::{
    Foundation::*,
    Graphics::Dwm::*,
    UI::{Controls::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

#[delegatable_trait]
pub trait BaseWinMessageHandler {
    fn message_handler(&mut self, hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
}

#[delegatable_trait]
pub trait WinNcCreate {
    fn WM_NCCREATE(&mut self, hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT>;
}

#[delegatable_trait]
pub trait WinMessageHandler: BaseWinMessageHandler + WinNcCreate {
    /// 生のメッセージハンドラ
    #[inline(always)]
    fn raw_message_handler(
        &mut self,
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT> {
        None
    }

    // デフォルト実装（改定箇所あり）

    #[inline(always)]
    fn WM_ERASEBKGND(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        Some(LRESULT(1)) // DirectCompositionで描画するため、背景消去をスキップ
    }

    #[inline(always)]
    fn WM_MOUSEENTER(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }

    #[inline(always)]
    fn WM_MOUSELEAVE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }

    // デフォルト実装

    #[inline(always)]
    fn WM_ACTIVATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_ACTIVATEAPP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_AFXFIRST(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_AFXLAST(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_APPCOMMAND(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_ASKCBFORMATNAME(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CANCELJOURNAL(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CANCELMODE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CAPTURECHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CHANGECBCHAIN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CHANGEUISTATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CHARTOITEM(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CHILDACTIVATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CLEAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CLIPBOARDUPDATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CLOSE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_COMMAND(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_COMMNOTIFY(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_COMPACTING(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_COMPAREITEM(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CONTEXTMENU(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_COPY(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_COPYDATA(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CREATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CTLCOLORBTN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CTLCOLORDLG(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CTLCOLOREDIT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CTLCOLORLISTBOX(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CTLCOLORMSGBOX(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CTLCOLORSCROLLBAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CTLCOLORSTATIC(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_CUT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DEADCHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DELETEITEM(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DESTROY(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DESTROYCLIPBOARD(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DEVICECHANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DEVMODECHANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DISPLAYCHANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DPICHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DPICHANGED_AFTERPARENT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DPICHANGED_BEFOREPARENT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DRAWCLIPBOARD(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DRAWITEM(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DROPFILES(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DWMCOLORIZATIONCOLORCHANGED(
        &mut self,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DWMCOMPOSITIONCHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DWMNCRENDERINGCHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DWMSENDICONICLIVEPREVIEWBITMAP(
        &mut self,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DWMSENDICONICTHUMBNAIL(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_DWMWINDOWMAXIMIZEDCHANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_ENABLE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_ENDSESSION(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_ENTERIDLE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_ENTERMENULOOP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_ENTERSIZEMOVE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_EXITMENULOOP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_EXITSIZEMOVE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_FONTCHANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_GESTURE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_GESTURENOTIFY(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_GETDLGCODE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_GETDPISCALEDSIZE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_GETFONT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_GETHOTKEY(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_GETICON(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_GETMINMAXINFO(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_GETOBJECT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_GETTEXT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_GETTEXTLENGTH(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_GETTITLEBARINFOEX(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_HANDHELDFIRST(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_HANDHELDLAST(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_HELP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_HOTKEY(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_HSCROLL(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_HSCROLLCLIPBOARD(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_ICONERASEBKGND(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_IME_CHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_IME_COMPOSITION(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_IME_COMPOSITIONFULL(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_IME_CONTROL(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_IME_ENDCOMPOSITION(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_IME_KEYDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_IME_KEYUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_IME_NOTIFY(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_IME_REQUEST(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_IME_SELECT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_IME_SETCONTEXT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_IME_STARTCOMPOSITION(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_INITDIALOG(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_INITMENU(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_INITMENUPOPUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_INPUT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_INPUTLANGCHANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_INPUTLANGCHANGEREQUEST(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_INPUT_DEVICE_CHANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_KEYDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_KEYUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_KILLFOCUS(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_LBUTTONDBLCLK(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_LBUTTONDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_LBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MBUTTONDBLCLK(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MBUTTONDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MDIACTIVATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MDICASCADE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MDICREATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MDIDESTROY(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MDIGETACTIVE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MDIICONARRANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MDIMAXIMIZE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MDINEXT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MDIREFRESHMENU(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MDIRESTORE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MDISETMENU(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MDITILE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MEASUREITEM(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MENUCHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MENUCOMMAND(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MENUDRAG(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MENUGETOBJECT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MENURBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MENUSELECT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MOUSEACTIVATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MOUSEHWHEEL(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MOUSEMOVE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MOUSEWHEEL(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MOVE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_MOVING(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCACTIVATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCCALCSIZE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCDESTROY(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCHITTEST(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCLBUTTONDBLCLK(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCLBUTTONDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCLBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCMBUTTONDBLCLK(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCMBUTTONDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCMBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCMOUSEHOVER(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCMOUSELEAVE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCMOUSEMOVE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCPAINT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCPOINTERDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCPOINTERUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCPOINTERUPDATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCRBUTTONDBLCLK(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCRBUTTONDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCRBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCXBUTTONDBLCLK(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCXBUTTONDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NCXBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NEXTDLGCTL(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NEXTMENU(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NOTIFY(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NOTIFYFORMAT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_NULL(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_PAINT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_PAINTCLIPBOARD(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_PAINTICON(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_PALETTECHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_PALETTEISCHANGING(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_PARENTNOTIFY(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_PASTE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_PENWINFIRST(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_PENWINLAST(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POINTERACTIVATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POINTERCAPTURECHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POINTERDEVICECHANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POINTERDEVICEINRANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POINTERDEVICEOUTOFRANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POINTERDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POINTERENTER(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POINTERHWHEEL(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POINTERLEAVE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POINTERROUTEDAWAY(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POINTERROUTEDRELEASED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POINTERROUTEDTO(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POINTERUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POINTERUPDATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POINTERWHEEL(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POWER(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_POWERBROADCAST(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_PRINT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_PRINTCLIENT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_QUERYDRAGICON(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_QUERYENDSESSION(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_QUERYNEWPALETTE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_QUERYOPEN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_QUERYUISTATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_QUEUESYNC(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_QUIT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_RBUTTONDBLCLK(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_RBUTTONDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_RBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_RENDERALLFORMATS(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_RENDERFORMAT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SETCURSOR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SETFOCUS(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SETFONT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SETHOTKEY(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SETICON(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SETREDRAW(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SETTEXT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SETTINGCHANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SHOWWINDOW(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SIZE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SIZECLIPBOARD(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SIZING(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SPOOLERSTATUS(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_STYLECHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_STYLECHANGING(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SYNCPAINT(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SYSCHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SYSCOLORCHANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SYSCOMMAND(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SYSDEADCHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SYSKEYDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_SYSKEYUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_TABLET_FIRST(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_TABLET_LAST(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_TCARD(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_THEMECHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_TIMECHANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_TIMER(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_TOOLTIPDISMISS(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_TOUCH(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_TOUCHHITTESTING(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_UNDO(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_UNICHAR(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_UNINITMENUPOPUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_UPDATEUISTATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_USERCHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_VKEYTOITEM(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_VSCROLL(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_VSCROLLCLIPBOARD(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_WINDOWPOSCHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_WINDOWPOSCHANGING(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_WTSSESSION_CHANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_XBUTTONDBLCLK(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_XBUTTONDOWN(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
    #[inline(always)]
    fn WM_XBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        None
    }
}

impl<T: WinMessageHandler + WinState> WinNcCreate for T {
    #[inline(always)]
    fn WM_NCCREATE(&mut self, hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        self.set_hwnd(hwnd);
        None
    }
}

impl<T: WinMessageHandler + WinState> BaseWinMessageHandler for T {
    fn message_handler(&mut self, hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if let Some(handled) = self.raw_message_handler(hwnd, msg, wparam, lparam) {
            return handled;
        }

        let handled = match msg {
            // 個別処理イベント
            WM_NCCREATE => self.WM_NCCREATE(hwnd, wparam, lparam),
            WM_MOUSEMOVE => {
                if !self.mouse_tracking() {
                    let mut tt = TRACKMOUSEEVENT {
                        cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                        dwFlags: TME_LEAVE,
                        hwndTrack: self.hwnd(),
                        dwHoverTime: HOVER_DEFAULT,
                    };
                    unsafe {
                        if TrackMouseEvent(&mut tt).is_ok() {
                            self.set_mouse_tracking(true);
                            self.WM_MOUSEENTER(wparam, lparam);
                        }
                    }
                }
                self.WM_MOUSEMOVE(wparam, lparam)
            }
            WM_MOUSELEAVE => {
                self.set_mouse_tracking(false);
                self.WM_MOUSELEAVE(wparam, lparam)
            }

            // その他のイベント
            WM_ACTIVATE => self.WM_ACTIVATE(wparam, lparam),
            WM_ACTIVATEAPP => self.WM_ACTIVATEAPP(wparam, lparam),
            WM_AFXFIRST => self.WM_AFXFIRST(wparam, lparam),
            WM_AFXLAST => self.WM_AFXLAST(wparam, lparam),
            WM_APPCOMMAND => self.WM_APPCOMMAND(wparam, lparam),
            WM_ASKCBFORMATNAME => self.WM_ASKCBFORMATNAME(wparam, lparam),
            WM_CANCELJOURNAL => self.WM_CANCELJOURNAL(wparam, lparam),
            WM_CANCELMODE => self.WM_CANCELMODE(wparam, lparam),
            WM_CAPTURECHANGED => self.WM_CAPTURECHANGED(wparam, lparam),
            WM_CHANGECBCHAIN => self.WM_CHANGECBCHAIN(wparam, lparam),
            WM_CHANGEUISTATE => self.WM_CHANGEUISTATE(wparam, lparam),
            WM_CHAR => self.WM_CHAR(wparam, lparam),
            WM_CHARTOITEM => self.WM_CHARTOITEM(wparam, lparam),
            WM_CHILDACTIVATE => self.WM_CHILDACTIVATE(wparam, lparam),
            WM_CLEAR => self.WM_CLEAR(wparam, lparam),
            WM_CLIPBOARDUPDATE => self.WM_CLIPBOARDUPDATE(wparam, lparam),
            WM_CLOSE => self.WM_CLOSE(wparam, lparam),
            WM_COMMAND => self.WM_COMMAND(wparam, lparam),
            WM_COMMNOTIFY => self.WM_COMMNOTIFY(wparam, lparam),
            WM_COMPACTING => self.WM_COMPACTING(wparam, lparam),
            WM_COMPAREITEM => self.WM_COMPAREITEM(wparam, lparam),
            WM_CONTEXTMENU => self.WM_CONTEXTMENU(wparam, lparam),
            WM_COPY => self.WM_COPY(wparam, lparam),
            WM_COPYDATA => self.WM_COPYDATA(wparam, lparam),
            WM_CREATE => self.WM_CREATE(wparam, lparam),
            WM_CTLCOLORBTN => self.WM_CTLCOLORBTN(wparam, lparam),
            WM_CTLCOLORDLG => self.WM_CTLCOLORDLG(wparam, lparam),
            WM_CTLCOLOREDIT => self.WM_CTLCOLOREDIT(wparam, lparam),
            WM_CTLCOLORLISTBOX => self.WM_CTLCOLORLISTBOX(wparam, lparam),
            WM_CTLCOLORMSGBOX => self.WM_CTLCOLORMSGBOX(wparam, lparam),
            WM_CTLCOLORSCROLLBAR => self.WM_CTLCOLORSCROLLBAR(wparam, lparam),
            WM_CTLCOLORSTATIC => self.WM_CTLCOLORSTATIC(wparam, lparam),
            WM_CUT => self.WM_CUT(wparam, lparam),
            WM_DEADCHAR => self.WM_DEADCHAR(wparam, lparam),
            WM_DELETEITEM => self.WM_DELETEITEM(wparam, lparam),
            WM_DESTROY => self.WM_DESTROY(wparam, lparam),
            WM_DESTROYCLIPBOARD => self.WM_DESTROYCLIPBOARD(wparam, lparam),
            WM_DEVICECHANGE => self.WM_DEVICECHANGE(wparam, lparam),
            WM_DEVMODECHANGE => self.WM_DEVMODECHANGE(wparam, lparam),
            WM_DISPLAYCHANGE => self.WM_DISPLAYCHANGE(wparam, lparam),
            WM_DPICHANGED => self.WM_DPICHANGED(wparam, lparam),
            WM_DPICHANGED_AFTERPARENT => self.WM_DPICHANGED_AFTERPARENT(wparam, lparam),
            WM_DPICHANGED_BEFOREPARENT => self.WM_DPICHANGED_BEFOREPARENT(wparam, lparam),
            WM_DRAWCLIPBOARD => self.WM_DRAWCLIPBOARD(wparam, lparam),
            WM_DRAWITEM => self.WM_DRAWITEM(wparam, lparam),
            WM_DROPFILES => self.WM_DROPFILES(wparam, lparam),
            WM_DWMCOLORIZATIONCOLORCHANGED => self.WM_DWMCOLORIZATIONCOLORCHANGED(wparam, lparam),
            WM_DWMCOMPOSITIONCHANGED => self.WM_DWMCOMPOSITIONCHANGED(wparam, lparam),
            WM_DWMNCRENDERINGCHANGED => self.WM_DWMNCRENDERINGCHANGED(wparam, lparam),
            WM_DWMSENDICONICLIVEPREVIEWBITMAP => {
                self.WM_DWMSENDICONICLIVEPREVIEWBITMAP(wparam, lparam)
            }
            WM_DWMSENDICONICTHUMBNAIL => self.WM_DWMSENDICONICTHUMBNAIL(wparam, lparam),
            WM_DWMWINDOWMAXIMIZEDCHANGE => self.WM_DWMWINDOWMAXIMIZEDCHANGE(wparam, lparam),
            WM_ENABLE => self.WM_ENABLE(wparam, lparam),
            WM_ENDSESSION => self.WM_ENDSESSION(wparam, lparam),
            WM_ENTERIDLE => self.WM_ENTERIDLE(wparam, lparam),
            WM_ENTERMENULOOP => self.WM_ENTERMENULOOP(wparam, lparam),
            WM_ENTERSIZEMOVE => self.WM_ENTERSIZEMOVE(wparam, lparam),
            WM_ERASEBKGND => self.WM_ERASEBKGND(wparam, lparam),
            WM_EXITMENULOOP => self.WM_EXITMENULOOP(wparam, lparam),
            WM_EXITSIZEMOVE => self.WM_EXITSIZEMOVE(wparam, lparam),
            WM_FONTCHANGE => self.WM_FONTCHANGE(wparam, lparam),
            WM_GESTURE => self.WM_GESTURE(wparam, lparam),
            WM_GESTURENOTIFY => self.WM_GESTURENOTIFY(wparam, lparam),
            WM_GETDLGCODE => self.WM_GETDLGCODE(wparam, lparam),
            WM_GETDPISCALEDSIZE => self.WM_GETDPISCALEDSIZE(wparam, lparam),
            WM_GETFONT => self.WM_GETFONT(wparam, lparam),
            WM_GETHOTKEY => self.WM_GETHOTKEY(wparam, lparam),
            WM_GETICON => self.WM_GETICON(wparam, lparam),
            WM_GETMINMAXINFO => self.WM_GETMINMAXINFO(wparam, lparam),
            WM_GETOBJECT => self.WM_GETOBJECT(wparam, lparam),
            WM_GETTEXT => self.WM_GETTEXT(wparam, lparam),
            WM_GETTEXTLENGTH => self.WM_GETTEXTLENGTH(wparam, lparam),
            WM_GETTITLEBARINFOEX => self.WM_GETTITLEBARINFOEX(wparam, lparam),
            WM_HANDHELDFIRST => self.WM_HANDHELDFIRST(wparam, lparam),
            WM_HANDHELDLAST => self.WM_HANDHELDLAST(wparam, lparam),
            WM_HELP => self.WM_HELP(wparam, lparam),
            WM_HOTKEY => self.WM_HOTKEY(wparam, lparam),
            WM_HSCROLL => self.WM_HSCROLL(wparam, lparam),
            WM_HSCROLLCLIPBOARD => self.WM_HSCROLLCLIPBOARD(wparam, lparam),
            WM_ICONERASEBKGND => self.WM_ICONERASEBKGND(wparam, lparam),
            WM_IME_CHAR => self.WM_IME_CHAR(wparam, lparam),
            WM_IME_COMPOSITION => self.WM_IME_COMPOSITION(wparam, lparam),
            WM_IME_COMPOSITIONFULL => self.WM_IME_COMPOSITIONFULL(wparam, lparam),
            WM_IME_CONTROL => self.WM_IME_CONTROL(wparam, lparam),
            WM_IME_ENDCOMPOSITION => self.WM_IME_ENDCOMPOSITION(wparam, lparam),
            WM_IME_KEYDOWN => self.WM_IME_KEYDOWN(wparam, lparam),
            WM_IME_KEYUP => self.WM_IME_KEYUP(wparam, lparam),
            WM_IME_NOTIFY => self.WM_IME_NOTIFY(wparam, lparam),
            WM_IME_REQUEST => self.WM_IME_REQUEST(wparam, lparam),
            WM_IME_SELECT => self.WM_IME_SELECT(wparam, lparam),
            WM_IME_SETCONTEXT => self.WM_IME_SETCONTEXT(wparam, lparam),
            WM_IME_STARTCOMPOSITION => self.WM_IME_STARTCOMPOSITION(wparam, lparam),
            WM_INITDIALOG => self.WM_INITDIALOG(wparam, lparam),
            WM_INITMENU => self.WM_INITMENU(wparam, lparam),
            WM_INITMENUPOPUP => self.WM_INITMENUPOPUP(wparam, lparam),
            WM_INPUT => self.WM_INPUT(wparam, lparam),
            WM_INPUTLANGCHANGE => self.WM_INPUTLANGCHANGE(wparam, lparam),
            WM_INPUTLANGCHANGEREQUEST => self.WM_INPUTLANGCHANGEREQUEST(wparam, lparam),
            WM_INPUT_DEVICE_CHANGE => self.WM_INPUT_DEVICE_CHANGE(wparam, lparam),
            WM_KEYDOWN => self.WM_KEYDOWN(wparam, lparam),
            WM_KEYUP => self.WM_KEYUP(wparam, lparam),
            WM_KILLFOCUS => self.WM_KILLFOCUS(wparam, lparam),
            WM_LBUTTONDBLCLK => self.WM_LBUTTONDBLCLK(wparam, lparam),
            WM_LBUTTONDOWN => self.WM_LBUTTONDOWN(wparam, lparam),
            WM_LBUTTONUP => self.WM_LBUTTONUP(wparam, lparam),
            WM_MBUTTONDBLCLK => self.WM_MBUTTONDBLCLK(wparam, lparam),
            WM_MBUTTONDOWN => self.WM_MBUTTONDOWN(wparam, lparam),
            WM_MBUTTONUP => self.WM_MBUTTONUP(wparam, lparam),
            WM_MDIACTIVATE => self.WM_MDIACTIVATE(wparam, lparam),
            WM_MDICASCADE => self.WM_MDICASCADE(wparam, lparam),
            WM_MDICREATE => self.WM_MDICREATE(wparam, lparam),
            WM_MDIDESTROY => self.WM_MDIDESTROY(wparam, lparam),
            WM_MDIGETACTIVE => self.WM_MDIGETACTIVE(wparam, lparam),
            WM_MDIICONARRANGE => self.WM_MDIICONARRANGE(wparam, lparam),
            WM_MDIMAXIMIZE => self.WM_MDIMAXIMIZE(wparam, lparam),
            WM_MDINEXT => self.WM_MDINEXT(wparam, lparam),
            WM_MDIREFRESHMENU => self.WM_MDIREFRESHMENU(wparam, lparam),
            WM_MDIRESTORE => self.WM_MDIRESTORE(wparam, lparam),
            WM_MDISETMENU => self.WM_MDISETMENU(wparam, lparam),
            WM_MDITILE => self.WM_MDITILE(wparam, lparam),
            WM_MEASUREITEM => self.WM_MEASUREITEM(wparam, lparam),
            WM_MENUCHAR => self.WM_MENUCHAR(wparam, lparam),
            WM_MENUCOMMAND => self.WM_MENUCOMMAND(wparam, lparam),
            WM_MENUDRAG => self.WM_MENUDRAG(wparam, lparam),
            WM_MENUGETOBJECT => self.WM_MENUGETOBJECT(wparam, lparam),
            WM_MENURBUTTONUP => self.WM_MENURBUTTONUP(wparam, lparam),
            WM_MENUSELECT => self.WM_MENUSELECT(wparam, lparam),
            WM_MOUSEACTIVATE => self.WM_MOUSEACTIVATE(wparam, lparam),
            WM_MOUSEHWHEEL => self.WM_MOUSEHWHEEL(wparam, lparam),
            WM_MOUSEWHEEL => self.WM_MOUSEWHEEL(wparam, lparam),
            WM_MOVE => self.WM_MOVE(wparam, lparam),
            WM_MOVING => self.WM_MOVING(wparam, lparam),
            WM_NCACTIVATE => self.WM_NCACTIVATE(wparam, lparam),
            WM_NCCALCSIZE => self.WM_NCCALCSIZE(wparam, lparam),
            WM_NCDESTROY => self.WM_NCDESTROY(wparam, lparam),
            WM_NCHITTEST => self.WM_NCHITTEST(wparam, lparam),
            WM_NCLBUTTONDBLCLK => self.WM_NCLBUTTONDBLCLK(wparam, lparam),
            WM_NCLBUTTONDOWN => self.WM_NCLBUTTONDOWN(wparam, lparam),
            WM_NCLBUTTONUP => self.WM_NCLBUTTONUP(wparam, lparam),
            WM_NCMBUTTONDBLCLK => self.WM_NCMBUTTONDBLCLK(wparam, lparam),
            WM_NCMBUTTONDOWN => self.WM_NCMBUTTONDOWN(wparam, lparam),
            WM_NCMBUTTONUP => self.WM_NCMBUTTONUP(wparam, lparam),
            WM_NCMOUSEHOVER => self.WM_NCMOUSEHOVER(wparam, lparam),
            WM_NCMOUSELEAVE => self.WM_NCMOUSELEAVE(wparam, lparam),
            WM_NCMOUSEMOVE => self.WM_NCMOUSEMOVE(wparam, lparam),
            WM_NCPAINT => self.WM_NCPAINT(wparam, lparam),
            WM_NCPOINTERDOWN => self.WM_NCPOINTERDOWN(wparam, lparam),
            WM_NCPOINTERUP => self.WM_NCPOINTERUP(wparam, lparam),
            WM_NCPOINTERUPDATE => self.WM_NCPOINTERUPDATE(wparam, lparam),
            WM_NCRBUTTONDBLCLK => self.WM_NCRBUTTONDBLCLK(wparam, lparam),
            WM_NCRBUTTONDOWN => self.WM_NCRBUTTONDOWN(wparam, lparam),
            WM_NCRBUTTONUP => self.WM_NCRBUTTONUP(wparam, lparam),
            WM_NCXBUTTONDBLCLK => self.WM_NCXBUTTONDBLCLK(wparam, lparam),
            WM_NCXBUTTONDOWN => self.WM_NCXBUTTONDOWN(wparam, lparam),
            WM_NCXBUTTONUP => self.WM_NCXBUTTONUP(wparam, lparam),
            WM_NEXTDLGCTL => self.WM_NEXTDLGCTL(wparam, lparam),
            WM_NEXTMENU => self.WM_NEXTMENU(wparam, lparam),
            WM_NOTIFY => self.WM_NOTIFY(wparam, lparam),
            WM_NOTIFYFORMAT => self.WM_NOTIFYFORMAT(wparam, lparam),
            WM_NULL => self.WM_NULL(wparam, lparam),
            WM_PAINT => self.WM_PAINT(wparam, lparam),
            WM_PAINTCLIPBOARD => self.WM_PAINTCLIPBOARD(wparam, lparam),
            WM_PAINTICON => self.WM_PAINTICON(wparam, lparam),
            WM_PALETTECHANGED => self.WM_PALETTECHANGED(wparam, lparam),
            WM_PALETTEISCHANGING => self.WM_PALETTEISCHANGING(wparam, lparam),
            WM_PARENTNOTIFY => self.WM_PARENTNOTIFY(wparam, lparam),
            WM_PASTE => self.WM_PASTE(wparam, lparam),
            WM_PENWINFIRST => self.WM_PENWINFIRST(wparam, lparam),
            WM_PENWINLAST => self.WM_PENWINLAST(wparam, lparam),
            WM_POINTERACTIVATE => self.WM_POINTERACTIVATE(wparam, lparam),
            WM_POINTERCAPTURECHANGED => self.WM_POINTERCAPTURECHANGED(wparam, lparam),
            WM_POINTERDEVICECHANGE => self.WM_POINTERDEVICECHANGE(wparam, lparam),
            WM_POINTERDEVICEINRANGE => self.WM_POINTERDEVICEINRANGE(wparam, lparam),
            WM_POINTERDEVICEOUTOFRANGE => self.WM_POINTERDEVICEOUTOFRANGE(wparam, lparam),
            WM_POINTERDOWN => self.WM_POINTERDOWN(wparam, lparam),
            WM_POINTERENTER => self.WM_POINTERENTER(wparam, lparam),
            WM_POINTERHWHEEL => self.WM_POINTERHWHEEL(wparam, lparam),
            WM_POINTERLEAVE => self.WM_POINTERLEAVE(wparam, lparam),
            WM_POINTERROUTEDAWAY => self.WM_POINTERROUTEDAWAY(wparam, lparam),
            WM_POINTERROUTEDRELEASED => self.WM_POINTERROUTEDRELEASED(wparam, lparam),
            WM_POINTERROUTEDTO => self.WM_POINTERROUTEDTO(wparam, lparam),
            WM_POINTERUP => self.WM_POINTERUP(wparam, lparam),
            WM_POINTERUPDATE => self.WM_POINTERUPDATE(wparam, lparam),
            WM_POINTERWHEEL => self.WM_POINTERWHEEL(wparam, lparam),
            WM_POWER => self.WM_POWER(wparam, lparam),
            WM_POWERBROADCAST => self.WM_POWERBROADCAST(wparam, lparam),
            WM_PRINT => self.WM_PRINT(wparam, lparam),
            WM_PRINTCLIENT => self.WM_PRINTCLIENT(wparam, lparam),
            WM_QUERYDRAGICON => self.WM_QUERYDRAGICON(wparam, lparam),
            WM_QUERYENDSESSION => self.WM_QUERYENDSESSION(wparam, lparam),
            WM_QUERYNEWPALETTE => self.WM_QUERYNEWPALETTE(wparam, lparam),
            WM_QUERYOPEN => self.WM_QUERYOPEN(wparam, lparam),
            WM_QUERYUISTATE => self.WM_QUERYUISTATE(wparam, lparam),
            WM_QUEUESYNC => self.WM_QUEUESYNC(wparam, lparam),
            WM_QUIT => self.WM_QUIT(wparam, lparam),
            WM_RBUTTONDBLCLK => self.WM_RBUTTONDBLCLK(wparam, lparam),
            WM_RBUTTONDOWN => self.WM_RBUTTONDOWN(wparam, lparam),
            WM_RBUTTONUP => self.WM_RBUTTONUP(wparam, lparam),
            WM_RENDERALLFORMATS => self.WM_RENDERALLFORMATS(wparam, lparam),
            WM_RENDERFORMAT => self.WM_RENDERFORMAT(wparam, lparam),
            WM_SETCURSOR => self.WM_SETCURSOR(wparam, lparam),
            WM_SETFOCUS => self.WM_SETFOCUS(wparam, lparam),
            WM_SETFONT => self.WM_SETFONT(wparam, lparam),
            WM_SETHOTKEY => self.WM_SETHOTKEY(wparam, lparam),
            WM_SETICON => self.WM_SETICON(wparam, lparam),
            WM_SETREDRAW => self.WM_SETREDRAW(wparam, lparam),
            WM_SETTEXT => self.WM_SETTEXT(wparam, lparam),
            WM_SETTINGCHANGE => self.WM_SETTINGCHANGE(wparam, lparam),
            WM_SHOWWINDOW => self.WM_SHOWWINDOW(wparam, lparam),
            WM_SIZE => self.WM_SIZE(wparam, lparam),
            WM_SIZECLIPBOARD => self.WM_SIZECLIPBOARD(wparam, lparam),
            WM_SIZING => self.WM_SIZING(wparam, lparam),
            WM_SPOOLERSTATUS => self.WM_SPOOLERSTATUS(wparam, lparam),
            WM_STYLECHANGED => self.WM_STYLECHANGED(wparam, lparam),
            WM_STYLECHANGING => self.WM_STYLECHANGING(wparam, lparam),
            WM_SYNCPAINT => self.WM_SYNCPAINT(wparam, lparam),
            WM_SYSCHAR => self.WM_SYSCHAR(wparam, lparam),
            WM_SYSCOLORCHANGE => self.WM_SYSCOLORCHANGE(wparam, lparam),
            WM_SYSCOMMAND => self.WM_SYSCOMMAND(wparam, lparam),
            WM_SYSDEADCHAR => self.WM_SYSDEADCHAR(wparam, lparam),
            WM_SYSKEYDOWN => self.WM_SYSKEYDOWN(wparam, lparam),
            WM_SYSKEYUP => self.WM_SYSKEYUP(wparam, lparam),
            WM_TABLET_FIRST => self.WM_TABLET_FIRST(wparam, lparam),
            WM_TABLET_LAST => self.WM_TABLET_LAST(wparam, lparam),
            WM_TCARD => self.WM_TCARD(wparam, lparam),
            WM_THEMECHANGED => self.WM_THEMECHANGED(wparam, lparam),
            WM_TIMECHANGE => self.WM_TIMECHANGE(wparam, lparam),
            WM_TIMER => self.WM_TIMER(wparam, lparam),
            WM_TOOLTIPDISMISS => self.WM_TOOLTIPDISMISS(wparam, lparam),
            WM_TOUCH => self.WM_TOUCH(wparam, lparam),
            WM_TOUCHHITTESTING => self.WM_TOUCHHITTESTING(wparam, lparam),
            WM_UNDO => self.WM_UNDO(wparam, lparam),
            WM_UNICHAR => self.WM_UNICHAR(wparam, lparam),
            WM_UNINITMENUPOPUP => self.WM_UNINITMENUPOPUP(wparam, lparam),
            WM_UPDATEUISTATE => self.WM_UPDATEUISTATE(wparam, lparam),
            WM_USERCHANGED => self.WM_USERCHANGED(wparam, lparam),
            WM_VKEYTOITEM => self.WM_VKEYTOITEM(wparam, lparam),
            WM_VSCROLL => self.WM_VSCROLL(wparam, lparam),
            WM_VSCROLLCLIPBOARD => self.WM_VSCROLLCLIPBOARD(wparam, lparam),
            WM_WINDOWPOSCHANGED => self.WM_WINDOWPOSCHANGED(wparam, lparam),
            WM_WINDOWPOSCHANGING => self.WM_WINDOWPOSCHANGING(wparam, lparam),
            WM_WTSSESSION_CHANGE => self.WM_WTSSESSION_CHANGE(wparam, lparam),
            WM_XBUTTONDBLCLK => self.WM_XBUTTONDBLCLK(wparam, lparam),
            WM_XBUTTONDOWN => self.WM_XBUTTONDOWN(wparam, lparam),
            WM_XBUTTONUP => self.WM_XBUTTONUP(wparam, lparam),

            // その他
            _ => None,
        };
        if let Some(res) = handled {
            return res;
        }

        // DWMのデフォルト
        let mut res = LRESULT(0);
        if unsafe { DwmDefWindowProc(hwnd, msg, wparam, lparam, &mut res as *mut _) }.as_bool() {
            return res;
        }

        // デフォルト処理
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }
}
