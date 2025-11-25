# Requirements: visual-tree-synchronization

**Feature ID**: `visual-tree-synchronization`  
**Created**: 2025-11-20  
**Updated**: 2025-11-25  
**Status**: Requirements Generated

---

## Introduction

本要件定義は、ECSのウィジットツリー（ChildOf/Children）とDirectCompositionのビジュアルツリー（Visual親子階層）を1:1で同期させる仕組みを定義する。

### 用語定義

| 用語 | 説明 |
|------|------|
| **ウィジットツリー (Widget Tree)** | ECS上の`ChildOf`/`Children`で構成されるEntity階層。アプリケーションの論理的なUI構造を表す |
| **ビジュアルツリー (Visual Tree)** | DirectCompositionの`IDCompositionVisual`親子階層。GPU合成の単位となる |
| **Surface** | `IDCompositionSurface`。実際の描画先となるGPUテクスチャ |

### 背景

`visual-tree-implementation`（アーカイブ済み）では、ウィジットツリー（ChildOf/Children）とArrangement座標変換システムが実装された。現在の実装では**WindowのSurfaceに全子孫を直接描画**する方式を採用している。

本仕様では、**全WidgetがVisualを持つ**WinUI3スタイルのアーキテクチャを採用し、ウィジットツリーとビジュアルツリーを1:1で同期させる。

### 設計原則: 全Visual方式

**ウィジットツリーとビジュアルツリーは1:1対応。**

| Entity種別 | Visual | Surface | 説明 |
|-----------|--------|---------|------|
| Window | ✅ 必須 | ✅ 必須 | ルートVisual |
| Container Widget | ✅ 必須 | ❌ なし | 子を持つだけのVisual（レイアウト用） |
| 描画Widget | ✅ 必須 | ✅ 必須 | 実際に描画するVisual |

### 設計判断の根拠

**採用理由:**
- **シンプルさ**: Widget = Visual の1:1対応で理解しやすい
- **Z-order保証**: 昇格・連鎖昇格の複雑なロジックが不要
- **アニメーション対応**: 全要素がVisualなので制約なし
- **デバッグ容易**: Widget構造 = Visual構造

**対象規模:**
- 「伺か」デスクトップマスコット用途
- 想定Widget数: 50〜200程度
- この規模では全Visual方式のオーバーヘッドは無視できる

### 現状分析

**既存実装**:
- ✅ ウィジットツリー管理（ChildOf/Children）: bevy_ecs::hierarchyで実装済み
- ✅ Arrangement座標変換システム: GlobalArrangement伝播実装済み
- ✅ Window EntityのみVisual+Surfaceを持つ
- ✅ `AddVisual` APIラッパー: `com/dcomp.rs`に存在
- ✅ taffyレイアウトエンジン統合: 完了

**未実装**:
- ❌ `RemoveVisual` APIラッパー
- ❌ Widget生成時のVisual自動作成
- ❌ ウィジットツリー変更→ビジュアルツリー同期
- ❌ Surface遅延作成（描画が必要になった時点で作成）

---

## Requirements

### Requirement 1: RemoveVisual APIラッパーの実装

**Objective:** システム開発者として、DirectCompositionのRemoveVisual APIをRustから安全に呼び出したい。これにより、ビジュアルツリーからの子Visualの削除が可能になる。

#### Acceptance Criteria (R1)

1. wintfシステムは、`com/dcomp.rs`の`DCompositionVisualExt`トレイトに`remove_visual`メソッドを追加しなければならない
2. When `remove_visual`が呼び出される時、wintfシステムは`IDCompositionVisual::RemoveVisual`を呼び出して指定されたVisualを親から削除しなければならない
3. wintfシステムは、`remove_all_visuals`メソッドを追加し、`IDCompositionVisual::RemoveAllVisuals`を呼び出して全子Visualを削除できなければならない
4. If存在しないVisualの削除が試みられた場合、wintfシステムはエラーを適切にハンドリングしなければならない

---

### Requirement 2: Visual追加時のVisualGraphics自動作成

