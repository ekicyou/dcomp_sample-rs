# Requirements Document

## Introduction

wintf フレームワークにおいて、DPI処理と座標変換の「あるべき姿」が定義されていないため、物理ピクセルと論理座標（DIP）の混在が構造的な設計課題となっている。本仕様は、成熟したUIフレームワーク（WPF/WinUI3）の座標系モデルを参考に wintf が採用すべき座標変換アーキテクチャを定義し、現状（As-Is）との対比によりギャップと改善ロードマップを明確にしたレポートを成果物として作成する調査仕様である。コード修正は本仕様のスコープ外とし、後続仕様の設計判断材料を提供することを目的とする。

## Requirements

### Requirement 1: 座標系インベントリ（As-Is 調査）

**Objective:** 開発者として、システム内で使われている全ての座標系を一覧化したい。どの座標系がどのコンポーネント・関数で使われているかを把握し、不整合の根本原因を特定するため。

#### Acceptance Criteria

1. The survey shall 全ソースコード中の座標値を扱うコンポーネント・フィールドを列挙し、各フィールドの座標系（物理ピクセル / DIP / スクリーン座標 / クライアント座標 / ローカル座標）を分類した一覧表を作成する
2. The survey shall `BoxStyle.inset`（物理ピクセル）と `BoxStyle.size`（DIP）のように同一コンポーネント内で座標系が混在している箇所を特定し、各箇所の意図的設計か不整合かを判定する
3. The survey shall レガシー `WinState` trait の DPI 管理と ECS `DPI` コンポーネントについて、各々の使用箇所と責務の違いを文書化し、統合方針案を提示する（※ 統合の実装は本仕様のスコープ外）
4. The survey shall Win32 API 呼び出し（`SetWindowPos`, `GetCursorPos`, `AdjustWindowRectExForDpi` 等）の入出力座標系をすべて特定し、DPI Awareness コンテキスト（Per-Monitor v2）下での挙動を文書化する

### Requirement 2: DPI 値フロー追跡（As-Is 調査）

**Objective:** 開発者として、DPI値がシステム内をどう流れるかの完全なデータフロー図を得たい。DPI値の取得・保持・伝播・消費の各段階での変換漏れや固定化バグを発見するため。

#### Acceptance Criteria

1. The survey shall DPI値の取得元（`GetDpiForWindow`, `GetDpiForMonitor`, `GetDpiForSystem`, `WM_DPICHANGED`）から最終消費先（`Visual.SetOffsetX/Y`, `Surface.BeginDraw`, `SetWindowPos`）までの全経路を追跡し、データフロー図として文書化する
2. The survey shall `DpiChangeContext`（スレッドローカル）を経由する `WM_DPICHANGED` → `WM_WINDOWPOSCHANGED` 間の同期伝達パスにおいて、DPI値が正しく受け渡されることを検証する
3. The survey shall `DPI` コンポーネントが `Arrangement.scale` に反映されるまでの変換チェーン（`update_arrangements_system` → `propagate_global_arrangements` → `GlobalArrangement`）の各段階でのスケール値を追跡し、二重スケーリングの有無を特定する
4. The survey shall Monitor の DPI (`Monitor.dpi`) が ECS ツリーの `Arrangement.scale` に反映されていない現状について、影響範囲を評価する

### Requirement 3: ドラッグ座標変換チェーン評価（As-Is / To-Be 対比）

**Objective:** 開発者として、ドラッグ操作時の座標変換チェーンを「あるべき姿」と対比して評価したい。現行の変換チェーンが To-Be アーキテクチャとどこでコンフリクトするかを明らかにし、改善の優先箇所を特定するため。

#### Acceptance Criteria

