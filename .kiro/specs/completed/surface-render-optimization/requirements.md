# Requirements Document

## Project Description (Input)
現在、render_surfaceシステムは、サーフェス描画を毎フレーム行っている。この状況を、「自分と小孫コマンドリストのいずれかが更新されたとき」に限ってサーフェス更新を行う実装にしたい。

## Requirements

### Requirement 1: 変更検知による描画判定
**Objective:** 描画コマンドや配置に変更があった場合のみサーフェス更新を行うことで、GPUおよびCPUリソースの消費を削減する。

#### Acceptance Criteria
1. **When** `SurfaceGraphics`を持つエンティティまたはその子孫エンティティの `GraphicsCommandList` コンポーネントが変更されたとき、`render_surface` システムは当該エンティティのサーフェス更新（描画処理）を実行しなければならない (shall)。
2. **When** `SurfaceGraphics`を持つエンティティまたはその子孫エンティティの `GlobalArrangement` コンポーネントが変更されたとき、`render_surface` システムは当該エンティティのサーフェス更新を実行しなければならない (shall)。
3. **When** `SurfaceGraphics`を持つエンティティの `SurfaceGraphics` コンポーネントが変更された（新規作成または再作成された）とき、`render_surface` システムは当該エンティティのサーフェス更新を実行しなければならない (shall)。
4. **If** 上記のいずれの変更も検知されない場合、`render_surface` システムは当該エンティティに対する `BeginDraw`、描画コマンド発行、`EndDraw` の一連の処理をスキップしなければならない (shall)。
5. **The** 変更検知の実装には、更新要求を表すマーカーコンポーネント（例: `SurfaceUpdateRequested`）を使用し、変更発生時に該当するサーフェスオーナーに付与する方式を採用しなければならない (shall)。

### Requirement 2: 子孫要素の走査範囲
**Objective:** `SurfaceGraphics`を持つエンティティに属するすべての描画要素の変更を正しく検知する。ただし、ネストされたサーフェスは独立して扱われるべきである。

**Note:** 現在の `render_surface` 実装は簡略化されており、ネストされた `SurfaceGraphics` を持つエンティティをスキップする機能は実装されていない。本仕様変更において、このスキップ機能を実装範囲に含める。

#### Acceptance Criteria
1. **The** `render_surface` システムは、変更検知において、`SurfaceGraphics`を持つエンティティ（オーナー）の子孫を再帰的に走査しなければならない (shall)。
2. **But** 走査の過程で、オーナー以外のエンティティが `SurfaceGraphics` を持っている場合、そのエンティティおよびその子孫は、現在のオーナーの変更検知対象から除外しなければならない (shall)。
3. **When** 上記の走査範囲内のエンティティのいずれか1つでも変更条件を満たした場合、`render_surface` システムはオーナー全体を再描画対象とみなさなければならない (shall)。

### Requirement 3: ログ出力の抑制（オプション）
**Objective:** 毎フレームの描画がなくなることに伴い、デバッグログの出力を適切に調整する。

#### Acceptance Criteria
1. **Where** 描画がスキップされる場合、`render_surface` システムは描画関連のログ出力を抑制しなければならない (shall)。
2. **When** 描画が実行される場合、`render_surface` システムは更新理由（どのエンティティが変更されたか等）を含むログを出力することが望ましい (should)。
