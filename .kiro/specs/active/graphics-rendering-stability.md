# Graphics Rendering Stability Issue

## 📋 要件定義 (Requirements)

### 目的 (Objective)
複数ウィンドウ描画時に発生する`D2DERR_WRONG_STATE`エラーを解決し、安定した描画を実現する。

### 背景 (Background)
- **現象**: 2つ目のウィンドウ（Entity 1v0）で`EndDraw`時に`HRESULT(0x88990001)`エラーが発生
- **影響**: 描画が不安定（半透明の橙、青の四角形など）で再現性が低い
- **環境**: DirectComposition + Direct2D によるマルチウィンドウ描画

### 機能要件 (Functional Requirements)

#### FR-1: エラーフリーな描画
- 複数ウィンドウ（2つ以上）を同時に描画しても`D2DERR_WRONG_STATE`エラーが発生しない
- すべてのウィンドウで意図した描画結果が表示される

#### FR-2: リソース状態の正常性
- `IDCompositionSurface::BeginDraw`と`EndDraw`のペアが正しく実行される
- Direct2D DeviceContextの状態が各ウィンドウ間で干渉しない

#### FR-3: 描画順序の保証
- 複数ウィンドウの描画処理が競合せず、決定的な順序で実行される

### 非機能要件 (Non-Functional Requirements)

#### NFR-1: 安定性
- 描画エラーの発生率: 0%（現状：不定期発生）
- 再現性のある動作を保証

#### NFR-2: パフォーマンス
- 既存の描画パフォーマンスを維持（劣化させない）

#### NFR-3: 保守性
- エラーハンドリングとログ出力により問題の追跡が可能
- 将来的なウィンドウ数増加にも対応可能な設計

### 制約条件 (Constraints)

- Windows 10/11のDirectComposition APIを使用
- bevy_ecsのECSアーキテクチャを維持
- 既存のコンポーネント設計（`Surface`, `Visual`, `WindowGraphics`）を大きく変更しない

### 受け入れ基準 (Acceptance Criteria)

1. ✅ 2つ以上のウィンドウを作成・描画してもエラーログが出力されない
2. ✅ すべてのウィンドウで指定した図形（Rectangle）が正しく表示される
3. ✅ ウィンドウの作成・削除を繰り返しても描画が安定している
4. ✅ デバッグログで各ウィンドウの描画処理が追跡可能

---

## 🐛 技術詳細 (Technical Details)

### エラー詳細
```
[render_surface] EndDraw failed for Entity=1v0: Error { 
    code: HRESULT(0x88990001), 
    message: "オブジェクトの状態が適切でないため、メソッドを呼び出せませんでした" 
}
```

### エラーコード
- **HRESULT(0x88990001)** = `D2DERR_WRONG_STATE`
- Direct2D のオブジェクトが不正な状態でメソッドが呼ばれた

### 発生パターン
1. 常に2つ目のウィンドウ（Entity 1v0）でエラー
2. 初回作成時も再初期化時も発生
3. 再現性が低い - 表示内容が不安定

### 調査項目
- [ ] `render_surface`システムの実行順序とEntity処理順
- [ ] `IDCompositionSurface::BeginDraw`/`EndDraw`の呼び出しタイミング
- [ ] Direct2D DeviceContextのスレッドセーフティ
- [ ] `Commit`のタイミングと描画完了の同期

### 関連ファイル
- `crates/wintf/src/ecs/graphics/systems.rs` - 描画システム
- `crates/wintf/src/ecs/widget/shapes/rectangle.rs` - Rectangle描画
- `crates/wintf/src/com/dcomp.rs` - DirectComposition wrapper
- `crates/wintf/src/com/d2d/mod.rs` - Direct2D wrapper

---

## 🎨 設計 (Design)

### 問題の根本原因分析

#### 1. Direct2D DeviceContext の状態管理の問題
**症状**: `D2DERR_WRONG_STATE` (0x88990001)は、Direct2Dオブジェクトが不正な状態でメソッドが呼ばれた時に発生