**Objective:** システム開発者として、`Visual`コンポーネントが追加された時に対応する`VisualGraphics`（DirectComposition Visual）を自動的に作成したい。これにより、論理VisualとGPUリソースの1:1対応が実現される。

#### コンポーネント設計

| コンポーネント | 層 | 説明 |
|---------------|-----|------|
| `Visual` | 論理層 | 論理的なビジュアル情報（既存） |
| `VisualGraphics` | GPU層 | `IDCompositionVisual3`を保持（既存） |
| `SurfaceGraphics` | GPU層 | `IDCompositionSurface`を保持（既存） |

**設計方針:** 新しいマーカーコンポーネントは追加しない。既存の`Visual`追加をトリガーとして`VisualGraphics`を自動作成する。`SurfaceGraphics`はこの時点では作成せず、描画が必要になった時点で遅延作成する（R5参照）。

#### Acceptance Criteria (R2)

1. When `Visual`コンポーネントが追加される時、wintfシステムは`VisualGraphics`コンポーネントを自動的に追加しなければならない
2. wintfシステムは、`GraphicsCore`から`IDCompositionDevice3::CreateVisual`を呼び出して新しいVisualを作成しなければならない
3. wintfシステムは、`create_visual_graphics`システムで`Added<Visual>`を検知してVisualGraphicsを作成しなければならない
4. wintfシステムは、Window Entityに対しても同様にVisualGraphicsを作成しなければならない（既存のWindowVisual作成と統合）
5. wintfシステムは、`Visual`追加時に`SurfaceGraphics`を作成してはならない（遅延作成はR5で規定）

---

### Requirement 3: Visual追加ヘルパー関数

**Objective:** ウィジット開発者として、ウィジットの`on_add`フックから簡単にVisualを追加したい。これにより、ウィジット実装の定型コードが削減される。

#### Acceptance Criteria (R3)

1. wintfシステムは、`insert_visual`関数を`ecs::graphics`モジュールに公開しなければならない
2. `insert_visual`関数は、`Commands`または`EntityCommands`を受け取り、`Visual::default()`を挿入しなければならない
3. `insert_visual`関数は、ウィジットの`on_add`フックから呼び出し可能でなければならない
4. wintfシステムは、`insert_visual_with`関数を提供し、カスタム`Visual`値を挿入できなければならない

```rust
// 使用例（ウィジットのon_addフック内）
pub fn on_label_add(mut world: DeferredWorld, entity: Entity, _: HookContext) {
    insert_visual(&mut world, entity);
}
```

---

### Requirement 4: 既存ウィジットへのVisual自動追加

**Objective:** システム開発者として、既存のウィジット（Label, Rectangle）にVisualを自動追加したい。これにより、既存ウィジットがビジュアルツリーに参加できる。

#### 対象ウィジット

| ウィジット | 場所 | Visual | Surface |
|-----------|------|--------|--------|
| `Label` | `ecs/widget/text/label.rs` | ✅ 追加 | ✅ 必要（テキスト描画） |
| `Rectangle` | `ecs/widget/shapes/rectangle.rs` | ✅ 追加 | ✅ 必要（矩形描画） |

#### Acceptance Criteria (R4)

1. wintfシステムは、`Label`コンポーネントに`#[component(on_add = on_label_add)]`を追加しなければならない
2. `on_label_add`フックは、`insert_visual`を呼び出してVisualを追加しなければならない
3. wintfシステムは、`Rectangle`コンポーネントに`#[component(on_add = on_rectangle_add)]`を追加しなければならない
4. `on_rectangle_add`フックは、`insert_visual`を呼び出してVisualを追加しなければならない
5. wintfシステムは、Label/Rectangleの描画時に`SurfaceGraphics`が存在しない場合、自動作成しなければならない

---

### Requirement 4a: Labelテキスト測定とBoxStyleへの反映

**Objective:** システム開発者として、Labelのテキストサイズを測定し、その結果をBoxStyleに反映したい。これにより、taffyレイアウトシステムがテキストの固有サイズを考慮できる。

#### 背景: 現状の問題

