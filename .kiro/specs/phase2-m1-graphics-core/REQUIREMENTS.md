# Requirements: Phase 2 Milestone 1 - GraphicsCore初期化

**Feature ID**: `phase2-m1-graphics-core`  
**Phase**: Phase 1 - Requirements  
**Updated**: 2025-11-14

---

## 📋 Requirements Overview

### 目的
Phase 2「はじめての描画」の基盤となるグローバルグラフィックスリソース（GraphicsCore）を初期化する。すべての描画処理（ウィンドウ、Surfaceなど）はこのGraphicsCoreから派生する。

### スコープ
- DirectComposition、Direct2D、Direct3D11、DirectWriteのファクトリを一括初期化
- `ProcessSingleton`パターンで管理（プロセス全体で1つ）
- アプリケーション起動時に自動初期化
- エラー処理とログ出力

---

## 📊 Functional Requirements

### FR-1: GraphicsCore構造体の定義
**優先度**: 必須  
**説明**: グローバルグラフィックスリソースを保持する構造体を定義する

**詳細**:
- **既存の`GraphicsDevices`構造体を改造して`GraphicsCore`に名前を変更**
- 既存のフィールド（`ecs/graphics.rs`から）:
  - `d3d: ID3D11Device` - Direct3D11デバイス
  - `dxgi: IDXGIDevice4` - DXGIデバイス（D3D11から取得）
  - `d2d: ID2D1Device` - Direct2Dデバイス
  - `desktop: IDCompositionDesktopDevice` - DirectCompositionデスクトップデバイス
  - `dcomp: IDCompositionDevice3` - DirectCompositionデバイス
- 追加が必要なフィールド:
  - `d2d_factory: ID2D1Factory` - Direct2Dファクトリ（マルチスレッド対応）
  - `dwrite_factory: IDWriteFactory2` - DirectWriteファクトリ
- `Send` + `Sync`を実装（スレッド間で共有可能）- **既存実装あり**
- `Debug`を実装（デバッグ時に内容確認可能）- **既存実装あり**
- `Resource`として定義（ECSリソース）- **既存実装あり**

**改造対象**:
- `ecs/graphics.rs`の`GraphicsDevices`構造体 → `GraphicsCore`に名前変更
- 新しいフィールド（`d2d_factory`, `dwrite_factory`）を追加

**受け入れ基準**:
- ✅ `GraphicsDevices`が`GraphicsCore`に名前変更されている
- ✅ 既存フィールド（`d3d`, `dxgi`, `d2d`, `desktop`, `dcomp`）が保持されている
- ✅ 新規フィールド（`d2d_factory`, `dwrite_factory`）が追加されている
- ✅ `Send` + `Sync` + `Debug` + `Resource`トレイトが実装されている

---

### FR-2: D3D11DeviceとDXGIDeviceの作成
**優先度**: 必須  
**説明**: Direct3D11デバイスとDXGIデバイスを作成する

**詳細**:
- **既存の`create_device_3d()`関数を活用** (`ecs/graphics.rs`)
- `d3d11_create_device()`を使用してデバイスを作成
- ハードウェアアクセラレーション優先（`D3D_DRIVER_TYPE_HARDWARE`）
- フラグ: `D3D11_CREATE_DEVICE_BGRA_SUPPORT`（既存実装）
- **追加**: デバッグモードでは`D3D11_CREATE_DEVICE_DEBUG`フラグも有効化
- `ID3D11Device`から`IDXGIDevice4`を取得（`cast()`）- **既存実装あり**
- エラー時は詳細なログを出力

**改造対象**:
- `create_device_3d()`関数のフラグにデバッグビルド時の条件分岐を追加

**受け入れ基準**:
- ✅ `ID3D11Device`の作成に成功
- ✅ `IDXGIDevice4`の取得に成功
- ✅ ハードウェアアクセラレーションが有効
- ✅ デバッグビルド時にデバッグレイヤーが有効
- ✅ 作成成功をログに出力

---

### FR-3: D2DFactoryの作成
**優先度**: 必須  
**説明**: Direct2Dファクトリを作成する

**詳細**:
- **新規追加**: `D2D1CreateFactory()`を使用してファクトリを作成
- ファクトリタイプ: `D2D1_FACTORY_TYPE_MULTI_THREADED`（マルチスレッド対応）
- `ID2D1Factory`として作成
- `GraphicsCore::new()`内で作成し、フィールドに保存
- エラー時は詳細なログを出力