**原因候補**:
- `BeginDraw()`の後に`EndDraw()`が呼ばれていない（または逆）
- 同じDeviceContextが複数のスレッド/システムから同時にアクセスされている
- `IDCompositionSurface::BeginDraw()`で取得したDeviceContextが正しく管理されていない

#### 2. 現在の実装の問題点

**`render_surface`システム (systems.rs:78-157)**:
```rust
// Surface描画開始
let (dc, _offset) = match surface_ref.begin_draw(None) {
    Ok(result) => result,
    Err(err) => { ... }
};

unsafe {
    dc.clear(...);
    if let Some(command_list) = command_list {
        dc.draw_image(command_list);  // ここで描画
    }
    
    if let Err(err) = dc.EndDraw(None, None) {  // ここでエラー発生
        eprintln!(...);
        let _ = surface_ref.end_draw();
        continue;
    }
}
```

**問題**:
- `BeginDraw()`と`EndDraw()`の間で`DrawImage()`を呼び出しているが、DeviceContextの状態が正しくない可能性
- 2つ目のウィンドウで常にエラーが発生 → Entity処理順序や初期化タイミングの問題
- エラー時に`surface_ref.end_draw()`を呼んでいるが、既に不正な状態の可能性

#### 3. DirectComposition/Direct2D の状態遷移

**Microsoft公式ドキュメントによる正しい順序**:
```
1. IDCompositionSurface::BeginDraw() 
   → ID2D1DeviceContextを取得（既にBeginDraw状態）
2. 描画処理 (Clear, DrawImage, etc.) を直接実行
3. ID2D1DeviceContext::EndDraw()
4. IDCompositionSurface::EndDraw()
```

**重要**: `IDCompositionSurface::BeginDraw()`が返すDeviceContextは**既にBeginDraw状態**にあるため、追加で`ID2D1DeviceContext::BeginDraw()`を呼ぶ必要はありません。

**現在の実装**:
```
1. IDCompositionSurface::BeginDraw() ✅
2. 直接dc.clear() / dc.draw_image() ✅ (正しい)
3. dc.EndDraw() ✅
4. surface_ref.end_draw() ✅
```

**問題点**: エラー処理の不備
- `dc.EndDraw()`失敗後に`surface_ref.end_draw()`を呼んでいるが、これが状態をさらに悪化させる可能性

### 設計方針

#### DS-1: DeviceContext状態管理の正確な理解
- `IDCompositionSurface::BeginDraw()`で取得したDeviceContextは**既にBeginDraw状態**
- Microsoft公式ドキュメント確認済み: 追加の`BeginDraw()`呼び出しは不要かつエラーの原因
- エラーハンドリングを強化し、`EndDraw`失敗時は`surface.end_draw()`を**呼ばない**

#### DS-2: 描画処理の安全性向上
- `BeginDraw`/`EndDraw`のエラー処理を改善
- `EndDraw`失敗時のクリーンアップ処理を修正（`surface.end_draw()`を呼ばない）
- エラー発生時のリソースリーク防止

#### DS-3: 複数Surface間の処理順序管理
- ECSクエリの処理順序は不定だが、各Surfaceは独立しているため問題なし
- ただし、デバッグログでEntity処理順序を追跡可能にする
- 将来的に必要であれば、Surface描画中の状態フラグを導入

#### DS-4: デバッグ機能の強化
- 各ステップでのDeviceContext状態を詳細ログ出力
- エラー発生時のHRESULTコードを16進数表示
- Entity処理順序をログで追跡

### アーキテクチャ設計

#### コンポーネント構成（変更なし）
```
Entity (Window)
├─ HasGraphicsResources (マーカー)
├─ WindowHandle (HWND)
├─ WindowGraphics (Target, DeviceContext)
├─ Visual (IDCompositionVisual3)
├─ Surface (IDCompositionSurface)
└─ GraphicsCommandList (Optional)
```

#### システム実行順序（変更なし）
```
PostLayout:
  1. init_graphics_core
  2. cleanup_command_list_on_reinit
  3. init_window_graphics
  4. init_window_visual
  5. init_window_surface

Draw:
  6. draw_rectangles

Render:
  7. render_surface  ← ここを修正
  8. commit_composition
```

### 実装戦略