現在の実装では：
1. ✅ `draw_labels`システムでテキストを測定し`TextLayoutMetrics {width, height}`を生成している
2. ❌ しかし、この測定結果が`BoxStyle`にフィードバックされていない
3. ❌ そのため、taffyは正しいサイズを知らずにレイアウト計算を行っている

#### 本来あるべきフロー

```
1. Label追加/変更
     ↓
2. テキスト測定（DirectWrite CreateTextLayout + GetMetrics）
     ↓
3. BoxStyle.size に測定結果を反映（intrinsic size）
     ↓
4. TaffyStyle構築システムがBoxStyleを変換
     ↓
5. Taffyレイアウト計算（テキストサイズを考慮）
     ↓
6. Arrangement確定
     ↓
7. 描画（確定サイズのSurfaceに描画）
```

#### 設計方針

| 方式 | 説明 | 採用 |
|-----|------|-----|
| A. BoxStyle.sizeを直接設定 | 測定結果をBoxStyle.sizeに設定 | ✅ シンプル |
| B. min_size追加 | BoxStyleにmin_sizeフィールドを追加 | ❌ 過剰設計 |
| C. intrinsic_size追加 | 固有サイズ専用フィールド | ❌ 複雑 |

**採用方式:** A案。Labelのテキストサイズを`BoxStyle.size`に設定する。ユーザーが明示的にBoxStyleを設定している場合はそちらを優先。

#### Acceptance Criteria (R4a)

1. wintfシステムは、`measure_text_size`システムを追加し、`Label`の変更時にテキストサイズを測定しなければならない
2. `measure_text_size`システムは、`IDWriteFactory::CreateTextLayout`と`GetMetrics`を使用してテキストサイズを取得しなければならない
3. wintfシステムは、測定結果を`TextLayoutMetrics`コンポーネントに保存しなければならない（既存機能を活用）
4. wintfシステムは、`sync_label_size_to_box_style`システムを追加し、`TextLayoutMetrics`から`BoxStyle.size`に値を反映しなければならない
5. If EntityがユーザーによるカスタムBoxStyleを持つ場合、wintfシステムはユーザー設定を優先しなければならない
6. wintfシステムは、`measure_text_size`システムを`PreLayout`スケジュールで実行しなければならない（`Layout`スケジュールの`build_taffy_styles_system`より前）
7. wintfシステムは、既存の`draw_labels`システムからテキスト測定ロジックを分離し、`measure_text_size`システムに移動しなければならない
8. `draw_labels`システムは、`TextLayoutMetrics`が既に存在する場合は再測定をスキップし、描画のみを行わなければならない

#### システム実行順序への影響

```text
PreLayout Schedule:
  measure_text_size           ← NEW: テキスト測定（draw_labelsから分離）
  sync_label_size_to_box_style ← NEW: BoxStyleに反映

Layout Schedule:
  build_taffy_styles_system   ← 更新されたBoxStyleを使用
  sync_taffy_tree_system
  compute_taffy_layout_system
  update_arrangements_system

Draw Schedule:
  draw_labels                 ← 測定済みのTextLayoutMetricsを使用して描画のみ
```

**Note:** 既存スケジュール構成では`PreLayout`が`Layout`の直前に実行されるため、新規スケジュールの追加は不要。

#### 測定タイミングの考慮

| イベント | 測定が必要か |
|---------|-------------|
| Label追加 | ✅ 必要 |
| Label.text変更 | ✅ 必要 |
| Label.font_size変更 | ✅ 必要 |
| Label.font_family変更 | ✅ 必要 |
| Label.direction変更 | ✅ 必要 |
| Label.color変更 | ❌ 不要（サイズに影響しない） |

**Note:** `draw_labels`は描画時（レイアウト後）に実行されるため、測定専用の軽量システムを別途用意する。描画はレイアウト確定後に行う。

---

### Requirement 5: SurfaceGraphics遅延作成

**Objective:** システム開発者として、描画内容を持つWidgetにのみSurfaceを作成したい。これにより、レイアウト専用のContainer Widgetはメモリを節約できる。

#### Acceptance Criteria (R5)

