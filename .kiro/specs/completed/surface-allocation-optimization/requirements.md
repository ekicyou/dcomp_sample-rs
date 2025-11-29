# Requirements Document

## Introduction
本仕様は、wintfライブラリにおけるSurfaceGraphics生成の最適化を定義する。現在の実装では、`Arrangement`コンポーネントの変更時に`GraphicsCommandList`の有無を確認せずSurfaceを一律作成しており、描画コマンドを持たないレイアウトコンテナ（例：FlexContainer）にも不要なVRAMが消費されている。また、Surfaceサイズが論理サイズで作成されており、DPIスケール適用時に描画がはみ出す問題がある。本最適化により、`GraphicsCommandList`の存在に基づいてSurface生成要否を動的に判定し、正しい物理ピクセルサイズでSurfaceを作成することで、GPUリソースの効率的な利用と正確な描画を実現する。

## Project Description (Input)
Surface生成の最適化: GraphicsCommandListの有無や要求サイズを集約し、SurfaceGraphicsの生成要否やサイズを動的に決定する仕組みの構築（一律作成からの脱却）

## Requirements

### Requirement 1: GraphicsCommandList存在に基づくSurface生成判定
**Objective:** 開発者として、描画コマンドを持たないエンティティにSurfaceが作成されないことで、VRAMとGPUリソースの消費を削減したい。

#### Acceptance Criteria
1. When `GraphicsCommandList`が追加された場合, the Graphics System shall 対象エンティティに`SurfaceGraphics`を作成し、`VisualGraphics`に登録する
2. If エンティティが`GraphicsCommandList`を持たない場合, then the Graphics System shall `SurfaceGraphics`の作成をスキップする
3. When `GraphicsCommandList`が削除された場合, the Graphics System shall 対応する`SurfaceGraphics`を解放し、`VisualGraphics`から登録解除する
4. The Graphics System shall Surface削除処理を専用システム（`cleanup_surface_on_commandlist_removed`）で実装する

### Requirement 2: Surface生成システムの一本化
**Objective:** 開発者として、Surface生成ロジックが一箇所に集約されることで、保守性と挙動の予測可能性を向上させたい。

#### Acceptance Criteria
1. The Graphics System shall `sync_surface_from_arrangement`システムを廃止する
2. The Graphics System shall `deferred_surface_creation_system`をSurface生成の唯一のシステムとする
3. The Graphics System shall `SurfaceGraphics`の生成トリガーを`GraphicsCommandList`の存在のみとする
4. When `Arrangement`が変更された場合 and `GraphicsCommandList`が存在しない場合, the Graphics System shall Surfaceを作成しない

### Requirement 3: DPIスケール対応のSurfaceサイズ計算
**Objective:** 開発者として、高DPI環境でも描画がはみ出さないことで、全てのディスプレイ設定で正確な描画を保証したい。

#### Acceptance Criteria
1. The Graphics System shall Surfaceサイズを`GlobalArrangement.bounds`から計算する（物理ピクセルサイズ）
2. When DPIスケールが100%以外の場合, the Graphics System shall スケール適用後のサイズでSurfaceを作成する
3. If `GlobalArrangement.bounds`のサイズが0の場合, then the Graphics System shall Surface生成をスキップする
4. When `GlobalArrangement`のサイズが変更された場合 and 既存の`SurfaceGraphics`が存在する場合, the Graphics System shall Surfaceを再作成する

### Requirement 4: 既存SurfaceGraphicsとの整合性維持
**Objective:** 開発者として、最適化導入後も既存の描画フローが正常に機能することで、リグレッションを防止したい。

#### Acceptance Criteria
1. While `SurfaceGraphics`が存在する状態で, the Graphics System shall 通常通りBeginDraw/EndDraw描画サイクルを実行する
2. When Surface生成がスキップされた場合, the Graphics System shall 子Visualの親Visual階層を正しく維持する
3. The Graphics System shall `VisualGraphics`（Visual階層管理）と`SurfaceGraphics`（描画バッファ管理）の独立性を維持する
4. If デバイスロストが発生した場合, then the Graphics System shall 最適化ロジックを含むすべてのSurfaceを正しく再初期化する

### Requirement 5: 診断とデバッグ支援
**Objective:** 開発者として、Surface生成の最適化状況を把握できることで、パフォーマンスチューニングとデバッグを効率化したい。

#### Acceptance Criteria
1. When Surface生成がスキップされた場合, the Graphics System shall ログにスキップ理由（CommandList不在等）を出力する
2. When Surfaceが作成された場合, the Graphics System shall ログにエンティティ名と物理ピクセルサイズを出力する
3. Where デバッグビルドが有効な場合, the Graphics System shall Surface生成統計（作成数、スキップ数）を提供する