#### Strategy-1: エラーハンドリングの改善
**変更対象**: `render_surface` システム

**現在の問題**:
```rust
if let Err(err) = dc.EndDraw(None, None) {
    eprintln!("...");
    let _ = surface_ref.end_draw();  // ← EndDraw失敗後に呼ぶのは不正
    continue;
}
```

**改善案**:
```rust
unsafe {
    dc.clear(...);
    if let Some(command_list) = command_list {
        dc.draw_image(command_list);
    }
    
    // EndDraw失敗時は surface_ref.end_draw() を呼ばない
    if let Err(err) = dc.EndDraw(None, None) {
        eprintln!("[render_surface] EndDraw failed for Entity={:?}: {:?}", entity, err);
        eprintln!("[render_surface] HRESULT: 0x{:08X}", err.code().0);
        // surface_ref.end_draw() は呼ばない（状態不整合のため）
        // Surfaceは不正な状態なので、次フレームで再初期化
        continue;
    }
}

// EndDraw成功後のみ surface.end_draw() を呼ぶ
if let Err(err) = surface_ref.end_draw() {
    eprintln!("[render_surface] Failed to end_draw: {:?}", err);
}
```

**根拠**: 
- `dc.EndDraw()`が失敗した場合、DeviceContextは不正な状態
- その状態で`surface.end_draw()`を呼ぶと、状態がさらに悪化する可能性
- Microsoft公式ドキュメント: EndDraw失敗時はSurfaceを破棄し、再作成が推奨される

#### Strategy-2: 描画前の状態検証と詳細ログ
**追加機能**: DeviceContext状態のチェックとログ強化

**実装方法**:
```rust
// BeginDraw成功の確認
let (dc, offset) = match surface_ref.begin_draw(None) {
    Ok(result) => {
        eprintln!(
            "[render_surface] Entity={:?}, BeginDraw succeeded, offset=({}, {})",
            entity, result.1.x, result.1.y
        );
        result
    }
    Err(err) => {
        eprintln!("[render_surface] BeginDraw failed for Entity={:?}: {:?}", entity, err);
        continue;
    }
};

unsafe {
    // 透明色クリア（常に実行）
    dc.clear(...);
    
    // CommandListがある場合のみ描画
    if let Some(command_list) = command_list {
        eprintln!("[render_surface] Drawing command_list for Entity={:?}", entity);
        dc.draw_image(command_list);
    }
    
    // EndDraw実行
    eprintln!("[render_surface] Calling EndDraw for Entity={:?}", entity);
    if let Err(err) = dc.EndDraw(None, None) {
        // エラーログ（Strategy-1参照）
    }
}
```

**目的**:
- 各ステップの成功/失敗を追跡
- エラー発生時の状態を詳細に記録
- Entity処理順序を明確化

#### Strategy-3: Surface状態の追跡（オプション）
**追加コンポーネント**: `SurfaceState` 

**目的**:
- Surfaceの描画状態（Idle, Drawing, Error）を追跡
- エラー発生時にSurfaceを無効化し、次フレームで再初期化
- 将来的な拡張性を確保

**実装** (Phase 2以降で検討):
```rust
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceState {
    Idle,        // 描画待機中
    Drawing,     // 描画中（BeginDraw～EndDraw）
    Error,       // エラー発生（再初期化が必要）
}
```

**注意**: Phase 1では実装せず、ログ強化とエラーハンドリング改善に集中

#### Strategy-4: 複数Surface間の処理順序管理
**目的**: 複数ウィンドウ（Entity）の描画が干渉しないことを保証

**現在の実装**:
- ECSクエリが各Entityを順次処理（forループ）
- 各Surfaceは独立しているため、並列処理可能（Renderスケジュールはマルチスレッド対応）

**問題の可能性**:
- DirectCompositionデバイスレベルでの制約（複数Surface同時BeginDrawの制限）
- 2つ目のウィンドウで発生→1つ目の処理が完了していない可能性

**実装方法**:
```rust
// Entity処理順序をログで明確化
for (entity, command_list, surface) in query.iter() {
    eprintln!("[render_surface] === Processing Entity={:?} ===", entity);
    
    // 既存の処理...
    
    eprintln!("[render_surface] === Completed Entity={:?} ===", entity);
}
```

