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
1. wintfは水平・垂直DPI値を保持する`dpi_x: u16`および`dpi_y: u16`フィールドを持つ`DPI`コンポーネントを定義する
2. `DPI`コンポーネントはWindowエンティティにのみ付与されるため、`SparseSet`ストレージ戦略を使用する（少数のエンティティにのみ付与されるため効率的）
3. `DPI`コンポーネントは`from_dpi(x_dpi: u16, y_dpi: u16)`コンストラクタメソッドを提供する
4. `DPI`コンポーネントはWindowsメッセージパラメータを解析する`from_WM_DPICHANGED(wparam: WPARAM, lparam: LPARAM)`コンストラクタメソッドを提供する
5. `DPI`コンポーネントはスケールファクター（DPI / 96.0）を`f32`で返す`scale_x()`および`scale_y()`メソッドを提供する

---

### Requirement 2: DPIコンポーネント自動付与
**Objective:** As a 開発者, I want DPIコンポーネントがWindowエンティティに自動付与される, so that 手動でDPIコンポーネントを追加する必要がなくなる

#### Acceptance Criteria
1. When `WindowHandle`コンポーネントがエンティティに追加された時, wintfは同じエンティティに`DPI`コンポーネントを自動挿入する
2. When `WindowHandle`が追加された時, wintfは`GetDpiForWindow` Win32 APIから取得した現在のDPI値で`DPI`を初期化する
3. If `GetDpiForWindow`が失敗した場合, wintfはデフォルトDPI値96（標準DPI）を使用する

---

### Requirement 3: DPI変更イベント処理
**Objective:** As a 開発者, I want WM_DPICHANGEDメッセージでDPIコンポーネントが更新される, so that モニター移動やDPI設定変更に対応できる

#### Acceptance Criteria
1. When `ecs_wndproc`で`WM_DPICHANGED`メッセージを受信した時, wintfはwparamから新しいDPI値を取得して`DPI`コンポーネントを更新する
2. When `DPI`コンポーネントを更新する時, wintfは値が実際に変更された場合のみ変更検知をトリガーするため`set_if_neq`を使用する
3. wintfはwparamの下位ワードから`dpi_x`を、上位ワードから`dpi_y`を抽出する

---

### Requirement 4: DPIからArrangement.scaleへの伝搬
**Objective:** As a 開発者, I want DPI変更がArrangement.scaleに反映される, so that 子孫エンティティのレイアウトがDPIスケールを考慮するようになる

#### Design Decision
`Arrangement`コンポーネントを変更するシステムは`update_arrangements_system`の1つのみとする。変更トリガーは以下の2条件のOR:
- `Changed<TaffyComputedLayout>`: `offset`と`size`フィールドを更新
- `Changed<DPI>`: `scale`フィールドを更新

これにより責務が単一システムに集約され、Arrangementの整合性が保証される。

#### Acceptance Criteria
1. When Windowエンティティで`DPI`コンポーネントが変更された時, `update_arrangements_system`は`Arrangement.scale`をDPIスケールファクターに更新する
2. `update_arrangements_system`はいずれかの変更に応答するため、クエリフィルターとして`Or<(Changed<TaffyComputedLayout>, Changed<DPI>)>`を使用する
3. `update_arrangements_system`は`DPI`コンポーネントにアクセスするため、入力パラメータとして`Option<&DPI>`を受け取る
4. When `Changed<TaffyComputedLayout>`によりシステムが実行された時, wintfは`Arrangement.offset`と`Arrangement.size`を更新する
5. When `Changed<DPI>`によりシステムが実行された時, wintfは`Arrangement.scale`のみを更新する（`offset`/`size`は変更しない）
6. When エンティティに`DPI`が存在する時, wintfは`Arrangement.scale.x`を`dpi.scale_x()`に、`Arrangement.scale.y`を`dpi.scale_y()`に設定する
7. When `DPI`が存在しない時, wintfはデフォルトスケール(1.0, 1.0)を使用する

---

### Requirement 5: 既存DpiTransformコンポーネントの整理
**Objective:** As a 開発者, I want 未使用のDpiTransformコンポーネントを削除, so that コードベースが整理され保守性が向上する

#### Acceptance Criteria
1. wintfはコードベースで未使用であることが確認された場合、`window.rs`から`DpiTransform`構造体定義を削除する
2. wintfは`DpiTransform`から有用な実装コード（例: `from_WM_DPICHANGED`メソッド、スケール計算ロジック）を新しい`DPI`コンポーネントに移行する
3. If `DpiTransform`がコードで参照されている場合, wintfはそれらの参照を新しい`DPI`コンポーネントを使用するよう更新する

---

### Requirement 6: 後方互換性
**Objective:** As a 開発者, I want 既存のレイアウトシステムが正常に動作し続ける, so that 既存のアプリケーションが壊れない

#### Acceptance Criteria
1. While エンティティに`DPI`コンポーネントが存在しない間, wintfは`Arrangement.scale`にデフォルトスケール(1.0, 1.0)を使用する
2. wintfは`DPI`コンポーネントを持たないエンティティ（非Windowエンティティ）の動作を変更しない
3. 既存の`propagate_global_arrangements`システムはエンティティツリー全体にスケール値を正しく伝搬する

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

### テスト検証方法

`taffy_flex_demo`サンプルを使用してDPI伝搬をテストできる。

**開発環境モニター構成:**
- 右モニター（プライマリ）: x=0〜3840, DPI=120 (125%スケール)
- 左モニター: x=-2880〜0, DPI=192 (200%スケール)

**テストシナリオ:**
1. ウィンドウを右モニター`(100, 100)`で作成（初期DPI=120）
2. 5秒後に左モニター`(-1500, 500)`へ移動
3. `WM_DPICHANGED`発火を確認: `dpi=(192, 192), scale=(2.00, 2.00)`
4. `Arrangement.scale`が`(2.0, 2.0)`に更新されることを確認

**検証済みログ出力:**
```
[WM_DPICHANGED] Entity 3v0: dpi=(192, 192), scale=(2.00, 2.00)
```