1. wintfシステムは、`SurfaceGraphics`が必要な時点で遅延作成しなければならない
2. wintfシステムは、作成したSurfaceを対応するVisualに`SetContent`で設定しなければならない
3. wintfシステムは、描画システムが`VisualGraphics`を持ち`SurfaceGraphics`を持たないEntityに対してSurfaceを作成しなければならない
4. Container Widget（Visualのみ、描画なし）はSurfaceGraphicsを持たなくてよい
5. wintfシステムは、`GlobalArrangement.bounds`が変更された時にSurfaceサイズを再評価しなければならない
6. wintfシステムは、Surfaceサイズが変更された場合、新しいSurfaceを作成して置き換えなければならない（`IDCompositionSurface`はリサイズ不可のため）
7. wintfシステムは、Surfaceサイズを`ceil(GlobalArrangement.bounds.width)`および`ceil(GlobalArrangement.bounds.height)`で計算しなければならない（小数点切り上げ）

---

### Requirement 5a: 描画方式の変更（自己描画方式）

**Objective:** システム開発者として、各Widgetが自分のSurfaceに自分のコマンドリストのみを描画するようにしたい。これにより、全Visual方式に対応した描画が実現される。

#### 背景: 現在の描画方式

現在の`render_surface`システムは、`draw_recursive`関数で**子孫すべてのコマンドリストを展開して描画**している：

```
現在の方式（親Surface集約描画）:
Window.Surface に描画:
  - Window自身のコマンドリスト
  - 子1のコマンドリスト（GlobalArrangementで座標変換）
  - 子1の孫のコマンドリスト
  - 子2のコマンドリスト
  - ...
```

#### 新しい描画方式

全Visual方式では、**各Widgetが自分のコマンドリストのみを自分のSurfaceに描画**する：

```
新しい方式（自己描画方式）:
Window.Surface に描画:
  - Window自身のコマンドリストのみ

Panel.Surface に描画:
  - Panel自身のコマンドリストのみ

Label.Surface に描画:
  - Label自身のコマンドリストのみ
```

**座標変換はDirectComposition（VisualのOffset）が担当**するため、GlobalArrangementによるCPU側座標変換は不要になる。

#### Acceptance Criteria (R5a)

1. wintfシステムは、`render_surface`システムを変更し、各Entityが**自分のコマンドリストのみ**を自分のSurfaceに描画しなければならない
2. wintfシステムは、`draw_recursive`関数を廃止し、子孫への再帰描画を行わないようにしなければならない
3. wintfシステムは、Surfaceサイズを`Arrangement.size × GlobalArrangement累積スケール`で決定しなければならない
4. wintfシステムは、Surface描画時に`GlobalArrangement.transform`の**スケール成分のみ**を`dc.set_transform()`に適用しなければならない
5. wintfシステムは、`GlobalArrangement.transform`の**平行移動成分**を描画に使用してはならない
6. wintfシステムは、VisualのOffset（`SetOffsetX`/`SetOffsetY`）で位置を制御し、GPU側での座標計算を活用しなければならない
7. wintfシステムは、`Arrangement.offset`を`Visual.SetOffsetX`/`SetOffsetY`に直接マッピングしなければならない
8. wintfシステムは、`SurfaceGraphics`を持つEntityのみを描画対象としなければならない

#### 座標系の変更まとめ

| 項目 | 変更前 | 変更後 |
|-----|--------|--------|
| **Surfaceサイズ** | 固定または手動指定 | `Arrangement.size × 累積スケール` |
| **描画時の変換** | `GlobalArrangement.transform`（全成分） | スケール成分のみ |
| **位置決め** | CPU側でMatrix3x2平行移動 | `Visual.SetOffsetX/Y`でGPU側 |
| **GlobalArrangementの用途** | 描画座標変換（全成分） | スケール成分のみ描画、ヒットテストに全成分使用 |

#### ドットパーフェクト描画の実現