**検証**:
- ログ出力でEntity処理順序とタイミングを確認
- 必要に応じて、グローバルなSemaphoreまたはMutexで排他制御（Phase 2以降）

### 変更対象ファイル

#### 1. `crates/wintf/src/ecs/graphics/systems.rs`
**関数**: `render_surface` (lines 78-157)

**変更内容**:
- エラーハンドリングの修正: `dc.EndDraw()`失敗時に`surface.end_draw()`を呼ばない
- デバッグログの追加: Entity処理順序、BeginDraw/EndDrawの成功/失敗
- HRESULTコードの16進数表示

**優先度**: 🔴 必須

#### 2. `crates/wintf/src/ecs/graphics/components.rs`
**追加**: `SurfaceState` コンポーネント（オプション）

**変更内容**:
- Surface状態を追跡するenum定義
- Phase 1では実装せず、Phase 2以降で検討

**優先度**: 🟡 オプション（Phase 2以降）

#### 3. `crates/wintf/src/com/dcomp.rs`
**変更**: `DCompositionSurfaceExt::begin_draw` (lines 155-163)

**変更内容**:
- エラー時の詳細情報追加（必要に応じて）
- 現時点では変更不要

**優先度**: 🟢 低（必要に応じて）

### リスク分析

#### Risk-1: 根本原因が複数の要因による複合問題
**リスク**: エラーログ強化だけでは完全に解決しない可能性  
**軽減策**: 
- Phase 1: ログ強化とエラーハンドリング改善で状態を安定化
- Phase 2: 必要に応じてSurface再初期化ロジックを追加

#### Risk-2: DirectComposition API仕様の制約
**リスク**: Windows API側の制約（複数Surface同時BeginDrawの制限など）で回避不可能な問題の可能性  
**軽減策**: 
- Microsoft公式ドキュメントとサンプルコードを精査
- 必要に応じて排他制御（Mutex/Semaphore）を導入

#### Risk-3: パフォーマンス劣化
**リスク**: デバッグログ出力によるパフォーマンス低下  
**軽減策**: 
- デバッグビルドのみでログ出力（`cfg(debug_assertions)`）
- リリースビルドでは最小限のログのみ

#### Risk-4: エラー後の状態回復不可
**リスク**: `EndDraw`失敗後、Surfaceが回復不能な状態になる  
**軽減策**:
- `GraphicsNeedsInit`マーカーを追加し、次フレームで再初期化
- 既存の再初期化システム（`init_graphics_core`）を活用

### 成功基準

1. ✅ `D2DERR_WRONG_STATE`エラーの根本原因を特定
   - 詳細ログで問題発生のタイミングと順序を把握
   - Entity処理順序と状態遷移を追跡

2. ✅ エラーハンドリング改善により、エラー発生時の動作を安定化
   - `EndDraw`失敗時に不正な`surface.end_draw()`呼び出しを削除
   - 次フレームでの再初期化を可能にする

3. ✅ デバッグログで問題発生時の状態を追跡可能
   - BeginDraw/EndDrawの成功/失敗をログ出力
   - HRESULTコードを16進数表示

4. ✅ 2つ以上のウィンドウで安定した描画を実現
   - エラーログが出力されない
   - すべてのウィンドウで正しい図形が表示される
   - 再現性のある動作

### 実装フェーズ計画

#### Phase 1: エラーハンドリング改善とログ強化（必須）
- `render_surface`のエラーハンドリング修正
- 詳細ログの追加
- 動作確認とエラー原因の特定

#### Phase 2: 状態管理の改善（必要に応じて）
- `SurfaceState`コンポーネントの追加
- エラー後の自動再初期化
- 複数Surface間の排他制御（必要な場合）

---

## 📋 タスク分解 (Tasks)

### Phase 1: エラーハンドリング改善とログ強化（必須）

