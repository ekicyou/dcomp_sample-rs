# Requirements Document

## Project Description (Input)
仮想デスクトップとディスプレイをエンティティ管理する。
仮想デスクトップをルートとしたツリーを構築し、
仮想デスクトップを頂点としたtaffyレイアウト計算を可能にする。

## Introduction
本機能は、wintfフレームワークにおいて、仮想デスクトップとモニタの階層構造をECSエンティティとして管理し、Taffyレイアウトエンジンによる統一的なレイアウト計算を実現します。

これまでのWindowエンティティを頂点としたレイアウトツリーから、VirtualDesktop → {Monitor, Window} → Widget の階層に拡張することで、マルチモニタ環境での柔軟なウィンドウ配置とレイアウト計算を可能にします。

### 設計決定: MonitorとWindowは同階層

**採用設計**: `VirtualDesktop → {Monitor, Window} → Widget`

**設計根拠**:
1. **ツリー張替えの回避**: Windowがモニター間を移動しても親子関係の変更が不要。ツリー張替えに伴うGPUリソースの意図しない初期化を防ぐ
2. **概念的等価性**: Monitorは全画面Windowと仮想的に等価であり、レイアウト計算上は対等な存在
3. **メタ情報としての機能**: Monitorはwindowの親ではなく、レイアウト参照用の情報として機能
4. **実装の単純化**: Window移動時の親子関係管理が不要になり、処理が大幅に簡素化される

**Window-Monitor関連付け**:
- WindowとMonitorの対応は親子関係ではなく、**参照コンポーネント**で管理
- `WindowMonitorRef` コンポーネントで現在のモニターを追跡
- Window移動時は参照コンポーネントのみを更新（ツリー構造は不変）

---

### 設計方針
- **レイアウト階層**: Windows API の内部実装（Window Station/Desktop Object）に縛られず、wintfフレームワーク独自の実用的な階層を構築
- **Windows APIマッピング**: 物理的なモニタ情報（HMONITOR）を活用し、仮想デスクトップは論理的なグループ化として実装
- **既存システムとの統合**: 既存の `Arrangement` システムと `taffy` レイアウトエンジンとの一貫性を保持
- **段階的移行**: 既存の `BoxStyle`/`BoxComputedLayout` を `TaffyStyle`/`TaffyComputedLayout` に名称変更し、段階的に新機能を追加

## Requirements

### Requirement 1: コンポーネント定義とECS統合
**Objective:** 開発者として、仮想デスクトップとモニタをECSエンティティとして扱えるようにしたい。これにより、既存のウィンドウ管理システムとシームレスに統合できる。

#### Acceptance Criteria
1. wintf システムは、`VirtualDesktop` コンポーネントを定義し、仮想デスクトップの名前と状態（アクティブ/非アクティブ）を保持しなければならない
2. wintf システムは、`Monitor` コンポーネントを定義し、HMONITOR ハンドル、画面座標（x, y）、サイズ（width, height）、DPI、プライマリモニタフラグを保持しなければならない
3. wintf システムは、`WindowMonitorRef` コンポーネントを定義し、Windowが現在配置されているMonitorエンティティへの参照を保持しなければならない
4. wintf システムは、`MonitorInfo` 構造体から Windows API（`GetMonitorInfoW`, `GetDpiForMonitor`）経由でモニタ情報を取得する機能を提供しなければならない
5. When システムが初期化される際、wintf システムは `EnumDisplayMonitors` を使用して全モニタを列挙し、`Monitor` エンティティとして生成しなければならない
6. wintf システムは、`VirtualDesktop` の子として `Monitor` および `Window` エンティティを `ChildOf` および `Children` コンポーネントで管理しなければならない
7. wintf システムは、`App` リソースを拡張し、`DisplayConfigurationChanged` フラグを保持しなければならない

### Requirement 2: エンティティ階層の構築
**Objective:** 開発者として、VirtualDesktop → {Monitor, Window} → Widget の階層構造を構築し、Taffy レイアウト計算のルートとして使用したい。

