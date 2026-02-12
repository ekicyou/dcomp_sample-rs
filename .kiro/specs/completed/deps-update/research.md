# Research & Design Decisions: deps-update

## Summary
- **Feature**: `deps-update`
- **Discovery Scope**: Extension（既存システムの依存パッケージ全面更新）
- **Key Findings**:
  - bevy 0.18 の破壊的変更は大量だが、本プロジェクト（スタンドアロンECS使用）への影響は限定的
  - ambassador 0.5 は基本APIパターンを維持しており、影響は最小限と推定
  - rand 0.10 は `Rng` → `RngExt` のリネームが主な影響点（使用箇所は1ファイルのみ）

## Research Log

### bevy 0.18 マイグレーションガイド分析

- **Context**: bevy_ecs/bevy_app/bevy_tasks/bevy_utils を 0.17.2 → 0.18.0 に更新するために、全破壊的変更の影響を調査
- **Sources Consulted**: https://bevy.org/learn/migration-guides/0-17-to-0-18/
- **Findings**:

#### 本プロジェクトに影響なしの変更（大多数）
以下はレンダリング/UI/アセット/glTF等のサブシステム固有であり、スタンドアロンECS使用の本プロジェクトには**無関係**:
- RenderTarget, Mesh, AssetLoader, FontAtlas, Camera, gltf, Atmosphere, AnimationTarget, Virtual Geometry, BevyManifest, NodeBorderRadius, LineHeight, TilemapChunk, Gizmos, TrackedRenderPass, BorderRect, BindGroupLayout, RenderPipelineDescriptor, ExtractedUiNode, bevy_gizmos, bevy_reflect, AmbientLight, Text/TextLayoutInfo, DragEnter, AssetSources, AssetServer, ImageRenderTarget, Feature cleanup, Cargo Feature Collections, ArchetypeQueryData, System Combinators, Winit user events

#### 本プロジェクトに影響する可能性のある変更

| 変更 | 影響度 | コードベース調査結果 |
|---|---|---|
| **Entity APIs**: `EntityRow` → `EntityIndex`, `Entity::row()` → `Entity::index()` | **なし** | `Entity::row()`, `EntityRow` の使用なし |
| **Tick-related refactors**: `Tick`, `ComponentTicks` が `component` → `change_detection` モジュールへ移動 | **なし** | bevy の `Tick` 型をインポートしていない |
| **Schedule cleanup**: `ScheduleGraph` 内部API変更 | **なし** | `ScheduleGraph` 直接操作なし |
| **FunctionSystem Generics**: 新ジェネリックパラメータ `In` 追加 | **低** | `IntoSystem::into_system()` をテスト3ファイルで計14箇所使用。public APIとしての `IntoSystem::into_system()` は互換性維持される見込み |
| **Removed SimpleExecutor** | **なし** | 未使用 |
| **clear_children → detach_\*** | **なし** | 未使用 |
| **Resource derive non-static lifetime** | **なし** | 非staticライフタイムのResource未使用 |
| **Internal removed** | **なし** | 未使用 |
| **Immutable EntityEvents** | **なし** | EntityEvent未使用 |
| **Bundle::component_ids returns iterator** | **なし** | カスタムBundle impl未使用 |
| **Column → ThinColumn** | **要確認** | 内部API。`bevy_ecs::prelude::*` 経由での間接影響を確認要 |
| **Entities rework** | **低** | `Entities` 型を直接操作していない |
| **Same State Transitions** | **なし** | bevy States未使用 |

#### 要コンパイル確認の項目
以下はマイグレーションガイドに明示されていないが、パス安定性を確認する必要がある:
- `bevy_ecs::change_detection::DetectChangesMut` — 3 src ファイル + 3 test ファイルでインポート
- `bevy_ecs::lifecycle::HookContext` — 7 src ファイルでインポート
- `bevy_ecs::component::Mutable` — 1 ファイルでインポート
- `bevy_ecs::system::lifetimeless::*` — 1 ファイルでインポート
- `bevy_ecs::message::{Message, Messages, MessageReader}` — 3 src ファイルでインポート
- `bevy_ecs::schedule::{ExecutorKind, ScheduleLabel, IntoScheduleConfigs, ScheduleSystem}` — 1 ファイルでインポート
- `bevy_ecs::hierarchy::{ChildOf, Children}` — 6 ファイルでインポート
- `bevy_ecs::name::Name` — 5 ファイルでインポート

**結論**: マイグレーションガイドに記載された変更の大部分（30+項目）は本プロジェクトに影響しない。影響の可能性があるのは `IntoSystem` ジェネリクス変更（テストのみ）と、明示されていないパス変更リスクのみ。**コンパイル→修正の反復アプローチが最も効率的**。

### ambassador 0.5 変更調査

- **Context**: ambassador 0.4.2 → 0.5.0 の破壊的変更を特定
- **Sources Consulted**: https://crates.io/crates/ambassador/0.5.0 、GitHub releases（リリースノートなし）
- **Findings**:
  - crates.io の 0.5.0 公式ページの使用例は 0.4 と同一のAPIパターン
  - `#[delegatable_trait]`, `#[derive(Delegate)]`, `#[delegate(Trait)]` のすべてが維持
  - `target = "..."`, `where = "..."`, `generics = "..."` キーも維持
  - GitHub releases ページにリリースノートなし → CHANGELOG も存在しない
  - **結論**: 基本的なAPIは互換性維持。内部改善またはバグ修正がメイン。コンパイルで確認する方針