**改造対象**:
- `GraphicsCore::new()`に`D2D1CreateFactory()`の呼び出しを追加
- 作成したファクトリを構造体フィールドに保存

**受け入れ基準**:
- ✅ `ID2D1Factory`の作成に成功
- ✅ マルチスレッドモードで作成
- ✅ 作成成功をログに出力

---

### FR-4: D2DDeviceの作成
**優先度**: 必須  
**説明**: DXGIDeviceからD2DDeviceを作成する

**詳細**:
- **既存実装を活用**: `d2d_create_device(&dxgi)`を使用してD2DDeviceを作成
- `IDXGIDevice4`を引数として渡す
- `ID2D1Device`として作成
- エラー時は詳細なログを出力

**改造対象**:
- なし（既存実装をそのまま使用）

**受け入れ基準**:
- ✅ `ID2D1Device`の作成に成功
- ✅ DXGIDeviceとの連携が正常に動作
- ✅ 作成成功をログに出力

---

### FR-5: DWriteFactoryの作成
**優先度**: 必須  
**説明**: DirectWriteファクトリを作成する

**詳細**:
- **新規追加**: `dwrite_create_factory()`を使用してファクトリを作成
- ファクトリタイプ: `DWRITE_FACTORY_TYPE_SHARED`（共有モード）
- `IDWriteFactory2`として作成
- `GraphicsCore::new()`内で作成し、フィールドに保存
- エラー時は詳細なログを出力

**改造対象**:
- `GraphicsCore::new()`に`dwrite_create_factory()`の呼び出しを追加
- 作成したファクトリを構造体フィールドに保存

**受け入れ基準**:
- ✅ `IDWriteFactory2`の作成に成功
- ✅ 共有モードで作成
- ✅ 作成成功をログに出力

---

### FR-6: DCompDeviceの作成
**優先度**: 必須  
**説明**: DirectCompositionデバイスを作成する

**詳細**:
- **既存実装を活用**: `dcomp_create_desktop_device(&d2d)`を使用してデバイスを作成
- D2DDeviceを引数として渡す
- `IDCompositionDesktopDevice`として作成
- `IDCompositionDevice3`へのキャスト（`cast()`）- **既存実装あり**
- エラー時は詳細なログを出力

**改造対象**:
- なし（既存実装をそのまま使用）

**受け入れ基準**:
- ✅ `IDCompositionDesktopDevice`の作成に成功
- ✅ `IDCompositionDevice3`へのキャストに成功
- ✅ D2DDeviceとの連携が正常に動作
- ✅ 作成成功をログに出力

---

### FR-7: ECSリソースとしての管理
**優先度**: 必須  
**説明**: GraphicsCoreをECSリソースとして管理する

**詳細**:
- **既存の`Resource`実装を維持** (`#[derive(Resource)]`)
- **ProcessSingletonパターンは採用しない**
- ECSの`Commands::insert_resource()`でリソースとして登録
- ECSの`Res<GraphicsCore>`でシステムからアクセス
- スレッドセーフ（`Send` + `Sync`）- **既存実装あり**

**変更点**:
- 当初の設計では`OnceLock`を使用したProcessSingletonを想定していたが、既存実装がECSリソースパターンを採用しているため、そちらに統一
- `GraphicsCore::get_or_init()`ではなく、ECSの`Res<GraphicsCore>`でアクセス

**受け入れ基準**:
- ✅ `Resource`トレイトが実装されている
- ✅ `Send` + `Sync`が実装されている
- ✅ ECSリソースとして登録可能
- ✅ システムから`Res<GraphicsCore>`でアクセス可能

---

### FR-8: 初期化システムの実装
**優先度**: 必須  
**説明**: ECSスケジュール上でGraphicsCoreを初期化するシステムを実装する

**詳細**:
- **既存の`ensure_graphics_devices`システムを改造** (`ecs/graphics.rs`)
- システム名: `ensure_graphics_devices` → `ensure_graphics_core`に名前変更
- スケジュール: `UISetup`（メインスレッド固定）
- 実行タイミング: アプリケーション起動時の最初のフレーム
- `GraphicsCore::new()`を呼び出してリソースを作成
- `Commands::insert_resource(graphics)`でECSに登録
- 初期化完了をログに出力（日本語化）
- 既に`Res<GraphicsCore>`が存在する場合は何もしない（冪等性を保証）- **既存実装あり**

