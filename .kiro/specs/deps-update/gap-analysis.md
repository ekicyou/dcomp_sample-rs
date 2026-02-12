# ギャップ分析レポート: deps-update

## 1. 現状調査

### 1.1 依存パッケージ一覧と最新バージョン

| パッケージ | 現行バージョン | 互換最新 | 最新 | 種別 | 備考 |
|---|---|---|---|---|---|
| bevy_ecs | 0.17.2 | 0.17.3 | **0.18.0** | workspace/本番 | メジャーAPI変更あり |
| bevy_app | 0.17.2 | 0.17.3 | **0.18.0** | workspace/本番 | bevy_ecsと同期必須 |
| bevy_tasks | 0.17.2 | 0.17.3 | **0.18.0** | workspace/本番 | bevy_ecsと同期必須 |
| bevy_utils | 0.17.2 | 0.17.3 | **0.18.0** | workspace/本番 | bevy_ecsと同期必須 |
| ambassador | 0.4.2 | 0.4.2 | **0.5.0** | workspace/本番 | メジャー変更の可能性 |
| rand | 0.9.2 | 0.9.2 | **0.10.0** | workspace/dev | API変更あり(2026年2月リリース) |
| taffy | 0.9.1 | **0.9.2** | 0.9.2 | workspace/本番 | パッチ更新のみ |
| human-panic | 2.0.3 | **2.0.6** | 2.0.6 | workspace/dev | パッチ更新のみ |
| async-io | 2.3 | **2.6.0** | 2.6.0 | crate/dev | マイナー更新 |
| windows | 0.62.2 | — | — | workspace/本番 | 最新済み |
| windows-core | 0.62.2 | — | — | workspace/本番 | 最新済み |
| windows-numerics | 0.3.1 | — | — | workspace/本番 | 最新済み |
| async-executor | 1.13.3 | — | — | workspace/本番 | 最新済み |
| async-task | 4.7.1 | — | — | workspace/本番 | 最新済み |
| nonmax | 0.5.5 | — | — | workspace/本番 | 最新済み |
| tracing | 0.1 | — | — | workspace/本番 | 最新済み |
| tracing-subscriber | 0.3 | — | — | workspace/dev | 最新済み |
| image | 0.25.9 | — | — | crate/dev | 最新済み |

### 1.2 コードベース使用状況サマリー

| パッケージ | 使用範囲 | 影響ファイル数 | リスク |
|---|---|---|---|
| bevy_ecs/bevy_app | src/ 全体 + tests/ 全体 | 30+ ファイル | **高** |
| ambassador | src/ 3ファイル + examples/ 1ファイル | 4 ファイル | **中** |
| rand | examples/ 1ファイル (dcomp_demo.rs) | 1 ファイル | **低** |
| taffy | src/ecs/layout/ 全体 | 5+ ファイル | **低** (パッチ) |
| async-io | examples/ 2ファイル | 2 ファイル | **低** |
| human-panic | examples/ | — | **低** (パッチ) |

## 2. 要件実現性分析

### R1: ワークスペース依存の最新化

#### 安全な更新 (互換)
- `taffy` 0.9.1 → 0.9.2: パッチ更新、API変更なし
- `human-panic` 2.0.3 → 2.0.6: パッチ更新、API変更なし
- `async-io` 2.3 → 2.6: マイナー更新、後方互換

#### 非互換更新 (破壊的変更の可能性)

**bevy_* 0.17.2 → 0.18.0**

本プロジェクトは bevy_ecs をスタンドアロンで使用しており、bevy のレンダリング/UI/アセットパイプラインは未使用。0.18マイグレーションガイドの変更の大半（RenderTarget, Mesh, AssetLoader, FontAtlas, Camera, gltf等）は**影響なし**。

影響する可能性のある変更:

| 変更 | 影響度 | 本プロジェクトでの該当 |
|---|---|---|
| **Entities APIs rework**: EntityRow→EntityIndex | **要調査** | `Entity` の使用は広範だが、`Entity::row()` 等の直接使用を要確認 |
| **Schedule cleanup**: ScheduleGraph内部API変更 | **低** | カスタム ScheduleLabel 14個定義あるが、ScheduleGraph直接操作なし |
| **Tick-related refactors**: Tick等がchange_detectionモジュールへ移動 | **要調査** | `change_detection::DetectChangesMut` のインポートあり |
| **Renamed clear_children → detach_\*** | **要調査** | hierarchy操作の使用を要確認 |
| **Internal component removed** | **低** | Internal コンポーネントの使用なし |
| **FunctionSystem Generics**: 新ジェネリックパラメータ追加 | **低** | `IntoSystem` 明示使用はテスト1ファイルのみ |
| **Resource derive: non-static lifetime禁止** | **低** | 非staticライフタイムのResourceは通常なし |
| **Immutable EntityEvents** | **低** | EntityEvent未使用 |
| **SimpleExecutor removed** | **低** | SimpleExecutor未使用 |

**既に移行済みの項目 (リスクなし)**:
- ✅ Events → Messages 移行済み（`#[derive(Message)]`使用中）
- ✅ Bundle derive 未使用
- ✅ Event derive 未使用
- ✅ SystemSet 未使用
- ✅ App::add_event 未使用

