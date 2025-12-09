//! # WM_NCHITTEST キャッシュ
//!
//! WM_NCHITTEST の高頻度呼び出しに対するパフォーマンス最適化を提供する。
//! 同一座標での重複ヒットテストをスキップし、World 借用オーバーヘッドを削減する。
//!
//! ## 設計
//! - thread_local! + RefCell パターンで内部可変性を提供
//! - HWND をキーとしたエントリ管理（HashMap）
//! - try_tick_world() 終了時に全エントリをクリア

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use tracing::trace;
use windows::Win32::Foundation::{HWND, LRESULT, POINT};
use windows::Win32::Graphics::Gdi::ScreenToClient;

use crate::ecs::layout::hit_test::{hit_test_in_window, PhysicalPoint};
use crate::ecs::world::EcsWorld;

// ============================================================================
// キャッシュエントリ
// ============================================================================

/// キャッシュエントリ
struct NchittestCacheEntry {
    /// スクリーン座標（物理ピクセル）
    screen_point: (i32, i32),
    /// WM_NCHITTEST 戻り値
    lresult: LRESULT,
}

// ============================================================================
// スレッドローカルキャッシュ
// ============================================================================

thread_local! {
    /// HWND ごとの WM_NCHITTEST 結果キャッシュ
    static NCHITTEST_CACHE: RefCell<HashMap<isize, NchittestCacheEntry>>
        = RefCell::new(HashMap::new());
}

// ============================================================================
// キャッシュ操作API
// ============================================================================

/// キャッシュルックアップ
fn lookup(hwnd: HWND, screen_point: (i32, i32)) -> Option<LRESULT> {
    NCHITTEST_CACHE.with(|cache| {
        let cache = cache.borrow();
        if let Some(entry) = cache.get(&(hwnd.0 as isize)) {
            if entry.screen_point == screen_point {
                return Some(entry.lresult);
            }
        }
        None
    })
}

/// キャッシュ挿入
fn insert(hwnd: HWND, screen_point: (i32, i32), lresult: LRESULT) {
    NCHITTEST_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.insert(
            hwnd.0 as isize,
            NchittestCacheEntry {
                screen_point,
                lresult,
            },
        );
    });
}

// ============================================================================
// 公開API
// ============================================================================

/// キャッシュ付き WM_NCHITTEST 処理
///
/// # Arguments
/// - `hwnd`: ウィンドウハンドル
/// - `screen_point`: スクリーン座標（lparam から取得済み）
/// - `entity`: ウィンドウエンティティ
/// - `ecs_world`: ECS World（借用元）
///
/// # Returns
/// - `Some(LRESULT)`: HTCLIENT (1) または HTTRANSPARENT (-1)
/// - `None`: 処理失敗時（DefWindowProcW に委譲）
pub fn cached_nchittest(
    hwnd: HWND,
    screen_point: (i32, i32),
    entity: bevy_ecs::prelude::Entity,
    ecs_world: &Rc<RefCell<EcsWorld>>,
) -> Option<LRESULT> {
    // キャッシュヒット判定
    if let Some(lresult) = lookup(hwnd, screen_point) {
        trace!(
            hwnd = ?hwnd,
            x = screen_point.0,
            y = screen_point.1,
            lresult = lresult.0,
            "NCHITTEST cache hit"
        );
        return Some(lresult);
    }

    // キャッシュミス: クライアント座標に変換
    let mut pt = POINT {
        x: screen_point.0,
        y: screen_point.1,
    };
    // SAFETY: ScreenToClient は HWND と POINT への有効なポインタを必要とする
    if unsafe { !ScreenToClient(hwnd, &mut pt).as_bool() } {
        return None;
    }

    // World 借用して hit_test 実行
    let hit_result = match ecs_world.try_borrow() {
        Ok(world_ref) => hit_test_in_window(
            world_ref.world(),
            entity,
            PhysicalPoint::new(pt.x as f32, pt.y as f32),
        ),
        Err(_) => {
            return None; // 借用失敗時は DefWindowProcW に委譲
        }
    };

    // HTCLIENT = 1, HTTRANSPARENT = -1
    const HTCLIENT: i32 = 1;
    const HTTRANSPARENT: i32 = -1;

    // 常にHTCLIENTを返す（クライアント領域としてマウスイベントを受け取る）
    // hit_testの結果に関わらず、ウィンドウ内のクリックは全て受け取る必要がある
    // HTTRANSPARENT を返すとマウスイベントがブロックされてしまう
    let lresult = LRESULT(HTCLIENT as isize);

    // キャッシュに挿入
    insert(hwnd, screen_point, lresult);

    trace!(
        hwnd = ?hwnd,
        x = screen_point.0,
        y = screen_point.1,
        lresult = lresult.0,
        hit_entity = ?hit_result,
        "NCHITTEST cache miss"
    );

    Some(lresult)
}