- **Implications**: ambassador の更新は Cargo.toml のバージョン変更のみで完了する可能性が高い。万一コンパイルエラーが出た場合もエラーメッセージに従って修正可能

### rand 0.10 変更調査

- **Context**: rand 0.9.2 → 0.10.0（dev-dependency）の破壊的変更を特定
- **Sources Consulted**: https://github.com/rust-random/rand/blob/master/CHANGELOG.md
- **Findings**:

#### 0.10.0 の主な破壊的変更（本プロジェクト関連）
| 変更 | 影響 |
|---|---|
| `Rng` → `RngExt`（`rand_core::RngCore` → `Rng` に伴いリネーム） | **高** — `use rand::*` で `RngExt` が入るが、`.random_range()` メソッドの呼び出しに影響 |
| `choose_multiple` → `sample`, `choose_multiple_array` → `sample_array` | **なし** — 未使用 |
| `os_rng` → `sys_rng`, `OsRng` → `SysRng` | **なし** — 未使用 |
| `SeedableRng::from_os_rng`, `try_from_os_rng` 削除 | **なし** — 未使用 |
| `StdRng`, `ReseedingRng` の `Clone` 削除 | **なし** — 未使用 |
| `ReseedingRng` 削除 | **なし** — 未使用 |
| `small_rng` feature 削除 | **確認要** — Cargo.toml の feature 指定を確認 |
| Edition 2024, MSRV 1.85 | **OK** — 本プロジェクトは Edition 2024 |

#### 本プロジェクトでの使用箇所（examples/dcomp_demo.rs のみ）
```rust
use rand::{seq::*, *};
let mut rng = rand::rng();           // rand::rng() は 0.9 で導入、0.10 でも維持
rng.random_range(b'A'..=b'Z')        // 0.9 で gen_range → random_range にリネーム済
                                      // 0.10 では RngExt trait のメソッド
values.shuffle(&mut rng)              // SliceRandom::shuffle — seq::* でインポート
```

**修正方針**: `use rand::*` で `RngExt` がインポートされるため、glob import を使用中の本コードはそのままコンパイルが通る可能性がある。通らない場合は `use rand::RngExt;` を明示的に追加。

### 互換パッケージ（修正不要）

| パッケージ | 更新 | 根拠 |
|---|---|---|
| taffy | 0.9.1 → 0.9.2 | パッチ更新。API変更なし |
| human-panic | 2.0.3 → 2.0.6 | パッチ更新。API変更なし |
| async-io | 2.3 → 2.6 | マイナー更新。後方互換 |

## Architecture Pattern Evaluation

本フィーチャーはアーキテクチャ変更を伴わない「依存パッケージ更新」であり、パターン評価は不要。

## Design Decisions

### Decision: 段階的更新アプローチ
- **Context**: 6パッケージのバージョン更新を安全に実施する更新順序の決定
- **Alternatives Considered**:
  1. 全パッケージ一括更新 — 一度に全更新して一気に修正
  2. パッケージグループ別段階更新 — 安全→非互換の順に段階的に更新
- **Selected Approach**: パッケージグループ別段階更新
- **Rationale**:
  - 互換更新（taffy, human-panic, async-io）を先に適用することで安全な基盤を確保
  - bevy 0.18 を次に適用（最大の影響範囲だが、ほとんどの変更は無関係と判明）
  - ambassador 0.5 と rand 0.10 を最後に適用（影響範囲が小さい）
  - 各段階でビルド・テスト確認を挟むことでエラー原因の特定が容易
- **Trade-offs**:
  - ✅ エラー原因の切り分けが容易
  - ✅ 途中段階でも動作する状態を維持
  - ❌ 全工程の時間がやや長い（ただし手戻りリスクが低いため総工数は同等以下）

### Decision: コンパイル駆動修正
- **Context**: bevy 0.18 の未文書化パス変更への対処方針
- **Selected Approach**: バージョン更新後にコンパイル → エラーメッセージに従って修正する反復アプローチ
- **Rationale**: bevy は内部モジュール再配置を行うが、多くの場合 `prelude::*` で再エクスポートされる。マイグレーションガイドに記載されていないパス変更は、コンパイラが正確に指摘してくれる

## Risks & Mitigations
- **bevy 0.18 未文書化パス変更** — コンパイル駆動修正で対処。`prelude::*` の再エクスポートに期待
- **ambassador 0.5 隠れた非互換** — CHANGELOGが存在しないため完全な事前把握は困難。コンパイルで確認
- **rand 0.10 の Rng → RngExt** — glob import (`use rand::*`) で自動解決される可能性あり。解決されない場合は明示的インポート追加
- **bevy_ecs Message API 安定性** — 0.17 で導入された Message API が 0.18 でパス変更される可能性。コンパイルで確認

## References
- [Bevy 0.17→0.18 Migration Guide](https://bevy.org/learn/migration-guides/0-17-to-0-18/) — 公式マイグレーションガイド
- [rand 0.10.0 CHANGELOG](https://github.com/rust-random/rand/blob/master/CHANGELOG.md) — rand 0.9→0.10 の全変更一覧
- [ambassador 0.5.0 crates.io](https://crates.io/crates/ambassador/0.5.0) — ambassador 0.5 の公式ページ（APIドキュメント）
- [gap-analysis.md](./gap-analysis.md) — 本仕様の事前ギャップ分析レポート
