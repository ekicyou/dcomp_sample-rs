# Research & Design Decisions: Phase 2 Milestone 2 - WindowGraphics + Visual作成

---
**Purpose**: 設計判断の根拠と調査結果を記録

**Usage**: design.mdの補足情報として、詳細な調査内容と設計判断の根拠を提供
---

## Summary
- **Feature**: `phase2-m2-window-graphics`
- **Discovery Scope**: Extension（既存システムへの拡張）
- **Key Findings**:
  - COM APIラッパーは完全に実装済み、新規API呼び出しは不要
  - 既存のECSパターン（create_windowsシステム）が明確な実装指針を提供
  - ID2D1DeviceContextはウィンドウ単位でキャッシュする設計が適切

## Research Log

### DirectComposition DeviceContext管理パターン

- **Context**: ID2D1DeviceContextをウィンドウごとに保持すべきか、毎フレーム作成すべきか
- **Sources Consulted**:
  - 既存実装: `com/dcomp.rs`の`DCompositionSurfaceExt::begin_draw()`パターン
  - ギャップ分析: `.kiro/specs/phase2-m2-window-graphics/gap-analysis.md`
  - DirectCompositionのベストプラクティス: Surface描画時にDeviceContextを取得
- **Findings**:
  - GraphicsCoreはアプリケーション全体で単一のID2D1Deviceを保持
  - 既存の`D2D1DeviceExt::create_device_context()`でDeviceContextを作成可能
  - Surface描画時は`begin_draw()`がID2D1DeviceContext3を返すため、描画時専用のコンテキストが提供される
  - ウィンドウ初期化時にDeviceContextを作成してキャッシュする設計が一般的
- **Implications**:
  - WindowGraphicsコンポーネントにID2D1DeviceContextを保持
  - 初回作成時にGraphicsCore::d2d.create_device_context()を呼び出す
  - オプション: D2D1_DEVICE_CONTEXT_OPTIONS_NONEを使用（標準設定）

### ECSコンポーネントライフサイクル管理

- **Context**: COMオブジェクトのライフタイム管理をどう実装するか
- **Sources Consulted**:
  - 既存実装: `ecs/graphics.rs`のGraphicsCoreパターン
  - Rust windows-rsドキュメント: スマートポインタによる自動管理
- **Findings**:
  - windows-rsの型（IDCompositionTarget, IDCompositionVisual3等）はスマートポインタ
  - Dropトレイト実装は不要（自動的にRelease()が呼ばれる）
  - Send + Syncトレイトの実装のみ必要
- **Implications**:
  - 明示的なDrop実装は不要
  - unsafe impl Send/Syncのみ追加

### システム実行順序とスケジュール配置

- **Context**: どのスケジュールステージでシステムを実行すべきか
- **Sources Consulted**:
  - 既存実装: `ecs/world.rs`のスケジュール定義
  - 既存実装: `ensure_graphics_core`と`create_windows`の登録パターン
- **Findings**:
  - UISetupステージ: ウィンドウ作成と同じステージ（メインスレッド固定）
  - 実行順序: ensure_graphics_core → create_windows → create_window_graphics → create_window_visual
  - `.before()`/`.after()`チェーンで依存関係を明示
- **Implications**:
  - 両システムをUISetupステージに登録
  - create_window_graphics.after(create_windows)
  - create_window_visual.after(create_window_graphics)

### エラーハンドリング戦略

- **Context**: COM APIエラーをどう処理すべきか
- **Sources Consulted**:
  - 既存実装: `ecs/window_system.rs`のエラーハンドリングパターン
  - 既存実装: `ecs/graphics.rs`のensure_graphics_coreパターン
- **Findings**:
  - windows::core::Resultを使用
  - エラー時は`eprintln!`でログ出力し、そのエンティティの処理をスキップ
  - パニックは避ける（アプリケーション全体をクラッシュさせない）
