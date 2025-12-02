# Requirements Document

## Project Description (Input)

event-hit-test の孫仕様：名前付きヒット領域と多角形ヒットテスト

親仕様 `wintf-P0-event-system` の Requirement 2（ヒット領域定義）を実装する。
キャラクター画像上の特定領域（頭、胴体、手など）を定義し、部位ごとに異なる反応を実装できるようにする。

### 背景

デスクトップマスコットアプリケーションでは、キャラクターの部位ごとに異なる反応が必要である。
例えば、頭を撫でると喜ぶ、手を触ると手を振る、などの細かいインタラクションを実現するため、
矩形以外の形状で名前付きヒット領域を定義できる機能が必要。

### 対応する親仕様の要件

- **wintf-P0-event-system Requirement 2**: ヒット領域定義
  - 2.1: 矩形（Rectangle）によるヒット領域定義をサポート
  - 2.2: 多角形（Polygon）によるヒット領域定義をサポート
  - 2.3: 1つのエンティティに複数の名前付きヒット領域を定義できる
  - 2.4: ヒット領域の名前を含むイベント情報を提供
  - 2.5: ヒット領域定義を外部ファイル（JSON/YAML）から読み込める

### event-hit-test からの依存

本仕様は `event-hit-test` で実装されたヒットテストAPIを拡張する：

- `HitTestMode` enum に新しいバリアント追加（`NamedRegions` など）
- `hit_test` / `hit_test_detailed` の拡張または新規API
- `GlobalArrangement.transform` を使用したローカル座標変換

### 主な機能

- 矩形ヒット領域の定義と名前付け
- 多角形（Polygon）ヒット領域の定義と名前付け
- 1エンティティに対する複数ヒット領域のサポート
- ヒット結果に領域名を含める
- 外部ファイル（JSON）からのヒット領域定義読み込み
- ヒット領域の優先順位（重なり時の処理）

### 技術的考慮事項

- 多角形のヒット判定アルゴリズム（点の内外判定）
- ヒット領域データの効率的な保持
- DPI対応（物理ピクセル/DIP座標系の考慮）
- アニメーション対応（将来：フレームごとのヒット領域変更）

## Requirements
<!-- Will be generated in /kiro-spec-requirements phase -->