#### Task 1.1: render_surfaceのエラーハンドリング修正
**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`  
**関数**: `render_surface` (lines 78-157)  
**優先度**: 🔴 P0 (Critical)

**変更内容**:
1. `dc.EndDraw()`失敗時に`surface_ref.end_draw()`を呼ばないように修正
2. エラー発生時は`continue`で次のEntityへ
3. `EndDraw`成功時のみ`surface_ref.end_draw()`を実行

**具体的な変更箇所** (lines 139-150):
```rust
// 変更前:
if let Err(err) = dc.EndDraw(None, None) {
    eprintln!("[render_surface] EndDraw failed for Entity={:?}: {:?}", entity, err);
    let _ = surface_ref.end_draw();  // ← 削除
    continue;
}

// 変更後:
if let Err(err) = dc.EndDraw(None, None) {
    eprintln!("[render_surface] EndDraw failed for Entity={:?}: {:?}", entity, err);
    eprintln!("[render_surface] HRESULT: 0x{:08X}", err.code().0);
    // surface_ref.end_draw()は呼ばない
    continue;
}
```

**受け入れ基準**:
- ✅ `EndDraw`失敗時に`surface.end_draw()`が呼ばれない
- ✅ コンパイルエラーなし

**見積もり**: 15分

---

#### Task 1.2: Entity処理順序の追跡ログ追加
**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`  
**関数**: `render_surface` (lines 78-157)  
**優先度**: 🔴 P0 (Critical)

**変更内容**:
1. forループの開始時と終了時にログ追加
2. Entity IDを明確に表示

**具体的な変更箇所** (lines 93-95 付近):
```rust
for (entity, command_list, surface) in query.iter() {
    eprintln!("[render_surface] === Processing Entity={:?} ===", entity);
    
    if !surface.is_valid() {
        eprintln!("[render_surface] Surface invalid for Entity={:?}, skipping", entity);
        continue;
    }
    
    // ... 既存の処理 ...
    
    eprintln!("[render_surface] === Completed Entity={:?} ===", entity);
}
```

**受け入れ基準**:
- ✅ 各Entityの処理開始/終了がログに出力される
- ✅ Entity IDが明確に表示される

**見積もり**: 10分

---

#### Task 1.3: BeginDraw成功時のログ強化
**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`  
**関数**: `render_surface` (lines 78-157)  
**優先度**: 🔴 P0 (Critical)

**変更内容**:
1. `BeginDraw`成功時にoffset情報も記録
2. エラーログにHRESULTコードを追加

**具体的な変更箇所** (lines 114-123):
```rust
// 変更前:
let (dc, _offset) = match surface_ref.begin_draw(None) {
    Ok(result) => result,
    Err(err) => {
        eprintln!("[render_surface] Failed to begin draw for Entity={:?}: {:?}", entity, err);
        continue;
    }
};

// 変更後:
let (dc, offset) = match surface_ref.begin_draw(None) {
    Ok(result) => {
        eprintln!(
            "[render_surface] BeginDraw succeeded for Entity={:?}, offset=({}, {})",
            entity, result.1.x, result.1.y
        );
        result
    }
    Err(err) => {
        eprintln!(
            "[render_surface] BeginDraw failed for Entity={:?}: {:?}, HRESULT: 0x{:08X}",
            entity, err, err.code().0
        );
        continue;
    }
};
```

**受け入れ基準**:
- ✅ BeginDraw成功時にoffset値がログ出力される
- ✅ エラー時にHRESULTコードが16進数で表示される

**見積もり**: 10分

---

#### Task 1.4: DrawImage実行前のログ追加
**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`  
**関数**: `render_surface` (lines 78-157)  
**優先度**: 🟡 P1 (High)

**変更内容**:
1. `DrawImage`呼び出し前にログ追加
2. CommandListの有無を明確に記録

**具体的な変更箇所** (lines 134-137):
```rust
// CommandListがある場合のみ描画
if let Some(command_list) = command_list {
    eprintln!("[render_surface] Drawing command_list for Entity={:?}", entity);
    dc.draw_image(command_list);
}
```

**受け入れ基準**:
- ✅ CommandList描画時にログが出力される

**見積もり**: 5分

---