**ambassador 0.4.2 → 0.5.0**

使用パターン: `#[delegatable_trait]`、`#[derive(Delegate)]`、`#[delegate(Trait, target="...")]`
- src/win_state.rs, src/win_message_handler.rs, src/com/d2d/mod.rs で trait定義
- examples/dcomp_demo.rs で derive使用
- 0.5.0 のREADMEは基本的に同じAPIパターンを使用しており、主要APIの互換性は維持されている可能性が高い
- **Research Needed**: 具体的なbreaking changesの確認（CHANGELOGの精査）

**rand 0.9.2 → 0.10.0**

使用箇所: examples/dcomp_demo.rs のみ (dev-dependency)
- `rand::rng()`, `rng.random_range()`, `values.shuffle()` を使用
- rand 0.10は2026年2月リリース。API変更あり
- **Research Needed**: `rng()`, `random_range()`, `shuffle()` の0.10互換性確認

### R2-R4: ビルド・テスト・サンプル成功の保証

- 互換更新のみであれば自動的に達成される
- 非互換更新の場合、コード修正が必要になる箇所は上記の通り
- テスト: 30+ のテストファイルが bevy_ecs Schedule API を使用

### R5: ステアリング情報の整合性

- tech.md の Key Libraries セクションにバージョン番号記載あり
- 更新対象: bevy_ecs, taffy, windows, async-executor, windows-numerics のバージョン番号

## 3. 実装アプローチオプション

### Option A: 互換更新のみ (Conservative)

bevy_* は 0.17.3（互換最新）にとどめ、他の互換更新のみ適用。

**更新内容**:
- bevy_ecs/bevy_app/bevy_tasks/bevy_utils: 0.17.2 → 0.17.3
- taffy: 0.9.1 → 0.9.2
- human-panic: 2.0.3 → 2.0.6
- async-io: 2.3 → 2.6

**Trade-offs**:
- ✅ コード修正ほぼ不要、低リスク
- ✅ ビルド・テスト通過がほぼ確実
- ❌ ambassador, rand は最新にならない（互換最新 = 現行）
- ❌ bevy 0.18の改善（Entity API改善、パフォーマンス向上）が得られない

### Option B: 全面最新化 (Aggressive)

全パッケージを最新安定バージョンに更新。

**更新内容**:
- bevy_*: 0.17.2 → 0.18.0
- ambassador: 0.4.2 → 0.5.0
- rand: 0.9.2 → 0.10.0
- taffy: 0.9.1 → 0.9.2
- human-panic: 2.0.3 → 2.0.6
- async-io: 2.3 → 2.6

**Trade-offs**:
- ✅ 全パッケージが最新、セキュリティ・パフォーマンス最大
- ✅ bevy 0.18のEntity API改善や最新機能が利用可能
- ❌ bevy 0.18のbreaking changes対応が必要（影響範囲は既移行済みにより限定的）
- ❌ ambassador 0.5, rand 0.10のbreaking changes対応が必要
- ❌ 作業量が大きく、全テスト・サンプルの検証が必要

### Option C: ハイブリッド (Recommended Investigation)

互換更新 + ambassador/rand を除く非互換更新を段階的に実施。

**Phase 1**: 互換更新のみ (Option Aと同等)
**Phase 2**: bevy_* 0.18.0 への更新 + コード修正
**Phase 3**: ambassador 0.5.0, rand 0.10.0 への更新（必要に応じて別仕様として分離）

**Trade-offs**:
- ✅ 段階的にリスクを管理
- ✅ Phase 1でまず安全な改善を得られる
- ✅ Phase 2/3で問題が出ても切り戻しが容易
- ❌ 複数フェーズの計画・実行が必要
- ❌ 最終的な作業量はOption Bと同等

## 4. 複雑度・リスク評価

| オプション | 工数 | リスク | 根拠 |
|---|---|---|---|
| Option A | **S** (1-2日) | **低** | パッチ/マイナー更新のみ、コード修正不要 |
| Option B | **M** (3-5日) | **中** | bevy 0.18のbreaking changesは多いが、本プロジェクトへの影響は限定的（スタンドアロンECS使用、既にMessage API移行済み）|
| Option C | **M** (3-5日, 分割可能) | **低-中** | 段階的実施でリスク分散 |

## 5. 設計フェーズへの推奨事項

### 採用アプローチ（開発者決定）
**Option B（全面最新化）**を採用。理由:
- 開発者判断により、非互換更新を含む全面最新化を実施
- bevy 0.18, ambassador 0.5, rand 0.10 への更新とAPI変更対応を本仕様に含める

### Research Items（設計フェーズで実施）
1. **bevy 0.18 breaking changes**: Entity API, Schedule, Tick関連の詳細調査とコード影響範囲の特定
2. **ambassador 0.5 breaking changes**: CHANGELOG精査と使用箇所への影響確認
3. **rand 0.10 API changes**: examples/dcomp_demo.rs での使用API（`rng()`, `random_range()`, `shuffle()`）の互換性確認
3. **rand 0.10 API変更**: dev-dependencyのみなので影響軽微（別仕様推奨）

### 主要な決定事項
- 更新範囲: 互換のみ vs 非互換含む
- bevy 0.18移行: 本仕様に含めるか、別仕様に分離するか