- **Implications**:
  - 各COM API呼び出しでResult<T>を処理
  - match式でOk/Errを分岐
  - Err時はエンティティIDとHRESULTを含むログを出力し、continueで次へ

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Option A: graphics.rs拡張 | WindowGraphics/Visualコンポーネントと2システムを既存graphics.rsに追加 | GraphicsCore関連が1ファイルに集約、グローバル/ウィンドウ単位の関係が明確、最小限のファイル変更 | ファイルサイズ約350行（許容範囲）| **選択** - ギャップ分析で推奨 |
| Option B: 新規window_graphics.rs | 専用ファイルを作成 | 物理的な責務分離 | ファイル数増加、GraphicsCoreとの関係が離れる、インポート複雑化 | 過剰な分離と判断 |
| Option C: ハイブリッド | 段階的に実装し、必要に応じて分割 | リスク軽減 | 複数フェーズの管理負担 | 初期実装が単純なため不要 |

## Design Decisions

### Decision: `graphics.rsを拡張してWindowGraphics/Visualを実装`

- **Context**: 新規コンポーネントとシステムをどこに配置するか
- **Alternatives Considered**:
  1. Option A: 既存graphics.rsに追加
  2. Option B: 新規window_graphics.rsを作成
  3. Option C: 段階的に実装して後でリファクタリング
- **Selected Approach**: Option A - graphics.rsに約250行を追加
- **Rationale**:
  - グラフィックス関連の責務が1箇所に集約される
  - GraphicsCore（グローバル）とWindowGraphics（ウィンドウ単位）の関係が同一ファイルで理解しやすい
  - 既存パターン（GraphicsCore + ensure_graphics_core）との一貫性
  - インポート構造がシンプル（`use crate::ecs::graphics::*`で全アクセス可能）
- **Trade-offs**:
  - ✅ 最小限のファイル変更
  - ✅ 関連コード集約で保守性向上
  - ⚠️ ファイルサイズ約350行（Rust標準では許容範囲）
- **Follow-up**: 将来的に500行を超えた場合はモジュール分割を検討

### Decision: `ID2D1DeviceContextをWindowGraphicsに保持`

- **Context**: DeviceContextをキャッシュすべきか、毎回作成すべきか
- **Alternatives Considered**:
  1. 初回作成してWindowGraphicsに保持（キャッシュ）
  2. 毎フレーム作成（再生成）
- **Selected Approach**: キャッシュ方式 - WindowGraphicsコンポーネントにID2D1DeviceContextフィールドを追加
- **Rationale**:
  - DirectCompositionでは通常DeviceContextを再利用する設計が一般的
  - パフォーマンス優先（毎フレーム作成はオーバーヘッド）
  - GraphicsCoreのd2dデバイスから`create_device_context(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)`で作成
- **Trade-offs**:
  - ✅ パフォーマンス最適化
  - ✅ DirectCompositionのベストプラクティスに準拠
  - ⚠️ ウィンドウごとにDeviceContextを保持（メモリ使用量増加は微小）
- **Follow-up**: Milestone 3の描画実装時にパフォーマンスを計測

### Decision: `SparseSet storageを使用しない`

- **Context**: WindowGraphics/VisualコンポーネントのECSストレージ戦略
- **Alternatives Considered**:
  1. デフォルトストレージ（Table）
  2. SparseSet storage（WindowHandleと同様）
- **Selected Approach**: デフォルトストレージ（Table）
- **Rationale**:
  - WindowHandleはlifecycle hooks（on_add/on_remove）のためにSparseSetを使用
  - WindowGraphics/Visualはlifecycle hooksを必要としない（ECSフレームワークが自動削除）
  - デフォルトストレージで十分（密なイテレーションに有利）
- **Trade-offs**:
  - ✅ シンプルな実装
  - ✅ イテレーション性能が高い
  - ⚠️ 追加/削除がやや遅い（ウィンドウ作成/削除は低頻度なので問題なし）
- **Follow-up**: なし

### Decision: `CommitCompositionスケジュールを追加`

- **Context**: DirectCompositionの変更確定（Commit呼び出し）をどこで行うか
- **Alternatives Considered**:
  1. create_window_visualシステムの最後に含める（複数ウィンドウで冗長）
  2. 専用の`commit_composition`システムをUISetupステージに追加
  3. 専用の`commit_composition`システムを専用スケジュール（CommitComposition）に配置