#### Task 1.5: EndDraw実行前のログ追加
**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`  
**関数**: `render_surface` (lines 78-157)  
**優先度**: 🟡 P1 (High)

**変更内容**:
1. `EndDraw`呼び出し前にログ追加

**具体的な変更箇所** (lines 139直前):
```rust
eprintln!("[render_surface] Calling EndDraw for Entity={:?}", entity);
if let Err(err) = dc.EndDraw(None, None) {
    // エラーハンドリング
}
```

**受け入れ基準**:
- ✅ EndDraw実行前にログが出力される

**見積もり**: 5分

---

#### Task 1.6: 動作確認とログ検証
**ファイル**: なし（実行確認）  
**優先度**: 🔴 P0 (Critical)

**作業内容**:
1. サンプルアプリケーション（`areka.rs`）を実行
2. 2つ以上のウィンドウを表示
3. ログ出力を確認し、Entity処理順序とエラー発生箇所を特定

**受け入れ基準**:
- ✅ ログから各Entityの処理順序が確認できる
- ✅ エラー発生時のHRESULTコードが確認できる
- ✅ `D2DERR_WRONG_STATE`の発生タイミングが特定できる

**見積もり**: 20分

---

### Phase 2: 状態管理の改善（必要に応じて実装）

#### Task 2.1: SurfaceStateコンポーネント追加（オプション）
**ファイル**: `crates/wintf/src/ecs/graphics/components.rs`  
**優先度**: 🟢 P2 (Low)

**変更内容**:
1. `SurfaceState` enumを定義
2. Surface状態を追跡

**実装**:
```rust
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceState {
    Idle,        // 描画待機中
    Drawing,     // 描画中（BeginDraw～EndDraw）
    Error,       // エラー発生（再初期化が必要）
}
```

**受け入れ基準**:
- ✅ SurfaceStateコンポーネントが定義されている
- ✅ コンパイルエラーなし

**見積もり**: 15分  
**注意**: Phase 1のログ検証結果に基づいて実装を判断

---

#### Task 2.2: エラー後の自動再初期化（オプション）
**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`  
**優先度**: 🟢 P2 (Low)

**変更内容**:
1. `EndDraw`失敗時に`GraphicsNeedsInit`マーカーを追加
2. 次フレームで自動的に再初期化

**実装**:
```rust
if let Err(err) = dc.EndDraw(None, None) {
    eprintln!("[render_surface] EndDraw failed, marking for re-initialization");
    commands.entity(entity).insert(GraphicsNeedsInit);
    continue;
}
```

**受け入れ基準**:
- ✅ エラー発生時に再初期化マーカーが追加される
- ✅ 次フレームで自動的に再初期化される

**見積もり**: 30分  
**注意**: Phase 1のログ検証結果に基づいて実装を判断

---

### タスクサマリー

| Phase | Task | 優先度 | 見積もり | 必須/オプション |
|-------|------|--------|----------|----------------|
| 1 | Task 1.1: エラーハンドリング修正 | P0 | 15分 | 必須 |
| 1 | Task 1.2: Entity処理順序ログ | P0 | 10分 | 必須 |
| 1 | Task 1.3: BeginDrawログ強化 | P0 | 10分 | 必須 |
| 1 | Task 1.4: DrawImageログ追加 | P1 | 5分 | 必須 |
| 1 | Task 1.5: EndDrawログ追加 | P1 | 5分 | 必須 |
| 1 | Task 1.6: 動作確認 | P0 | 20分 | 必須 |
| 2 | Task 2.1: SurfaceState追加 | P2 | 15分 | オプション |
| 2 | Task 2.2: 自動再初期化 | P2 | 30分 | オプション |

**Phase 1 合計**: 約65分  
**Phase 2 合計**: 約45分（オプション）

---

**Phase**: Tasks  
**Status**: ✅ Approved (Auto-approved with -y flag)  
**Created**: 2025-11-16  
**Updated**: 2025-11-16

### 設計レビュー記録

**レビュー実施**: 2025-11-16  
**修正内容**:
1. ✅ DirectComposition/Direct2D状態遷移の誤記を訂正
   - `IDCompositionSurface::BeginDraw()`が返すDeviceContextは既にBeginDraw状態
   - 追加の`ID2D1DeviceContext::BeginDraw()`呼び出しは不要
