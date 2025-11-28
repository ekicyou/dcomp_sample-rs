# Requirements Document

## Project Description (Input)
WindowエンティティのDPIをArrangement.scaleを経由してエンティティツリーに伝搬する仕組みを作る。
1. DPIコンポーネント（dpi_x: u16/dpi_y: u16）を作る。このコンポーネントはWindowエンティティにしか作成されないのでコンポーネントメモリ戦略に注意。
2. DPIコンポーネントはWindowHandleコンポーネント作成時にWindowエンティティに作成される。
3. DPIコンポーネントは`fn ecs_wndproc`内の`WM_DPICHANGED`メッセージで更新される。
4.`fn update_arrangements_system`に`Changed<DPI>`のOR条件を追加し、`Option<&DPI>`を入力に追加。DPIをArrangement.scaleに伝搬する。
5. DpiTransformコンポーネントについて、未使用であることが確認されたら削除する。ただし、実装コードで有用なものは新DPIコンポーネントに流用する。

## Introduction

本仕様は、WindowsのDPI（Dots Per Inch）情報をECSコンポーネントとして管理し、レイアウトシステムの`Arrangement.scale`フィールドを通じてエンティティツリー全体に伝搬する仕組みを定義します。これにより、高DPIモニター環境やマルチモニター環境でのスケーリングが正しく機能するようになります。

---

## Requirements

### Requirement 1: DPIコンポーネント定義
**Objective:** As a 開発者, I want DPI情報を保持するECSコンポーネント, so that WindowのDPI値をECSシステムで参照・変更検知できる

#### Acceptance Criteria
1. The wintf shall define a `DPI` component with `dpi_x: u16` and `dpi_y: u16` fields to store horizontal and vertical DPI values
2. The `DPI` component shall use `SparseSet` storage strategy because it is only attached to Window entities (少数のエンティティにのみ付与されるため)
3. The `DPI` component shall provide a `from_dpi(x_dpi: u16, y_dpi: u16)` constructor method
4. The `DPI` component shall provide a `from_WM_DPICHANGED(wparam: WPARAM, lparam: LPARAM)` constructor method to parse Windows message parameters
5. The `DPI` component shall provide `scale_x()` and `scale_y()` methods that return `f32` scale factors (DPI / 96.0)

---

### Requirement 2: DPIコンポーネント自動付与
**Objective:** As a 開発者, I want DPIコンポーネントがWindowエンティティに自動付与される, so that 手動でDPIコンポーネントを追加する必要がなくなる

#### Acceptance Criteria
1. When `WindowHandle` component is added to an entity, the wintf shall automatically insert a `DPI` component to the same entity
2. When `WindowHandle` is added, the wintf shall initialize `DPI` with the current DPI value obtained from `GetDpiForWindow` Win32 API
3. If `GetDpiForWindow` fails, the wintf shall use the default DPI value of 96 (standard DPI)

---

### Requirement 3: DPI変更イベント処理
**Objective:** As a 開発者, I want WM_DPICHANGEDメッセージでDPIコンポーネントが更新される, so that モニター移動やDPI設定変更に対応できる

#### Acceptance Criteria
1. When `WM_DPICHANGED` message is received in `ecs_wndproc`, the wintf shall update the `DPI` component with the new DPI values from wparam
2. When updating `DPI` component, the wintf shall use `set_if_neq` to trigger change detection only when the value actually changes
3. The wintf shall extract `dpi_x` from the low word of wparam and `dpi_y` from the high word of wparam

---

### Requirement 4: DPIからArrangement.scaleへの伝搬
**Objective:** As a 開発者, I want DPI変更がArrangement.scaleに反映される, so that 子孫エンティティのレイアウトがDPIスケールを考慮するようになる

#### Acceptance Criteria
1. When `DPI` component changes on a Window entity, the `update_arrangements_system` shall update `Arrangement.scale` to reflect the DPI scale factors
2. The `update_arrangements_system` shall include `Changed<DPI>` in its query filter (OR condition with existing `Changed<TaffyComputedLayout>`)
3. The `update_arrangements_system` shall accept `Option<&DPI>` as an input parameter to access the DPI component
4. When `DPI` is present on an entity, the wintf shall set `Arrangement.scale.x` to `dpi.scale_x()` and `Arrangement.scale.y` to `dpi.scale_y()`
5. When `DPI` is not present, the wintf shall use default scale of (1.0, 1.0)

---

### Requirement 5: 既存DpiTransformコンポーネントの整理
**Objective:** As a 開発者, I want 未使用のDpiTransformコンポーネントを削除, so that コードベースが整理され保守性が向上する

#### Acceptance Criteria
1. The wintf shall remove the `DpiTransform` struct definition from `window.rs` if it is confirmed unused in the codebase
2. The wintf shall migrate any useful implementation code from `DpiTransform` to the new `DPI` component (e.g., `from_WM_DPICHANGED` method, scale calculation logic)
3. If `DpiTransform` is referenced by any code, the wintf shall update those references to use the new `DPI` component

---

### Requirement 6: 後方互換性
**Objective:** As a 開発者, I want 既存のレイアウトシステムが正常に動作し続ける, so that 既存のアプリケーションが壊れない

#### Acceptance Criteria
1. While `DPI` component is not present on an entity, the wintf shall use default scale (1.0, 1.0) for `Arrangement.scale`
2. The wintf shall not change the behavior of entities without `DPI` component (non-Window entities)
3. The existing `propagate_global_arrangements` system shall correctly propagate scale values through the entity tree

---

## Out of Scope

- MonitorエンティティへのDPI伝搬（Windowエンティティのみが対象）
- DPI変更時のウィンドウサイズ自動調整（`WM_DPICHANGED`のlparamで提供される推奨サイズへの対応）
- Per-Monitor DPI Awareness v2の完全対応（基本的なDPI伝搬のみ）

---

## Technical Notes

### 既存コードの参照
- `DpiTransform`（`window.rs:259-290`）: `from_WM_DPICHANGED`メソッドとスケール計算ロジックを流用可能
- `on_window_handle_add`（`window.rs:212-227`）: DPIコンポーネント挿入のフックポイント
- `ecs_wndproc`（`window_proc.rs:28-`）: WM_DPICHANGED処理の追加箇所
- `update_arrangements_system`（`layout/systems.rs:252-`）: DPI→scale伝搬の実装箇所

### メモリ戦略
- `DPI`コンポーネントは`SparseSet`ストレージを使用（Windowエンティティのみに付与されるため、Table戦略より効率的）