**改造対象**:
- システム名: `ensure_graphics_devices` → `ensure_graphics_core`
- 引数の型: `Option<Res<GraphicsDevices>>` → `Option<Res<GraphicsCore>>`
- ログメッセージを日本語化

**受け入れ基準**:
- ✅ `ensure_graphics_core`システムが実装されている
- ✅ `UISetup`スケジュールに登録されている
- ✅ 初期化が1回だけ実行される
- ✅ 初期化完了のログが日本語で出力される

---

## 🔧 Non-Functional Requirements

### NFR-1: パフォーマンス
**説明**: 初期化処理は高速でなければならない

**詳細**:
- 初期化時間: 100ms以内
- アプリケーション起動時に遅延を感じさせない
- 初期化後のアクセスはオーバーヘッドなし（静的参照）

**受け入れ基準**:
- ✅ 初期化が100ms以内に完了
- ✅ 初期化後のアクセスにロックが不要

---

### NFR-2: エラーハンドリング
**説明**: 初期化失敗時は適切にエラーを報告する

**詳細**:
- 各ファクトリ作成時の`Result`を適切に処理
- エラー発生時は`panic!`でアプリケーションを終了
- エラーメッセージにはどの段階で失敗したかを含める
- エラーメッセージは日本語で記述

**受け入れ基準**:
- ✅ エラー発生時に適切なメッセージが出力される
- ✅ エラーメッセージが日本語で記述されている
- ✅ どの段階で失敗したかが明確

---

### NFR-3: ログ出力
**説明**: 初期化プロセスをログで追跡可能にする

**詳細**:
- 各ファクトリ作成の開始と完了をログに出力
- `eprintln!`マクロを使用
- ログフォーマット: `[GraphicsCore] <メッセージ>`
- ログは日本語で記述

**受け入れ基準**:
- ✅ 初期化開始のログが出力される
- ✅ 各ファクトリ作成のログが出力される
- ✅ 初期化完了のログが出力される
- ✅ ログが日本語で記述されている

---

### NFR-4: スレッド安全性
**説明**: GraphicsCoreは複数スレッドから安全にアクセス可能

**詳細**:
- `Send` + `Sync`トレイトを実装
- `OnceLock`による初期化の競合保護
- COM APIの制約を考慮（メインスレッドで初期化）

**受け入れ基準**:
- ✅ `Send` + `Sync`が実装されている
- ✅ 複数スレッドからの同時アクセスが安全
- ✅ 初期化はメインスレッドで実行される

---

## 📐 Technical Constraints

### TC-1: COM API初期化順序
**説明**: COM APIの初期化には特定の順序が必要

**詳細**:
```
GraphicsCore::new()の実行順序:
1. D3D11Device (独立) - create_device_3d()
2. IDXGIDevice4 (D3D11Deviceからcast)
3. D2DFactory (独立) - D2D1CreateFactory() ← 新規追加
4. D2DDevice (IDXGIDevice → D2DDevice) - d2d_create_device()
5. DWriteFactory (独立) - dwrite_create_factory() ← 新規追加
6. IDCompositionDesktopDevice (D2DDevice → DCompDevice) - dcomp_create_desktop_device()
7. IDCompositionDevice3 (DesktopDeviceからcast)
```

**制約**:
- D2DDeviceはIDXGIDevice4から作成する必要がある（既存実装済み）
- DCompDeviceはD2DDeviceから作成する必要がある（既存実装済み）
- D2DFactoryとDWriteFactoryは独立して作成可能（新規追加）
- 初期化順序を守る必要がある

---

### TC-2: COM API使用環境
**説明**: COMオブジェクトの作成はメインスレッドで行う

**詳細**:
- `UISetup`スケジュールで実行（SingleThreaded executor）
- COM初期化は不要（Win32 APIが暗黙的に処理）
- デバイスロスト時の再作成は将来のマイルストーンで対応

---

### TC-3: 既存COMラッパーAPIの活用
**説明**: 既存の`com/`モジュールのラッパー関数を使用する

**詳細**:
- `com/d3d11.rs` - `d3d11_create_device()`
- `com/d2d/mod.rs` - `d2d_create_device()`
- `com/dwrite.rs` - `dwrite_create_factory()`
- `com/dcomp.rs` - `dcomp_create_desktop_device()`
- 新しいラッパー関数の追加は不要

---

## 🔄 Integration Requirements

### IR-1: ECSスケジュールとの統合
**説明**: 既存のECSスケジュールシステムに統合する