#### Acceptance Criteria
1. wintf システムは、`VirtualDesktop` エンティティをルートノードとし、複数の `Monitor` および `Window` エンティティを子として持つ階層を構築しなければならない
2. wintf システムは、`Monitor` と `Window` を同じ階層レベル（VirtualDesktopの直接の子）に配置しなければならない
3. When ウィンドウが作成される際、wintf システムは `MonitorFromWindow` API を使用してモニタを特定し、`WindowMonitorRef` コンポーネントに対応するMonitorエンティティへの参照を設定しなければならない
4. When ウィンドウがモニター間を移動した場合、wintf システムは `WindowMonitorRef` コンポーネントのみを更新し、ツリー構造（親子関係）は変更しなければならない
5. When モニタ構成が変更された場合（モニタの追加/削除/解像度変更）、wintf システムは `Monitor` エンティティの情報を更新しなければならない
6. wintf システムは、`VirtualDesktop` エンティティが削除される際、子孫の `Monitor`, `Window`, `Widget` エンティティも適切にクリーンアップしなければならない

### Requirement 3: Taffy スタイルコンポーネントの名称変更
**Objective:** 開発者として、既存のレイアウトコンポーネント名を Taffy との統合を明示する名称に変更し、コードの意図を明確にしたい。

#### Acceptance Criteria
1. wintf システムは、既存の `BoxStyle` コンポーネントを `TaffyStyle` に名称変更しなければならない
2. wintf システムは、既存の `BoxComputedLayout` コンポーネントを `TaffyComputedLayout` に名称変更しなければならない
3. When 名称変更が完了した際、wintf システムは既存の全システム関数とテストコードで新しい名称を使用しなければならない
4. wintf システムは、`TaffyStyle` コンポーネントに `taffy::Style` 構造体を保持し、Taffy レイアウトエンジンと直接統合しなければならない
5. wintf システムは、`TaffyComputedLayout` コンポーネントに `taffy::Layout` 構造体を保持し、計算結果を格納しなければならない

### Requirement 4: Taffy ツリーの構築と管理
**Objective:** 開発者として、ECS エンティティ階層から Taffy のノードツリーを構築し、レイアウト計算を実行できるようにしたい。

#### Acceptance Criteria
1. wintf システムは、`TaffyTree` リソースを定義し、`taffy::Taffy` インスタンスと Entity → NodeId のマッピングを保持しなければならない
2. When `VirtualDesktop`, `Monitor`, `Window`, `Widget` エンティティが生成される際、wintf システムは対応する Taffy ノードを作成し、ツリー構造を構築しなければならない
3. wintf システムは、`Monitor` の物理サイズ（width, height）と座標（x, y）を `TaffyStyle` の `size` および `inset` プロパティに反映しなければならない
4. When エンティティ階層が変更された場合、wintf システムは Taffy ツリーの親子関係を同期しなければならない
5. wintf システムは、`build_taffy_tree_system` を提供し、ECS エンティティから Taffy ツリーを構築する処理を実行しなければならない

### Requirement 5: レイアウト計算の実行
**Objective:** 開発者として、VirtualDesktop をルートとして Taffy レイアウト計算を実行し、計算結果を各エンティティに反映したい。

#### Acceptance Criteria
1. wintf システムは、`compute_taffy_layout_system` を提供し、`VirtualDesktop` エンティティをルートノードとして Taffy のレイアウト計算を実行しなければならない
2. When レイアウト計算が完了した際、wintf システムは計算結果（`taffy::Layout`）を各エンティティの `TaffyComputedLayout` コンポーネントに書き込まなければならない
3. wintf システムは、`distribute_computed_layouts_system` を提供し、Taffy の計算結果を各エンティティの `TaffyComputedLayout` に配布しなければならない
4. When `TaffyComputedLayout` が更新された際、wintf システムは既存の `Arrangement` 更新システムを呼び出し、最終的なウィンドウ配置を計算しなければならない
5. wintf システムは、レイアウト計算を `VirtualDesktop` → `Monitor` → `Window` → `Widget` の順序で実行し、親から子への依存関係を保証しなければならない

### Requirement 6: 増分更新とパフォーマンス最適化
**Objective:** 開発者として、レイアウト計算を増分的に行い、毎回全ツリーを再計算することなく効率的に更新したい。

#### Acceptance Criteria
1. wintf システムは、`LayoutDirty` マーカーコンポーネントを定義し、レイアウト再計算が必要なエンティティを追跡しなければならない
2. When `TaffyStyle`, `MonitorInfo`, または階層構造が変更された場合、wintf システムは該当エンティティに `LayoutDirty` マーカーを付与しなければならない
3. When `MonitorInfo` が変更された場合、wintf システムは `LayoutDirty.subtree_dirty` フラグを true に設定し、サブツリー全体の再計算をマークしなければならない
4. When `TaffyStyle` のみが変更された場合、wintf システムは該当ノードのみを `taffy.mark_dirty()` でマークし、部分的な再計算を実行しなければならない
5. wintf システムは、`LayoutDirty` マーカーを持つエンティティのみをクエリし、不要なレイアウト計算を回避しなければならない

