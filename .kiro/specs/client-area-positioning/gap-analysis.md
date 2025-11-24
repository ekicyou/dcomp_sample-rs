# Gap Analysis: client-area-positioning

## 1. Current State Investigation

### Key Files and Modules

**コア実装ファイル**:
- `crates/wintf/src/ecs/graphics/systems.rs` - `apply_window_pos_changes`システムの実装場所（520行目）
- `crates/wintf/src/ecs/window.rs` - `WindowPos`コンポーネント定義（174行目）、`WindowStyle`コンポーネント（122行目）
- `crates/wintf/src/win_state.rs` - `effective_window_size`メソッド（既存のクライアント領域→ウィンドウサイズ変換実装）
- `crates/wintf/src/api.rs` - `get_window_long_ptr`ヘルパー関数（10行目）

**統合ポイント**:
- `crates/wintf/src/ecs/world.rs` - システム登録（153-157行目、`apply_window_pos_changes`を`UISetup`スケジュールに登録）
- `crates/wintf/src/ecs/window_proc.rs` - `WM_WINDOWPOSCHANGED`メッセージハンドラ（エコーバック処理、78-113行目）
- `crates/wintf/examples/taffy_flex_demo.rs` - テストアプリケーション（`POINT { x: 100, y: 100 }`設定済み）

### Reusable Components and Patterns

**既存の座標変換実装**:
```rust
// win_state.rs: effective_window_size メソッド（39-65行目）
fn effective_window_size(&self, client_size: Vector2) -> Result<Vector2> {
    let dpi = self.dpi();
    let scale = dpi / 96.0;
    // クライアント領域をピクセルに変換
    let client_size_px = Vector2 { X: client_size.X * scale, Y: client_size.Y * scale };
    
    let mut rect = RECT { left: 0, top: 0, right: ..., bottom: ... };
    let hwnd = self.hwnd();
    let style = WINDOW_STYLE(get_window_long_ptr(hwnd, GWL_STYLE)? as u32);
    let ex_style = WINDOW_EX_STYLE(get_window_long_ptr(hwnd, GWL_EXSTYLE)? as u32);
    let dpi_value = dpi as u32;
    unsafe { AdjustWindowRectExForDpi(&mut rect, style, false, ex_style, dpi_value)? }
    // ウィンドウ全体のサイズを返す
}
```

この実装は**サイズ変換**を行っているが、**位置変換**は含まれていない。

**エコーバックメカニズム**:
- `WindowPos::is_echo()` - SetWindowPosで送信した値とWM_WINDOWPOSCHANGEDで受信した値を比較（432行目）
- `last_sent_position` / `last_sent_size` - エコーバック検知用フィールド（195-196行目）
- `apply_window_pos_changes`で送信後に`last_sent_*`を記録（551-553行目）

### Architecture Patterns and Constraints

**レイヤー分離**:
- COM API呼び出し → `unsafe`ブロック隔離
- ECSコンポーネント → Win32 APIラッパー呼び出し
- システム → コンポーネント変更検知 + APIラッパー呼び出し

**DPI情報へのアクセス**:
- `DpiTransform`コンポーネント（87行目）: ECSコンポーネントとして存在するが、**ウィンドウEntityには自動的に付与されていない**
- `WinState::dpi()` trait: 非ECSパターン（外部からDPI値を取得）
- **制約**: `apply_window_pos_changes`システム内でDPI値にアクセスする方法が不明確

### Integration Surfaces

**クエリパターン**:
```rust
// apply_window_pos_changes の現在のクエリ（520-535行目）
Query<
    (Entity, &WindowHandle, &mut WindowPos),
    (Changed<WindowPos>, With<Window>)
>
```

**API呼び出しパターン**:
```rust
// WindowPos::set_window_pos (400行目)
pub fn set_window_pos(&self, hwnd: HWND) -> windows::core::Result<()>
```

## 2. Requirements Feasibility Analysis

### Technical Needs from EARS Requirements

**Requirement 1: クライアント領域座標の調整機能**
- **必要な機能**: `WindowPos::position`/`size` → クライアント領域基準の座標変換
- **既存資産**: `effective_window_size` (サイズ変換のみ)、`AdjustWindowRectExForDpi` API呼び出しパターン
- **ギャップ**: 位置変換ロジックが未実装（`rect.left`/`rect.top`を使った位置オフセット計算）

**Requirement 2: ウィンドウスタイル情報の取得**
- **必要な機能**: HWND → `WINDOW_STYLE` / `WINDOW_EX_STYLE` / DPI値
- **既存資産**: `get_window_long_ptr`ヘルパー、`WindowStyle::from_hwnd`（138行目）
- **ギャップ**: DPI値取得方法が不明確（`DpiTransform`コンポーネントが自動付与されていない）