/// キャッシュをクリア
///
/// try_tick_world() 終了時に呼び出す。
/// 全ウィンドウのキャッシュエントリを削除する。
pub fn clear_nchittest_cache() {
    NCHITTEST_CACHE.with(|cache| {
        cache.borrow_mut().clear();
    });
}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// キャッシュルックアップとインサートの基本動作
    #[test]
    fn test_cache_lookup_insert() {
        // テスト用のダミー HWND
        let hwnd = HWND(12345 as *mut std::ffi::c_void);
        let screen_point = (100, 200);
        let lresult = LRESULT(1);

        // キャッシュが空の状態でルックアップ
        assert!(lookup(hwnd, screen_point).is_none());

        // キャッシュに挿入
        insert(hwnd, screen_point, lresult);

        // 同じ座標でルックアップ
        assert_eq!(lookup(hwnd, screen_point), Some(lresult));

        // 異なる座標でルックアップ
        assert!(lookup(hwnd, (101, 200)).is_none());

        // クリーンアップ
        clear_nchittest_cache();
    }

    /// 異なる HWND で独立したキャッシュ
    #[test]
    fn test_cache_multiple_hwnds() {
        let hwnd1 = HWND(111 as *mut std::ffi::c_void);
        let hwnd2 = HWND(222 as *mut std::ffi::c_void);
        let screen_point = (50, 50);
        let lresult1 = LRESULT(1);
        let lresult2 = LRESULT(-1);

        insert(hwnd1, screen_point, lresult1);
        insert(hwnd2, screen_point, lresult2);

        assert_eq!(lookup(hwnd1, screen_point), Some(lresult1));
        assert_eq!(lookup(hwnd2, screen_point), Some(lresult2));

        // クリーンアップ
        clear_nchittest_cache();
    }

    /// キャッシュクリアの動作確認
    #[test]
    fn test_cache_clear() {
        let hwnd = HWND(999 as *mut std::ffi::c_void);
        let screen_point = (10, 20);
        let lresult = LRESULT(1);

        insert(hwnd, screen_point, lresult);
        assert!(lookup(hwnd, screen_point).is_some());

        clear_nchittest_cache();
        assert!(lookup(hwnd, screen_point).is_none());
    }

    /// キャッシュ更新の動作確認
    #[test]
    fn test_cache_update() {
        let hwnd = HWND(777 as *mut std::ffi::c_void);
        let point1 = (100, 100);
        let point2 = (200, 200);
        let lresult1 = LRESULT(1);
        let lresult2 = LRESULT(-1);

        // 最初の座標を挿入
        insert(hwnd, point1, lresult1);
        assert_eq!(lookup(hwnd, point1), Some(lresult1));

        // 異なる座標で上書き
        insert(hwnd, point2, lresult2);
        // 古い座標はヒットしない
        assert!(lookup(hwnd, point1).is_none());
        // 新しい座標がヒット
        assert_eq!(lookup(hwnd, point2), Some(lresult2));

        // クリーンアップ
        clear_nchittest_cache();
    }
}