```text
コマンドリスト作成時:
  - Arrangement.size でベクター命令を記録
  - offset (0, 0) から描画

Surface描画時:
  - Surfaceサイズ = Arrangement.size × 累積スケール（物理ピクセル）
  - dc.set_transform(累積スケール)  // スケールのみ、平行移動なし
  - dc.DrawImage(CommandList)       // ベクター→ラスタライズ

Visual配置時:
  - Visual.SetOffsetX(Arrangement.offset.x)
  - Visual.SetOffsetY(Arrangement.offset.y)
```

**結果:** コマンドリストは累積スケールでラスタライズされ、ドットパーフェクトな描画が実現される。位置決めはGPU側のVisual階層で行われる。

#### 描画コードの変更イメージ

```rust
// 変更前: 子孫再帰描画
fn draw_recursive(entity, dc, ...) {
    // 自分を描画（GlobalArrangementで座標変換）
    dc.set_transform(&arr.transform);
    dc.DrawImage(command_list, ...);
    
    // 子を再帰描画
    for child in children {
        draw_recursive(child, dc, ...);
    }
}

// 変更後: 自己描画のみ
fn draw_self(entity, dc, ...) {
    // 自分のコマンドリストのみを原点から描画
    dc.set_transform(&Matrix3x2::identity());
    if let Some(cmd_list) = command_list {
        dc.DrawImage(cmd_list, ...);
    }
    // 子への再帰なし - 子は自分のSurfaceに描画する
}
```

---

### Requirement 6: ウィジットツリー変更の検知

**Objective:** システム開発者として、ウィジットツリーの変更（ChildOf追加/変更/削除）を効率的に検知したい。これにより、ビジュアルツリーの同期タイミングを特定できる。

#### Acceptance Criteria (R6)

1. wintfシステムは、`Added<ChildOf>`クエリを使用して新しく親子関係が設定されたEntityを検知しなければならない
2. wintfシステムは、`Changed<ChildOf>`クエリを使用して親が変更されたEntityを検知しなければならない
3. wintfシステムは、`RemovedComponents<ChildOf>`を使用して親子関係が削除されたEntityを検知しなければならない
4. wintfシステムは、`VisualGraphics`を持つEntityのみをビジュアルツリー同期の対象としなければならない

---

### Requirement 7: ビジュアルツリーへの同期

**Objective:** システム開発者として、ウィジットツリーの変更をビジュアルツリーに自動的に反映したい。これにより、両ツリーの1:1対応が保証される。

#### Acceptance Criteria (R7)

1. When VisualGraphicsを持つEntityにChildOfが追加される時、wintfシステムは親EntityのVisualに対して`AddVisual`を呼び出さなければならない
2. When VisualGraphicsを持つEntityのChildOfが変更される時、wintfシステムは旧親Visualから`RemoveVisual`を呼び出し、新親Visualに`AddVisual`を呼び出さなければならない
3. When VisualGraphicsを持つEntityからChildOfが削除される時、wintfシステムは親Visualから`RemoveVisual`を呼び出さなければならない
4. wintfシステムは、`sync_visual_tree`システムでビジュアルツリー同期を実行しなければならない
5. wintfシステムは、Childrenの順序に従ってVisualのZ-orderを設定しなければならない（後の子が手前）

---

### Requirement 8: VisualのOffset同期

**Objective:** システム開発者として、WidgetのArrangementをDirectComposition VisualのOffset（SetOffsetX/Y）に反映したい。これにより、GPU側での座標計算が活用される。

#### Acceptance Criteria (R8)

1. When Entityの`Arrangement`が変更される時、wintfシステムは対応するVisualの`SetOffsetX`と`SetOffsetY`を呼び出さなければならない
2. wintfシステムは、`Arrangement`のoffset成分をVisualのOffset値として設定しなければならない
3. wintfシステムは、`sync_visual_offsets`システムで`Changed<Arrangement>`かつ`VisualGraphics`を持つEntityのVisual Offsetを更新しなければならない

**Note:** 全Visual方式では、各WidgetのVisualが親からの相対位置（`Arrangement`）を持つため、`GlobalArrangement`ではなく`Arrangement`を使用する。

---

### Requirement 9: Visualライフサイクル管理

**Objective:** システム開発者として、Entityのdespawn時にVisualリソースを適切にクリーンアップしたい。これにより、リソースリークを防止できる。