**Requirement 3: エラーハンドリングと既存動作の保持**
- **必要な機能**: 変換失敗時のフォールバック、エコーバックメカニズムの維持
- **既存資産**: `is_echo`メカニズム完備、`Result<T>`型使用慣習
- **ギャップ**: なし（既存パターンを踏襲すれば実装可能）

**Requirement 4: CW_USEDEFAULTと特殊値の扱い**
- **必要な機能**: `CW_USEDEFAULT`チェック、座標変換スキップ
- **既存資産**: `apply_window_pos_changes`内で既にチェック実装（542-545行目）
- **ギャップ**: なし（既存チェックの前に変換処理を挿入すれば対応可能）

**Requirement 5: テストアプリケーション動作確認**
- **必要な機能**: `taffy_flex_demo`での動作検証
- **既存資産**: `taffy_flex_demo`に`POINT { x: 100, y: 100 }`設定済み（70行目）
- **ギャップ**: なし（実装後に実行して検証するのみ）

### Gaps and Constraints

**Missing Capabilities**:
1. **位置変換ロジック**: `AdjustWindowRectExForDpi`で得られた`rect.left`/`rect.top`を使った位置オフセット計算
2. **DPI値取得**: `apply_window_pos_changes`システム内でDPI値にアクセスする方法

**Research Needed**:
- **DPI値の取得方法**: 以下のいずれかを調査
  - Option A: `GetDpiForWindow` Win32 APIを直接呼び出し（HWNDから取得）
  - Option B: `DpiTransform`コンポーネントをウィンドウ作成時に自動付与し、クエリに追加
  - Option C: `WinState` traitを実装したリソースからDPI値を取得

**Constraints**:
- エコーバックメカニズムを破壊しない（`last_sent_position`/`last_sent_size`は**変換後の値**を記録する必要がある）
- メインスレッド固定（`apply_window_pos_changes`は`UISetup`スケジュールに登録済み）
- `unsafe`ブロック最小化（Win32 API呼び出しのみに限定）

### Complexity Signals

**実装複雑度**: 低～中
- 既存の`effective_window_size`パターンを流用可能
- Win32 API呼び出しは既存コードで実績あり
- エラーハンドリングパターンも確立済み

**統合複雑度**: 低
- `apply_window_pos_changes`関数内に局所的な変更（10-20行程度の追加）
- 既存システムスケジュールの変更不要
- 他のシステムへの影響なし

## 3. Implementation Approach Options

### Option A: Extend `apply_window_pos_changes` with Inline Conversion

**戦略**: `apply_window_pos_changes`システム関数内で座標変換ロジックをインライン実装

**変更対象ファイル**:
- `crates/wintf/src/ecs/graphics/systems.rs` (520行目～) - `apply_window_pos_changes`関数内に変換処理を追加

**実装アプローチ**:
```rust
pub fn apply_window_pos_changes(query: Query<...>) {
    for (_entity, window_handle, mut window_pos) in query.iter_mut() {
        // 1. エコーバックチェック（既存）
        // 2. CW_USEDEFAULTチェック（既存）
        
        // 3. 座標変換（新規追加）
        let (adjusted_x, adjusted_y, adjusted_width, adjusted_height) = 
            adjust_client_to_window_coords(
                window_handle.hwnd,
                window_pos.position,
                window_pos.size
            )?;
        
        // 4. SetWindowPos呼び出し（変換後の値を使用）
        // 5. last_sent記録（変換後の値を記録）
    }
}

// ヘルパー関数（systems.rsに追加）
fn adjust_client_to_window_coords(
    hwnd: HWND,
    client_position: Option<POINT>,
    client_size: Option<SIZE>
) -> Result<(i32, i32, i32, i32)> {
    // GetDpiForWindow or DPI取得ロジック
    // get_window_long_ptr で style/ex_style取得
    // AdjustWindowRectExForDpi 呼び出し
    // rect.left/top/right/bottom から変換値を計算
}
```

**互換性評価**:
- ✅ 既存の`apply_window_pos_changes`シグネチャは変更なし
- ✅ エコーバックメカニズムを破壊しない（変換後の値を記録）
- ✅ 他のシステムへの影響なし

**複雑度と保守性**:
- ✅ 変更箇所が局所的（1ファイル、1関数内）
- ✅ 認知負荷は中程度（変換ロジックがインラインで追加される）
- ⚠️ `apply_window_pos_changes`関数が肥大化（現在38行 → 60-70行程度）

**Trade-offs**:
- ✅ 最小限の変更で実装可能
- ✅ 既存パターンを踏襲
- ✅ テストが容易（1システム関数のみ）
- ❌ ヘルパー関数が`systems.rs`内に増える（モジュール内結合度が上がる）
- ❌ 座標変換ロジックが他の場所で再利用できない

### Option B: Create New Conversion Module

**戦略**: 座標変換ロジックを専用モジュール化し、`WindowPos`にメソッド追加