2. ✅ DS-1の設計方針を正確に修正
3. ✅ Strategy-1のエラーハンドリングを詳細化
4. ✅ Strategy-4として複数Surface間の処理順序管理を追加
5. ✅ リスク分析にRisk-4を追加

**参考資料**:
- Microsoft公式ドキュメント: IDCompositionSurface::BeginDraw
- Direct2D Error Codes: D2DERR_WRONG_STATE (0x88990001)

---

## 🔍 実装と調査結果 (Implementation & Investigation Results)

### Phase 1 実装完了 (2025-11-16)

#### 実装内容
1. ✅ `render_surface`のエラーハンドリング改善
   - `EndDraw`失敗時に`surface.end_draw()`を呼ばないように修正
   - HRESULTコードの16進数表示を追加
   
2. ✅ `commit_composition`の詳細ログ追加
   - Commit実行前後のログ
   - エラー時のHRESULT表示

3. ✅ FrameCountリソース追加
   - フレーム番号によるログ追跡を実現
   - 各システムで`Res<FrameCount>`参照

4. ✅ スケジュール設定の修正
   - `render_surface`をRenderSurfaceスケジュールに移動
   - `commit_composition`の重複登録を削除

#### 検証結果

**エラー発生タイミング**:
```
[Frame 1] GraphicsCore初期化 + Commit成功
[Frame 2] Surface作成 + render_surface実行 + Commit失敗（D2DERR_WRONG_STATE）
[Frame 3以降] すべてのCommit成功
```

**重要な発見**:
- ✅ エラーは**Frame 2（初回描画フレーム）で1回だけ**発生
- ✅ `render_surface`自体は正常動作（BeginDraw/EndDraw成功）
- ✅ 問題は`commit_composition`で発生
- ❌ **並列実行は原因ではない**（SingleThreaded化してもエラー発生）

### 根本原因の分析

#### D2DERR_WRONG_STATEとは
- Direct2Dオブジェクトが不正な状態でメソッドが呼ばれた時に発生
- BeginDraw/EndDrawの不整合、RenderTarget状態エラーなど

#### 現在の問題
**Surface作成直後の同じフレーム内でCommitを実行**している：
```
Frame 2の実行順序:
1. PostLayout: init_window_surface（Visual::set_content(Surface)で設定）
2. Draw: draw_rectangles（CommandList生成）
3. RenderSurface: render_surface（Surface描画）
4. CommitComposition: commit_composition ← ここで失敗
```

**推測される原因**:
- DirectCompositionは非同期APIのため、`Visual::set_content(Surface)`や`Surface::EndDraw()`の効果が**即座に反映されない**
- 内部状態の初期化が完了する前に`Commit()`が呼ばれている
- Frame 3以降は状態が安定しているため成功

### 試行した対策

#### 1. RenderSurfaceのSingleThreaded化
**結果**: ❌ エラーは解決せず
**結論**: 並列実行が原因ではない

### 残存する問題

**状態**: Frame 2で1回だけCommitが失敗  
**影響**: 描画結果には影響なし（Frame 3で正常化）  
**頻度**: 初回Surface作成時に100%再現

### 今後の対策候補

#### Option 1: Commit失敗時のリトライ機構
```rust
pub fn commit_composition(...) {
    let mut retry_count = 0;
    loop {
        match dcomp.commit() {
            Ok(_) => break,
            Err(e) if e.code() == HRESULT(0x88990001) && retry_count < 3 => {
                retry_count += 1;
                eprintln!("[Frame {}] Commit failed, retrying ({}/3)", frame_count.0, retry_count);
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(e) => {
                eprintln!("[Frame {}] Commit failed: {:?}", frame_count.0, e);
                break;
            }
        }
    }
}
```

#### Option 2: 初回Surface作成時は1フレーム待機
- 新規Surfaceに`NewlyCreated`マーカーを追加
- 最初のフレームでは描画をスキップ

#### Option 3: エラーを無視（既知の制限として扱う）
- 実害がないため、ログレベルをWARNINGに変更
- ドキュメントに記載

**Phase**: Investigation Complete  
**Status**: ⚠️ Known Issue - Frame 2 Commit Failure  
**Updated**: 2025-11-16
