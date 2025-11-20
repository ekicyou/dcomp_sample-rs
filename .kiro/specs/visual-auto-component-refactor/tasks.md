# Implementation Plan - Visual Auto Component Refactor

- [ ] 1. **論理Visualコンポーネントの定義**
  - `components.rs`に`is_visible`、`opacity`、`transform`フィールドを持つ`Visual`コンポーネント構造体を定義する。
  - `Visual`コンポーネントに`Default`トレイトを実装する。
  - ECSアプリ設定にコンポーネントを登録する。
  - _Requirements: R1, R2_

- [ ] 2. **リソース管理システムの実装 (Systemsアプローチ)**
  - リソース管理システムを格納する新しいモジュール`visual_manager.rs`を作成する。
  - `Added<Visual>`を持つエンティティをクエリするシステムを実装する。
  - `GraphicsCore`にアクセスして`IDCompositionVisual`および`IDCompositionSurface`リソースを作成する。
  - リソース作成前にデバイスの有効性チェックを処理する。
  - `VisualGraphics`および`SurfaceGraphics`コンポーネントをエンティティに挿入する。
  - **注意:** パニック回避のため、エンティティの存在確認 (`world.get_entity(e).is_ok()`) を行うか、安全なコマンド発行を行うこと。
  - _Requirements: R1.5, R2, R4_

- [ ] 3. **ウィンドウ統合処理の実装 (重要)**
  - `visual_manager.rs` または `systems.rs` に、`WindowGraphics` と `VisualGraphics` の紐付けを行うシステムを実装する。
  - `Query<(Entity, &WindowGraphics, &VisualGraphics), Changed<VisualGraphics>>` 等を用いて、Visualが生成されたタイミングを検知する。
  - `WindowGraphics` (CompositionTarget) の `SetRoot` メソッドを呼び出し、生成されたVisualをルートとして設定する。
  - これを行わないと画面が描画されないため、確実に実装する。
  - _Requirements: R5.3_

- [ ] 4. **既存コードのリファクタリング**
  - `systems.rs` 内の `init_window_visual` 等で `VisualGraphics` を直接生成しているコードを削除する。
  - 代わりに `Visual` コンポーネントを付与する処理に変更する。
  - `init_window_surface` 等のSurface生成処理も、自動管理システムに委譲できるか確認し、重複を排除する。
  - _Requirements: R5_

- [ ] 5. **動作検証**
  - アプリケーションを起動し、ウィンドウが表示され、コンテンツ（矩形やテキスト）が描画されることを確認する。
  - ウィンドウを閉じた際にパニックが発生しないことを確認する。
  - _Requirements: R5.3_

- [ ] 6. **(Optional) Hooksアプローチの実装と比較**
  - Systemsアプローチで安定動作確認後、Hooks (`on_add`) を用いた実装を試行する。
  - 比較結果に基づき、最終的な採用案を決定する。