**新規作成ファイル**:
- `crates/wintf/src/ecs/window_coords.rs` - 座標変換ロジック専用モジュール

**変更対象ファイル**:
- `crates/wintf/src/ecs/window.rs` - `WindowPos`に`adjust_for_client_area`メソッド追加
- `crates/wintf/src/ecs/graphics/systems.rs` - `apply_window_pos_changes`で新メソッド呼び出し
- `crates/wintf/src/ecs/mod.rs` - 新モジュール`pub use`

**実装アプローチ**:
```rust
// window_coords.rs (新規作成)
pub struct WindowCoordinateConverter {
    hwnd: HWND,
    style: WINDOW_STYLE,
    ex_style: WINDOW_EX_STYLE,
    dpi: u32,
}

impl WindowCoordinateConverter {
    pub fn from_hwnd(hwnd: HWND) -> Result<Self> { ... }
    
    pub fn client_to_window_position(&self, client_pos: POINT) -> POINT { ... }
    pub fn client_to_window_size(&self, client_size: SIZE) -> SIZE { ... }
}

// window.rs (WindowPosに追加)
impl WindowPos {
    pub fn adjust_for_client_area(&self, hwnd: HWND) 
        -> Result<(POINT, SIZE)> {
        let converter = WindowCoordinateConverter::from_hwnd(hwnd)?;
        let adjusted_pos = converter.client_to_window_position(self.position?);
        let adjusted_size = converter.client_to_window_size(self.size?);
        Ok((adjusted_pos, adjusted_size))
    }
}
```

**互換性評価**:
- ✅ 既存の`WindowPos`コンポーネントフィールドは変更なし
- ✅ 既存メソッドへの影響なし
- ✅ オプトイン方式（新メソッドを呼ぶ側で制御）

**複雑度と保守性**:
- ✅ 単一責任原則を維持（座標変換ロジックが独立）
- ✅ テストが容易（モジュール単体でテスト可能）
- ⚠️ ナビゲーションコストが上がる（ファイル数+1）
- ⚠️ インターフェース設計が必要（DPI取得方法の決定）

**Trade-offs**:
- ✅ 座標変換ロジックが再利用可能（将来的にウィンドウ作成時にも利用可能）
- ✅ 関心の分離が明確
- ✅ `apply_window_pos_changes`の肥大化を防ぐ
- ❌ ファイル数が増える
- ❌ 初期実装コストがOption Aより高い

### Option C: Hybrid - Extend WindowPos + Inline Conversion

**戦略**: `WindowPos`コンポーネントに変換メソッドを追加し、`apply_window_pos_changes`内でインライン呼び出し

**変更対象ファイル**:
- `crates/wintf/src/ecs/window.rs` - `WindowPos`に`to_window_coords`メソッド追加（同ファイル内に実装）
- `crates/wintf/src/ecs/graphics/systems.rs` - `apply_window_pos_changes`で新メソッド呼び出し

**実装アプローチ**:
```rust
// window.rs (WindowPosに追加)
impl WindowPos {
    /// クライアント領域座標をウィンドウ全体座標に変換
    pub fn to_window_coords(&self, hwnd: HWND) 
        -> Result<(i32, i32, i32, i32)> {
        // DPI取得（GetDpiForWindow直接呼び出し）
        let dpi = unsafe { GetDpiForWindow(hwnd) };
        
        // style/ex_style取得
        let style = WINDOW_STYLE(get_window_long_ptr(hwnd, GWL_STYLE)? as u32);
        let ex_style = WINDOW_EX_STYLE(get_window_long_ptr(hwnd, GWL_EXSTYLE)? as u32);
        
        // AdjustWindowRectExForDpi呼び出し
        let mut rect = RECT { 
            left: 0, top: 0,
            right: self.size?.cx, 
            bottom: self.size?.cy 
        };
        unsafe { AdjustWindowRectExForDpi(&mut rect, style, false, ex_style, dpi)? }
        
        // 位置オフセット計算
        let offset_x = -rect.left;
        let offset_y = -rect.top;
        let window_pos = self.position?;
        
        Ok((
            window_pos.x - offset_x,  // 調整後X座標
            window_pos.y - offset_y,  // 調整後Y座標
            rect.right - rect.left,   // 調整後幅
            rect.bottom - rect.top    // 調整後高さ
        ))
    }
}

// systems.rs (apply_window_pos_changes内で呼び出し)
let (x, y, width, height) = match window_pos.to_window_coords(window_handle.hwnd) {
    Ok(coords) => coords,
    Err(e) => {
        eprintln!("[apply_window_pos_changes] 座標変換失敗: {:?}", e);
        // フォールバック: 元の値を使用
        (position.x, position.y, size.cx, size.cy)
    }
};
```

