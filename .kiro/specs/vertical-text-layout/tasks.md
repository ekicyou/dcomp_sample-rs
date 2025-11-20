# Implementation Plan

## Task Format Template

### Major task only
- [x] 1. コアデータ構造の定義
  - `TextDirection` enumを定義し、Horizontal (LTR/RTL) と Vertical (RL/LR) モードのバリアントを含める。
  - `Label` コンポーネントを更新し、`direction` フィールドを追加する。
  - `TextLayoutMetrics` コンポーネントを作成し、計算された物理的な幅と高さを保持する。
  - _Requirements: 1.1, 1.2, 4.1_

- [x] 2. レンダリングおよびレイアウトロジックの実装 (P)
  - `draw_labels` システムを更新し、`Label.direction` を読み取るようにする。
  - DirectWriteの `IDWriteTextFormat` に適切な `SetReadingDirection` と `SetFlowDirection` の値を設定する。
  - `IDWriteTextLayout::GetMetrics` を使用してテキストメトリクスを取得する。
  - 物理座標系の一貫性を保つため、縦書き方向の場合に幅と高さを入れ替えるロジックを実装する。
  - `TextLayoutMetrics` コンポーネントに値を設定し、エンティティに挿入する。
  - _Requirements: 1.3, 2.1, 2.2, 3.1, 3.2, 4.2_

- [x] 3. 検証用サンプルの更新 (P)
  - `simple_window.rs` を拡張し、検証用の新しいUIコンテナを含める。
  - 縦書き（右から左）に設定された `Label` を追加する。
  - 横書き（RTL）に設定された `Label` を追加する。
  - 新しい要素が既存のコンテンツと重ならないように正しく配置されていることを確認する。
  - _Requirements: 5.1, 5.2_