- **Selected Approach**: Option 3 - 専用スケジュール（CommitComposition）を作成し、ワールドスケジュールの最後に実行
- **Rationale**:
  - **責務の分離**: ビジュアル作成とCommitを独立したシステムに分離
  - **効率性**: すべてのウィンドウのビジュアル変更をまとめてコミット（1回の呼び出し）
  - **将来の拡張性**: Milestone 3以降で描画やビジュアルツリー更新が追加されても、Commit処理は変更不要
  - **明確な実行順序**: 常にワールドスケジュールの最後に実行される専用スケジュール
  - **DirectCompositionのベストプラクティス**: すべての変更を行った後に1回Commitする設計が推奨
- **Trade-offs**:
  - ✅ 責務が明確
  - ✅ 複数ウィンドウで効率的
  - ✅ 拡張性が高い
  - ✅ DirectCompositionのベストプラクティスに準拠
  - ⚠️ 新しいスケジュールが追加される（world.rsの変更が必要）
- **Follow-up**: Milestone 3以降でビジュアルツリー更新やアニメーションが追加された場合も、CommitCompositionスケジュールは変更不要

**重要な設計原則**:
```
スケジュール実行順序（world.rs）:
Input → Update → PreLayout → Layout → PostLayout → UISetup → Draw → RenderSurface → Composition → CommitComposition
                                                                                                        ↑
                                                                                    常に最後に実行される専用スケジュール
```

この設計により、どのスケジュールでビジュアル変更が行われても、最後にCommitCompositionスケジュールで一括確定される。

### Decision: `PostLayoutスケジュールでグラフィックスシステムを実行`

- **Context**: create_window_graphicsとcreate_window_visualシステムをどのスケジュールに配置するか
- **Alternatives Considered**:
  1. UISetupスケジュールに配置（UIスレッド固定、単一スレッド実行）
  2. PostLayoutスケジュールに配置（マルチスレッド実行可能）
- **Selected Approach**: Option 2 - PostLayoutスケジュールに配置
- **Rationale**:
  - **DirectComposition/Direct2Dはマルチスレッド対応**: CreateTargetForHwnd, CreateVisual, SetRoot, CreateDeviceContextはすべてスレッドセーフ
  - **UISetupの目的は限定的**: Win32 API（CreateWindowEx, DestroyWindow等）のUIスレッド固定処理のみ
  - **パフォーマンス**: 複数ウィンドウのグラフィックスを並列作成可能
  - **UIスレッドの負荷軽減**: 不要な処理をUIスレッドから分離
  - **設計の一貫性**: UISetup="UI thread required"、PostLayout="after layout, multi-thread safe"
- **Trade-offs**:
  - ✅ マルチスレッドで実行可能（スケーラビリティ向上）
  - ✅ UIスレッドの負荷を軽減
  - ✅ DirectComposition APIの特性に合致
  - ✅ レイアウト計算後の自然な配置
  - ⚠️ UISetupとPostLayoutの責務分担を理解する必要がある
- **Follow-up**: なし

**スケジュール配置ルール**:
```
UISetup:
  - Win32 API（CreateWindowEx, DestroyWindow, SetWindowPos等）
  - UIスレッド固定が必要な処理のみ
  - SingleThreadedエグゼキュータで実行

PostLayout:
  - レイアウト計算後の処理
  - DirectComposition/Direct2D等のマルチスレッド対応API
  - マルチスレッドで実行可能
```

この設計により、グラフィックスシステムは不要なスレッド制約を受けず、最大限のパフォーマンスを発揮できる。

### Decision: `システム登録はworld.rsのデフォルトシステムに含める`

- **Context**: 新規システムをどこに登録するか
- **Alternatives Considered**:
  1. world.rsのEcsWorld::newでデフォルト登録
  2. アプリケーション側（examples/）で個別登録
- **Selected Approach**: world.rsのデフォルトシステムに追加
- **Rationale**:
  - ensure_graphics_coreとcreate_windowsが既にデフォルト登録されている
  - ウィンドウグラフィックス初期化は必須機能（全アプリケーションで必要）
  - 一貫性のあるシステム登録パターン
- **Trade-offs**:
  - ✅ 全アプリケーションで自動的に有効化
  - ✅ 既存パターンとの一貫性
  - ⚠️ オプトアウト不可（現時点では問題なし）
- **Follow-up**: なし

## Risks & Mitigations

- **Risk 1**: 複数ウィンドウでのCompositionTarget動作が未検証
  - **Mitigation**: simple_window.rsを拡張して2ウィンドウのテストケースを作成
  