**互換性評価**:
- ✅ 既存の`WindowPos`フィールドは変更なし
- ✅ 既存メソッドへの影響なし
- ✅ エコーバックメカニズムを破壊しない

**複雑度と保守性**:
- ✅ `WindowPos`に論理的に属するメソッド（結合度が自然）
- ✅ ファイル数は増えない（window.rs内に実装）
- ⚠️ `window.rs`がやや肥大化（現在435行 → 470行程度）
- ✅ `apply_window_pos_changes`はシンプルに保たれる

**Trade-offs**:
- ✅ Option Aより保守性が高い（メソッド単位でテスト可能）
- ✅ Option Bよりファイル数が少ない
- ✅ バランスの取れたアプローチ
- ⚠️ `window.rs`の責務がやや広がる（座標変換ロジックを含むようになる）

## 4. Implementation Complexity & Risk

### Effort Estimation

**Option A: Extend apply_window_pos_changes**
- **Effort**: S (1-2日)
  - 理由: 1ファイル、1関数内の変更、既存パターン流用
  
**Option B: Create New Conversion Module**
- **Effort**: M (3-4日)
  - 理由: 新モジュール設計、インターフェース定義、統合テスト

**Option C: Hybrid - Extend WindowPos + Inline**
- **Effort**: S-M (2-3日)
  - 理由: 1ファイル追加変更、メソッド実装、統合調整

### Risk Assessment

**Option A**
- **Risk**: Low
  - 理由: 既存の`effective_window_size`パターンを流用、変更箇所が局所的、既知のWin32 API

**Option B**
- **Risk**: Low-Medium
  - 理由: 新モジュール設計によるインターフェース不整合リスク、DPI取得方法の選定が必要

**Option C**
- **Risk**: Low
  - 理由: `WindowPos`に自然に統合、既存パターン踏襲、変更範囲が明確

### Key Unknowns

**DPI値取得方法の決定** (すべてのオプションに共通):
- Option 1: `GetDpiForWindow(hwnd)` - 最もシンプル、HWNDから直接取得
- Option 2: `DpiTransform`コンポーネント - ECS統合だが現状自動付与されていない
- Option 3: `WinState` trait - 非ECSパターン、既存実装との整合性

**推奨**: Option 1 (`GetDpiForWindow`) - 最小限の変更、既存コードベースへの影響が少ない

## 5. Recommendations for Design Phase

### Preferred Approach

**推奨**: **Option C (Hybrid - Extend WindowPos + Inline Conversion)**

**理由**:
1. **バランスの取れた保守性**: メソッド単位でテスト可能、ファイル数増加なし
2. **既存パターンとの整合性**: `WindowPos::set_window_pos`と同様のパターン（HWNDを引数に取るメソッド）
3. **低リスク**: 変更範囲が明確、既存の`effective_window_size`実装を参考にできる
4. **適度な結合度**: `WindowPos`に座標変換ロジックが含まれることは論理的に妥当

### Key Design Decisions

1. **DPI取得方法**: `GetDpiForWindow(hwnd)` Win32 APIを直接呼び出す
   - 理由: 最小限の変更、既存コードベースへの影響が少ない、DPI対応を保証

2. **エラーハンドリング戦略**: 変換失敗時は元の座標・サイズでフォールバック、`eprintln!`でログ出力
   - 理由: Requirement 3の要件を満たす、既存の堅牢性パターンを踏襲

3. **エコーバック値の記録**: 変換**後**の座標・サイズを`last_sent_position`/`last_sent_size`に記録
   - 理由: `WM_WINDOWPOSCHANGED`で受信する値は変換後の値なので、エコーバック判定を正しく機能させる

### Research Items to Carry Forward

1. **GetDpiForWindow APIの動作検証**: ウィンドウ作成直後（HWND取得直後）にDPI値が正しく取得できるか確認
2. **タイトルバーなしウィンドウの挙動**: `WS_POPUP`スタイルなど、タイトルバーがない場合の座標変換が正しく機能するか検証
3. **マルチモニター環境でのDPI変化**: 異なるDPIを持つモニター間でウィンドウを移動した際の挙動確認

### Implementation Order

1. **Phase 1**: `WindowPos::to_window_coords`メソッド実装（テスト駆動開発推奨）
2. **Phase 2**: `apply_window_pos_changes`に変換処理統合、エラーハンドリング実装
3. **Phase 3**: `taffy_flex_demo`で動作検証、エッジケーステスト（CW_USEDEFAULT、DPI変化など）

## 6. Conclusion

本機能は**低リスク・短期間**で実装可能。既存の`effective_window_size`実装パターンを流用し、`WindowPos`コンポーネントに座標変換メソッドを追加するハイブリッドアプローチが最適。DPI取得は`GetDpiForWindow` APIを直接呼び出し、エコーバックメカニズムを破壊しないよう変換後の値を記録する設計とする。