1. The survey shall ドラッグイベント発生（`DragEvent.delta`）から `SetWindowPos` 呼び出しまでの現行変換チェーンを追跡し、各ステップでの座標系を文書化する。その上で、To-Be アーキテクチャ（Req 4）に基づく理想的な変換チェーンを併記し、コンフリクト箇所を指摘する
2. The survey shall 現行の `BoxStyle.inset`（物理ピクセル）→ Taffy → `Arrangement` → `GlobalArrangement` の変換チェーンについて、To-Be（DIP統一）に移行した場合の変換チェーンとの差異を評価する
3. The survey shall `sync_window_arrangement_from_window_pos`（現在無効化中）の設計意図を文書化し、To-Be アーキテクチャにおいて有効化すべきか・不要かを評価する

### Requirement 4: あるべき座標変換アーキテクチャ（To-Be 設計指針）

**Objective:** 開発者として、座標変換の理想アーキテクチャの設計指針を得たい。WPF/WinUI3 等の成熟したフレームワークの設計パターンを参考に、wintf が採用すべき座標系モデルを定義するため。

#### Acceptance Criteria

1. The survey shall WPF の「デバイス独立ユニット（DIP）統一モデル」および WinUI3 の DPI スケーリングモデルを調査し、wintf に適用可能な設計パターンを提示する
2. The survey shall 「全ての内部座標を DIP で統一し、物理ピクセルへの変換は出力層（Win32 API 呼び出し、DirectComposition Visual 設定）でのみ行う」方針の妥当性を評価する
3. The survey shall `BoxStyle` コンポーネントにおける `inset`（位置）と `size`（サイズ）の座標系統一方針を提示する。統一する場合に必要な変更箇所と影響範囲を列挙する
4. The survey shall Window エンティティの `Arrangement.offset` と `WindowPos.position` の関係を再定義する設計指針を提示する。特に `sync_window_arrangement_from_window_pos` を有効化するための前提条件を明確にする
5. Where Per-Monitor DPI v2 環境でウィンドウがモニタ間を移動する場合, the survey shall DPI 変更時の座標再計算フローの理想的な処理順序を定義する

### Requirement 5: ギャップ分析と優先度付き改善提案

**Objective:** 開発者として、現状（As-Is）と理想（To-Be）のギャップを一覧化し、改善の優先度を知りたい。限られた開発リソースで最大のインパクトを得るためのロードマップを策定するため。

#### Acceptance Criteria

1. The survey shall 各ギャップ項目に対して「影響度（High/Medium/Low）」「修正コスト（High/Medium/Low）」「ブロックしている仕様」を評価し、優先度マトリクスを作成する
2. The survey shall 現状（As-Is）から To-Be アーキテクチャへの移行ロードマップを提示する。段階的移行（既存機能を壊さず漸進的に改善）と一括移行のアプローチを比較し、それぞれのリスクとコストを評価する
3. The survey shall `dpi-propagation`（完了済み）で実装された基盤と、`wintf-P1-dpi-scaling`（バックログ）の要件との差分を明確にし、P1 仕様の要件が現実的かを再評価する

### Requirement 6: 成果物としてのレポート

**Objective:** 開発者として、調査結果を単一の構造化されたレポートとして参照したい。今後の DPI 関連仕様（`wintf-P1-dpi-scaling` 等）の設計判断の根拠資料として利用するため。

#### Acceptance Criteria

1. The survey shall 最終成果物として `.kiro/specs/dpi-coordinate-transform-survey/report.md` に調査レポートを出力する
2. The survey shall レポートに以下のセクションを含める: (a) エグゼクティブサマリー, (b) 座標系インベントリ, (c) DPI データフロー図, (d) ドラッグ座標変換チェーン評価（As-Is / To-Be 対比）, (e) To-Be アーキテクチャ設計指針, (f) ギャップ分析マトリクス, (g) 改善ロードマップ
3. The survey shall レポート内のフロー図・依存関係図に Mermaid 記法を使用し、一覧表には Markdown table を使用する。また、コードベースの具体的なファイルパス・行番号への参照を含める
4. The survey shall レポートの各改善提案に対して、関連する既存仕様（`dpi-propagation`, `wintf-P1-dpi-scaling`, `event-drag-system`）へのクロスリファレンスを付与する