#### Acceptance Criteria (R9)

1. When `VisualGraphics`を持つEntityがdespawnされる時、wintfシステムは`on_remove`フックで`VisualGraphics`のクリーンアップを実行しなければならない
2. wintfシステムは、despawn時に親Visualから子Visualを`RemoveVisual`で削除しなければならない
3. wintfシステムは、COM参照カウントにより`IDCompositionVisual`の適切な解放を保証しなければならない
4. Ifビジュアルツリー操作中にエラーが発生した場合、wintfシステムはエラーをログに記録して処理を継続しなければならない

---

### Requirement 10: システム実行順序

**Objective:** システム開発者として、ビジュアルツリー同期が適切なタイミングで実行されることを保証したい。これにより、レイアウト計算後・描画前に階層が確定する。

#### Acceptance Criteria (R10)

1. wintfシステムは、`create_visual_graphics`システムを`Added<Visual>`検知後すぐに実行しなければならない
2. wintfシステムは、`sync_visual_tree`システムを`propagate_arrangements_system`の後に実行しなければならない
3. wintfシステムは、`sync_visual_offsets`システムを`sync_visual_tree`の後に実行しなければならない
4. wintfシステムは、ビジュアルツリー同期後に`IDCompositionDevice::Commit`を呼び出して変更を確定しなければならない

---

## Out of Scope（今回のスコープ外）

以下の機能は本仕様のスコープ外とし、将来の仕様で対応する：

1. **アニメーションシステム**: Visual Animationの活用は別仕様
2. **スクロールコンテナ**: Clip機能の活用は別仕様
3. **デバイスロスト対応**: 既存のinvalidate機構を活用
4. **複数Window間のVisual共有**: 単一Window前提

---

## 技術メモ

### Visual階層とOffset

全Visual方式では、各WidgetのVisualが親Visualからの相対位置を持つ：

```
Widget Tree:              Visual Tree:
Window                    Window.Visual (0, 0)
  ├─ Panel                  ├─ Panel.Visual (10, 10)
  │    ├─ Label1            │    ├─ Label1.Visual (0, 0)  ← Panel内での相対位置
  │    └─ Label2            │    └─ Label2.Visual (0, 30)
  └─ Button                 └─ Button.Visual (10, 200)
```

### システム実行順序

```text
Startup/Runtime:
  create_visual_graphics    ← NEW: Visual追加時にVisualGraphics作成

Label/TextMeasure Schedule:
  measure_text_size           ← NEW: テキスト測定
  sync_label_size_to_box_style ← NEW: BoxStyleに反映

Layout Schedule:
  build_taffy_styles_system
  sync_taffy_tree_system
  compute_taffy_layout_system
  propagate_arrangements_system

Render Schedule:
  sync_visual_tree          ← NEW: ChildOf変更→Visual親子同期
  sync_visual_offsets       ← NEW: LocalArrangement→Visual Offset同期
  ensure_surface_graphics   ← NEW: 描画前にSurfaceGraphics確保
  render_surfaces_system
  commit_system
```

### ウィジット実装パターン

```rust
// Label ウィジット（描画あり）
#[derive(Component)]
#[component(on_add = on_label_add)]
pub struct Label { pub text: String }

fn on_label_add(mut world: DeferredWorld, entity: Entity, _: HookContext) {
    insert_visual(&mut world, entity);  // Visual追加 → VisualGraphics自動作成
}
// 描画時にSurfaceGraphicsが自動作成される

// Panel（Container、描画なし）
#[derive(Component)]
#[component(on_add = on_panel_add)]
pub struct Panel;

fn on_panel_add(mut world: DeferredWorld, entity: Entity, _: HookContext) {
    insert_visual(&mut world, entity);  // Visual追加 → VisualGraphics自動作成
}
// SurfaceGraphicsは作成されない（描画しないため）
```

---

## 次のステップ

要件が承認された場合:
1. `/kiro-validate-gap visual-tree-synchronization` でGap分析を実行（推奨）
2. `/kiro-spec-design visual-tree-synchronization -y` で設計フェーズに進む