### Requirement 7: モニタ情報の動的更新
**Objective:** 開発者として、モニタ構成の変更（解像度変更、モニタ追加/削除）を検知し、自動的にモニタ情報とレイアウトを更新したい。

#### Acceptance Criteria
1. When Windows メッセージ `WM_DISPLAYCHANGE` を受信した場合、wintf システムはメッセージハンドラで `DisplayConfigurationChanged` フラグを `App` リソースに設定しなければならない
2. wintf システムは、`detect_display_change_system` を提供し、`App` リソースの `DisplayConfigurationChanged` フラグを監視しなければならない
3. When `DisplayConfigurationChanged` フラグが true の場合、wintf システムは `EnumDisplayMonitors` を使用して全モニタ情報を再取得しなければならない
4. When 新しいモニタが検出された場合、wintf システムは新しい `Monitor` エンティティを生成し、`VirtualDesktop` の子として追加しなければならない
5. When モニタが削除された場合、wintf システムは該当する `Monitor` エンティティを削除しなければならない
6. When モニタが削除された場合、wintf システムは削除されたモニタを参照していた全 `Window` エンティティの `WindowMonitorRef` をプライマリモニタに更新しなければならない
7. When モニタ情報が更新された場合、wintf システムは `LayoutDirty` マーカーを付与し、レイアウトの再計算をトリガーしなければならない
8. wintf システムは、`DisplayConfigurationChanged` フラグを処理後に false にリセットしなければならない

### Requirement 8: システムスケジュールの統合
**Objective:** 開発者として、新しいレイアウトシステムが既存のECSスケジュールに適切に統合され、正しい順序で実行されるようにしたい。

#### Acceptance Criteria
1. wintf システムは、レイアウト関連のシステムを以下の順序で実行しなければならない: `update_virtual_desktop_style` → `update_monitor_style` → `update_window_style` → `compute_taffy_layout` → `distribute_computed_layouts` → `update_arrangements`
2. wintf システムは、`compute_taffy_layout_system` を `update_*_style` システムの後に実行するよう、依存関係を設定しなければならない
3. wintf システムは、`distribute_computed_layouts_system` を `compute_taffy_layout_system` の後に実行しなければならない
4. wintf システムは、既存の `Arrangement` 更新システムを `distribute_computed_layouts_system` の後に実行しなければならない
5. When システムスケジュールが構築された際、wintf システムは循環依存が存在しないことを保証しなければならない

### Requirement 9: 既存システムとの互換性維持
**Objective:** 開発者として、既存のウィンドウ管理とレイアウトシステムを破壊せず、段階的に新機能を追加したい。

#### Acceptance Criteria
1. While 新しい階層システムが実装される間、wintf システムは既存の `Window` エンティティベースのレイアウト計算を継続してサポートしなければならない
2. wintf システムは、`VirtualDesktop` または `Monitor` エンティティが存在しない場合でも、既存のレイアウトシステムが正常に動作することを保証しなければならない
3. When 既存のテストが実行された際、wintf システムは名称変更（`BoxStyle` → `TaffyStyle`）を除き、すべてのテストがパスしなければならない
4. wintf システムは、`GlobalArrangement` から `WindowPos` への変換処理を維持し、Win32 API との統合を保持しなければならない
5. wintf システムは、既存の `Surface` 最適化機能（サイズ変更時の再生成判定）を継続してサポートしなければならない

### Requirement 10: テストとバリデーション
**Objective:** 開発者として、新しい階層システムの動作を検証し、レイアウト計算が正しく実行されることを確認したい。

#### Acceptance Criteria
1. wintf システムは、`VirtualDesktop` → `Monitor` → `Window` 階層が正しく構築されることを検証するテストを提供しなければならない
2. wintf システムは、`Monitor` の物理サイズと座標が `TaffyStyle` に正しく反映されることを検証するテストを提供しなければならない
3. wintf システムは、Taffy レイアウト計算の結果が `TaffyComputedLayout` に正しく格納されることを検証するテストを提供しなければならない
4. wintf システムは、`LayoutDirty` による増分更新が正しく動作し、不要な計算が回避されることを検証するテストを提供しなければならない
5. wintf システムは、モニタ構成変更時に階層とレイアウトが正しく更新されることを検証するテストを提供しなければならない
