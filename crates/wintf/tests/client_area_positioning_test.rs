//! client-area-positioning機能のテスト
//!
//! TDD: RED フェーズ - テストを先に記述

use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

/// Task 3.1: WS_OVERLAPPEDWINDOWスタイルでの座標変換テスト
/// 標準ウィンドウスタイルでto_window_coordsの動作を検証
#[test]
fn test_to_window_coords_overlapped_window() {
    // 実際のウィンドウを作成してテストする
    // ウィンドウ作成
    let hwnd = create_test_window(WS_OVERLAPPEDWINDOW | WS_VISIBLE, WS_EX_NOREDIRECTIONBITMAP);
    assert!(!hwnd.is_invalid(), "テストウィンドウの作成に失敗");

    // WindowPosを作成 - クライアント領域 (100, 100, 800, 600)
    let window_pos = wintf::ecs::window::WindowPos::new()
        .with_position(POINT { x: 100, y: 100 })
        .with_size(SIZE { cx: 800, cy: 600 });

    // 座標変換を実行
    let window_handle = wintf::ecs::window::WindowHandle {
        hwnd,
        instance: windows::Win32::Foundation::HINSTANCE::default(),
    };
    let result = window_pos.to_window_coords(&window_handle);

    // クリーンアップ
    unsafe {
        let _ = DestroyWindow(hwnd);
    }

    // 変換成功を確認
    let (x, y, width, height) = result.expect("座標変換が成功すること");

    // WS_OVERLAPPEDWINDOWスタイルでは、タイトルバーとボーダーが付くため:
    // - x座標はクライアント領域より左にオフセット
    // - y座標はクライアント領域より上にオフセット（タイトルバー分）
    // - width/heightはボーダー分だけ大きくなる

    // 座標がクライアント領域以下になっていることを確認（左・上にオフセット）
    assert!(x <= 100, "x座標がクライアント領域以下であること: got {}", x);
    assert!(
        y <= 100,
        "y座標がクライアント領域以下であること (タイトルバー分): got {}",
        y
    );

    // サイズがクライアント領域より大きいことを確認
    assert!(
        width >= 800,
        "幅がクライアント領域以上であること: got {}",
        width
    );
    assert!(
        height >= 600,
        "高さがクライアント領域以上であること: got {}",
        height
    );
}

/// Task 3.2: WS_POPUPスタイルでの座標変換テスト【必須】
/// ボーダーレスウィンドウでの変換動作を検証
#[test]
fn test_to_window_coords_popup_window() {
    // WS_POPUPウィンドウを作成（タイトルバー・ボーダーなし）
    let hwnd = create_test_window(WS_POPUP | WS_VISIBLE, WINDOW_EX_STYLE(0));
    assert!(!hwnd.is_invalid(), "テストウィンドウの作成に失敗");

    // WindowPosを作成 - クライアント領域 (100, 100, 800, 600)
    let window_pos = wintf::ecs::window::WindowPos::new()
        .with_position(POINT { x: 100, y: 100 })
        .with_size(SIZE { cx: 800, cy: 600 });

    // 座標変換を実行
    let window_handle = wintf::ecs::window::WindowHandle {
        hwnd,
        instance: windows::Win32::Foundation::HINSTANCE::default(),
    };
    let result = window_pos.to_window_coords(&window_handle);

    // クリーンアップ
    unsafe {
        let _ = DestroyWindow(hwnd);
    }

    // 変換成功を確認
    let (x, y, width, height) = result.expect("座標変換が成功すること");

    // WS_POPUPスタイルでは装飾なしのため、入力座標と同一であること
    assert_eq!(x, 100, "x座標がクライアント領域と同一であること");
    assert_eq!(y, 100, "y座標がクライアント領域と同一であること");
    assert_eq!(width, 800, "幅がクライアント領域と同一であること");
    assert_eq!(height, 600, "高さがクライアント領域と同一であること");
}

/// Task 3.3: エラーハンドリングテスト
/// 無効なHWND(HWND(0))を渡した場合のテスト
#[test]
fn test_to_window_coords_invalid_hwnd() {
    // 無効なHWND
    let invalid_hwnd = HWND(std::ptr::null_mut());

    // WindowPosを作成
    let window_pos = wintf::ecs::window::WindowPos::new()
        .with_position(POINT { x: 100, y: 100 })
        .with_size(SIZE { cx: 800, cy: 600 });

    // 座標変換を実行 - Errが返されることを期待
    let window_handle = wintf::ecs::window::WindowHandle {
        hwnd: invalid_hwnd,
        instance: windows::Win32::Foundation::HINSTANCE::default(),
    };
    let result = window_pos.to_window_coords(&window_handle);

    // エラーが返されることを確認
    assert!(result.is_err(), "無効なHWNDでErrが返されること");

    // エラーメッセージが設定されていることを確認
    let error_msg = result.unwrap_err();
    assert!(!error_msg.is_empty(), "エラーメッセージが空でないこと");
}

/// テスト用ウィンドウを作成するヘルパー関数
fn create_test_window(style: WINDOW_STYLE, ex_style: WINDOW_EX_STYLE) -> HWND {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;

    let class_name: Vec<u16> = OsStr::new("ClientAreaTestWindow")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap_or_default();

        // ウィンドウクラス登録
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(test_window_proc),
            hInstance: hinstance.into(),
            lpszClassName: windows::core::PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        RegisterClassExW(&wc);

        // ウィンドウ作成
        CreateWindowExW(
            ex_style,
            windows::core::PCWSTR(class_name.as_ptr()),
            windows::core::PCWSTR(class_name.as_ptr()),
            style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(hinstance.into()),
            None,
        )
        .unwrap_or_default()
    }
}

/// テスト用ウィンドウプロシージャ
unsafe extern "system" fn test_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    DefWindowProcW(hwnd, msg, wparam, lparam)
}