**詳細**:
- `crates/wintf/src/ecs/world.rs`の`EcsWorld::new()`で管理
- `UISetup`スケジュールに`ensure_graphics_core`システムを登録
- デフォルトシステムとして自動登録（`EcsWorld::new()`内で登録）
- アプリケーション側での明示的な初期化呼び出しは不要

**改造対象**:
- `ecs/world.rs`の`EcsWorld::new()`内のデフォルトシステム登録セクション
- 現在: `create_windows`のみ登録されている
- 追加: `schedules.add_systems(UISetup, ensure_graphics_core);`
- **重要**: `ensure_graphics_core`は`create_windows`より前に実行する必要がある（`.before(create_windows)`）

**実装例**:
```rust
// デフォルトシステムの登録
{
    let mut schedules = world.resource_mut::<Schedules>();
    schedules.add_systems(
        UISetup, 
        ensure_graphics_core.before(create_windows)
    );
    schedules.add_systems(UISetup, create_windows);
}
```

---

### IR-2: 既存の`GraphicsDevices`との互換性
**説明**: 既存のコードで`GraphicsDevices`を使用している箇所を更新

**詳細**:
- `ecs/graphics.rs`の構造体名とシステム名を更新
- **確認済み**: 他のモジュールでは`GraphicsDevices`を直接参照していない
- `ecs/mod.rs`で`pub use graphics::*;`されているため、外部からは型名経由でアクセス
- フィールド名は既存のまま維持（`d3d`, `dxgi`, `d2d`, `desktop`, `dcomp`）
- 新しいフィールド（`d2d_factory`, `dwrite_factory`）を追加

**影響範囲**:
- ✅ `ecs/graphics.rs`のみ（直接の変更対象）
- ✅ 外部モジュールからの参照は型名経由のため、`pub use`で自動的に更新される
- ⚠️ サンプルアプリ（`examples/`）で直接使用している可能性は確認が必要（実装時）

---

## 📚 Dependencies

### 前提条件
- Phase 1完了（ウィンドウシステムが動作している）
- 既存のCOMラッパーAPI（`com/`モジュール）が実装済み
- `bevy_ecs`のスケジュールシステムが動作している

### 外部依存
- `windows` crate (0.62.1)
- `bevy_ecs` crate (0.17.2)
- Windows 10/11環境（DirectComposition対応）

---

## ✅ Acceptance Criteria Summary

### 必須要件
1. ✅ `GraphicsDevices`が`GraphicsCore`に名前変更されている
2. ✅ 既存フィールド（`d3d`, `dxgi`, `d2d`, `desktop`, `dcomp`）が保持されている
3. ✅ 新規フィールド（`d2d_factory`, `dwrite_factory`）が追加されている
4. ✅ すべてのファクトリが正常に作成される
5. ✅ ECSリソースとして管理されている
6. ✅ `ensure_graphics_core`システムが実装されている
7. ✅ 初期化が1回だけ実行される
8. ✅ 初期化プロセスがログに出力される（日本語）
9. ✅ エラー時に適切なメッセージが表示される（日本語）
10. ✅ スレッド間で安全にアクセス可能

### 動作確認
1. ✅ アプリケーション起動時にエラーなく初期化完了
2. ✅ ログで各ファクトリの作成が確認できる（日本語）
3. ✅ システムから`Res<GraphicsCore>`でアクセス可能
4. ✅ 複数回呼び出しても初期化は1回だけ
5. ✅ 既存のコードが引き続き動作する（後方互換性）

---

## 📖 Related Documents

- `.kiro/specs/phase2-m1-graphics-core/SPEC.md` - 仕様概要
- `.kiro/specs/brainstorming-next-features/MILESTONES.md` - マイルストーン全体像
- `.kiro/steering/tech.md` - 技術スタック
- `crates/wintf/src/ecs/graphics.rs` - **改造対象の既存実装**
- `crates/wintf/src/ecs/world.rs` - ECSスケジュール管理
- `crates/wintf/src/com/` - COMラッパーAPI

---

## 🎯 Out of Scope (このマイルストーンでは扱わない)

- ウィンドウ単位のグラフィックスリソース（Milestone 2で実装）
- デバイスロスト対応（Phase 2完了後に実装）
- 描画処理（Milestone 3で実装）
- Surface作成（Milestone 3で実装）
- Visual作成（Milestone 2で実装）

---

_Phase 1 (Requirements) completed. Ready for design phase._

**次のステップ**:
```bash
/kiro-spec-design phase2-m1-graphics-core
```