- **Risk 2**: DeviceContext作成オプションが不明確
  - **Mitigation**: D2D1_DEVICE_CONTEXT_OPTIONS_NONEで開始、問題があればMilestone 3で調整

- **Risk 3**: COMエラーのデバッグが困難
  - **Mitigation**: HRESULTコードを含む詳細ログ出力、エンティティIDも記録

## References

- [Microsoft Docs - DirectComposition](https://learn.microsoft.com/en-us/windows/win32/directcomp/directcomposition-portal) — DirectComposition概要
- [Microsoft Docs - Direct2D Device Contexts](https://learn.microsoft.com/en-us/windows/win32/direct2d/devices-and-device-contexts) — DeviceContext管理
- [windows-rs Repository](https://github.com/microsoft/windows-rs) — Rust Windows APIバインディング
- `.kiro/steering/structure.md` — プロジェクト構造とレイヤードアーキテクチャ
- `.kiro/specs/phase2-m2-window-graphics/gap-analysis.md` — 実装ギャップ分析

---

## COM API Version Investigation (2025-11-14)

### 目的
Windows 11環境で利用可能な最新のCOM APIインターフェイスを確認し、現在の実装が最適かを検証。

### 調査結果

#### windows-rs 0.62.2でのインターフェイス対応状況

##### DirectComposition
- ✅ `IDCompositionDevice` (Windows 8+)
- ✅ `IDCompositionDevice2` (Windows 8.1+)
- ✅ `IDCompositionDevice3` (Windows 10 1703+) **← 現在使用中**
- ❌ `IDCompositionDevice4` (Windows 11 22H2+) - windows-rsで未サポート
- ✅ `IDCompositionVisual` (Windows 8+)
- ✅ `IDCompositionVisual2` (Windows 8.1+)
- ✅ `IDCompositionVisual3` (Windows 10 1703+) **← 現在使用中**
- ❌ `IDCompositionVisual4` (Windows 11 22H2+) - windows-rsで未サポート

##### Direct2D
- ✅ `ID2D1Device` ~ `ID2D1Device7` (Device7はWindows 10 1809+)
- ✅ `ID2D1DeviceContext` ~ `ID2D1DeviceContext7` (DeviceContext7はWindows 10 1809+)
- ✅ `ID2D1Factory` ~ `ID2D1Factory8` (Factory8はWindows 11+)

#### 現在の実装
- `crates/wintf/src/ecs/graphics.rs`: `IDCompositionDevice3`, `ID2D1Device`
- `crates/wintf/src/com/dcomp.rs`: `IDCompositionDevice3`, `IDCompositionVisual3`
- `crates/wintf/src/com/d2d/mod.rs`: `ID2D1Device`, `ID2D1DeviceContext`
- `crates/wintf/examples/dcomp_demo.rs`: `ID2D1DeviceContext7`へのcast実績あり（640行目）

### 結論: 現状維持が最適解

#### 理由
1. **DirectCompositionはDevice3が実質最新**
   - Device4/Visual4はwindows-rs 0.62.2で未対応
   - Windows APIとしては存在するが、Rustバインディングが追いついていない
   - Device3で必要な機能はすべて利用可能

2. **Direct2Dの上位バージョンは必要時にcastで対応可能**
   - 基本的な描画には`ID2D1Device`、`ID2D1DeviceContext`で十分
   - 上位バージョン固有機能（HDR、高度なカラー管理等）が必要な場合は`cast()`で対応
   - `dcomp_demo.rs`で`ID2D1DeviceContext7`への動的キャスト実績あり

3. **実装の柔軟性を維持**
   - 基本インターフェイスで実装することで、将来的なwindows-rsの更新に容易に対応可能
   - 特定バージョン固有機能は必要になった時点で追加実装

#### 追加調査事項
- windows-rsのバージョンを0.62.1から0.62.2に更新済み
- ビルド検証完了（cargo check --lib成功）
- `IDCompositionDevice4`/`Visual4`の対応は将来のwindows-rsアップデート待ち

### 参考
- Microsoft Docs: DirectComposition interfaces
- windows-rs GitHub: Interface bindings coverage
- 実装例: `crates/wintf/examples/dcomp_demo.rs`（ID2D1DeviceContext7 cast実装）
