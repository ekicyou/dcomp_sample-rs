# Requirements Document

## Project Description (Input)

taffyを導入し、レイアウト構造の基礎を実現する。
1. 名称変更「BoxStyle,BoxComputedLayout」⇒「TaffyStyle,TaffyComputedLayout」
2. TaffyStyleを組み立てるためのコンポーネント（positionなどの内部要素をジャンル別に分解したもの、または分解せずに利用？）
3. (2)で用意した個別要素をTaffyStyleに組み立てるシステム
4. widgetツリーからTaffyStyleを組み立てて、taffyにツリー計算させるシステム。出力はBoxComputedLayoutに行う
5. 更新されたBoxComputedLayoutより、Arrangementを更新するシステム
6. レイアウト計算を増分的に行い、毎回全部やり直すことがないようにする仕組みの構築。
7. その他taffyレイアウトを構成するために必要なシステム。

## Requirements

### Requirement 1: コンポーネント名称変更
**目的**: taffyライブラリとの統合を明確化し、既存の`BoxStyle`と`BoxComputedLayout`をtaffyの用途を反映した名称に変更する。

#### 受入基準
1. レイアウトシステムは、`BoxStyle`を`TaffyStyle`に名称変更すること
2. レイアウトシステムは、`BoxComputedLayout`を`TaffyComputedLayout`に名称変更すること
3. レイアウトシステムは、`TaffyComputedLayout`が`#[repr(transparent)]`属性を持ち、メモリレイアウトが内部の`Layout`と一致すること
4. レイアウトシステムは、`TaffyComputedLayout`の`Default`トレイト実装が内部の`Layout::default()`を利用すること
5. レイアウトシステムは、`TaffyComputedLayout`が`Layout`が実装するトレイト（`Clone`、`Debug`、`PartialEq`、`Copy`等）を実装すること
6. レイアウトシステムは、名称変更後も既存のECSコンポーネントとして機能すること
7. レイアウトシステムは、名称変更後も既存のテストがすべて通ること

### Requirement 2: TaffyStyleコンポーネント構造
**目的**: taffyの`Style`構造を構築するためのECSコンポーネントを設計し、宣言的なレイアウト記述を可能にする。ただし、taffyの詳細はwintfの内部実装として隠蔽する。

#### 受入基準
1. レイアウトシステムは、`TaffyStyle`がtaffyの`Style`構造体をラップするコンポーネントとして存在すること
2. レイアウトシステムは、`TaffyStyle`が`#[repr(transparent)]`属性を持ち、メモリレイアウトが内部の`Style`と一致すること
3. レイアウトシステムは、`TaffyStyle`の`Default`トレイト実装が内部の`Style::default()`を利用すること
4. レイアウトシステムは、`TaffyStyle`が`Style`が実装するトレイト（`Clone`、`Debug`、`PartialEq`等）を実装すること
5. レイアウトシステムは、`TaffyStyle`を内部実装として扱い、公開APIでは直接露出しないこと
6. レイアウトシステムは、UIウィジェット向けの高レベルレイアウトプロパティ（例: `BoxSize`、`BoxMargin`、`BoxPadding`、`FlexContainer`、`FlexItem`）をECSコンポーネントとして提供すること
7. When `TaffyStyle`がデフォルト値（`Style::default()`）の場合、レイアウトシステムは、親のレイアウトに従う子要素として正しく計算すること
8. レイアウトシステムは、`TaffyStyle`のデフォルト値が明示的に設定されていないエンティティに対して、レイアウト計算を正常に実行すること

### Requirement 3: 高レベルレイアウトコンポーネント
**目的**: taffyを隠蔽し、wintfユーザーが直感的に使用できる高レベルレイアウトコンポーネントを提供する。今回のスコープでは、基本的なFlexboxレイアウトに必要な最小限のコンポーネントを実装する。UIフレームワークとして、変更がない時間が大半を占めることを前提に、変更検知ベースの効率的な更新を行う。コンポーネントは同時に設定される可能性が高いプロパティをグループ化し、適切な粒度で設計する。taffyの共通型（`Dimension`、`LengthPercentage`等）はwintfでre-exportし、ユーザーが直接taffyをuseせずに利用できるようにする。フィールド名はtaffyの命名規則に従い、特に理由がない限りtaffyと同じ名前を使用する（例: `width`、`height`、`left`、`right`、`top`、`bottom`、`direction`、`justify_content`、`align_items`等）。

#### 受入基準
1. レイアウトシステムは、サイズ指定用の`BoxSize`コンポーネントを提供すること
2. レイアウトシステムは、`BoxSize`が以下のフィールドを持つこと: `width`、`height`（それぞれ`Option<Dimension>`型で、`None`は未指定を意味し、taffyのフィールド名と一致させる）
3. レイアウトシステムは、`Dimension`型（taffy由来）をre-exportし、ピクセル値（`Px(f32)`）、パーセント値（`Percent(f32)`）、自動サイズ（`Auto`）をサポートすること
4. レイアウトシステムは、余白指定用の`BoxMargin`コンポーネントを提供すること
5. レイアウトシステムは、`BoxMargin`が`Rect<LengthPercentageAuto>`型（taffy由来をre-export）で、`left`、`right`、`top`、`bottom`フィールドを持つこと
6. レイアウトシステムは、内部余白指定用の`BoxPadding`コンポーネントを提供すること
7. レイアウトシステムは、`BoxPadding`が`Rect<LengthPercentage>`型（taffy由来をre-export）で、`left`、`right`、`top`、`bottom`フィールドを持つこと
8. レイアウトシステムは、Flexboxコンテナー設定用の`FlexContainer`コンポーネントを提供すること
9. レイアウトシステムは、`FlexContainer`が以下のフィールドを持つこと: `direction`（`FlexDirection`、taffy由来をre-export）、`justify_content`（`Option<JustifyContent>`、taffy由来をre-export）、`align_items`（`Option<AlignItems>`、taffy由来をre-export）
10. レイアウトシステムは、Flexboxアイテム設定用の`FlexItem`コンポーネントを提供すること
11. レイアウトシステムは、`FlexItem`が以下のフィールドを持つこと: `grow`（`f32`）、`shrink`（`f32`）、`basis`（`Dimension`）、`align_self`（`Option<AlignSelf>`、taffy由来をre-export）
12. レイアウトシステムは、レイアウト関連の共通型（`Dimension`、`LengthPercentage`、`LengthPercentageAuto`、`Rect`、`FlexDirection`、`JustifyContent`、`AlignItems`、`AlignSelf`等）をwintfの公開APIでre-exportし、ユーザーが`use taffy::`を記述せずに利用できるようにすること
13. When高レベルレイアウトコンポーネントが変更された場合のみ、レイアウトシステムは、該当エンティティの内部`TaffyStyle`を再構築すること
14. レイアウトシステムは、ECSの`Changed<T>`クエリを使用して変更検知を行い、変更されたエンティティのみを処理すること
15. レイアウトシステムは、高レベルコンポーネントから`TaffyStyle`への変換を専用のECSシステムで実行すること
16. If高レベルコンポーネントが指定されていない場合、レイアウトシステムは、デフォルト値を使用すること
17. レイアウトシステムは、未対応のtaffy機能（`Gap`、`Position`、`MinSize`、`MaxSize`等）については将来の拡張に備えて設計を拡張可能にすること

### Requirement 4: Taffyレイアウト計算システム
**目的**: Widgetツリーを走査して`TaffyStyle`からtaffyのレイアウトツリーを構築し、計算結果を`TaffyComputedLayout`に出力する。taffyのレイアウトアルゴリズムとキャッシュ機構はブラックボックスとして扱い、wintf側は値の転送と結果の読み取りのみを管理する。

#### 受入基準
1. When `TaffyStyle`が変更された場合、レイアウトシステムは、`taffy.set_style(node_id, style)`を呼び出してtaffyに変更を通知すること
2. レイアウトシステムは、ECS階層構造（`ChildOf`リレーション）をtaffyのツリー構造（`add_child`等のAPI）に同期すること
3. レイアウトシステムは、taffyのレイアウト計算（ダーティトラッキング、キャッシュ、アルゴリズム）をブラックボックスとして扱い、内部実装の詳細に依存しないこと
4. レイアウトシステムは、`taffy.compute_layout(root_node, available_space)`を呼び出してレイアウト計算を実行すること
5. When taffyがレイアウト計算を完了した場合、レイアウトシステムは、`taffy.layout(node_id)`で結果を取得し、各エンティティの`TaffyComputedLayout`を更新すること
6. レイアウトシステムは、`TaffyComputedLayout`にレイアウト位置（x、y）とサイズ（width、height）を格納すること
7. レイアウトシステムは、taffy計算結果を直接使用し、wintf側で独自の検証や調整を行わないこと
8. If レイアウト計算中にエラーが発生した場合、レイアウトシステムは、エラーをログに記録し、デフォルトレイアウトを適用すること

### Requirement 5: Arrangement更新システム
**目的**: `TaffyComputedLayout`の計算結果を`Arrangement`コンポーネントに反映し、既存のグラフィックスシステムと連携する。

#### 受入基準
1. When `TaffyComputedLayout`が更新された場合、レイアウトシステムは、対応する`Arrangement`コンポーネントを更新すること
2. レイアウトシステムは、`TaffyComputedLayout`のレイアウト位置を`Arrangement.offset`に変換すること
3. レイアウトシステムは、`TaffyComputedLayout`のレイアウトサイズを`Arrangement.size`に変換すること
4. レイアウトシステムは、既存の`propagate_global_arrangements`システムと互換性を保つこと
5. When `Arrangement`が更新された場合、レイアウトシステムは、`ArrangementTreeChanged`マーカーを設定すること

### Requirement 6: 増分レイアウト計算
**目的**: レイアウト計算を増分的に行い、変更されたサブツリーのみを再計算することで、パフォーマンスを最適化する。taffyが内蔵するキャッシュ機構とダーティトラッキング機能を活用し、wintf側は変更検知と値の転送のみを担当する。UIフレームワークとして、変更がない時間が大半を占めることを前提に、変更時のみの転送で効率化する。

#### 受入基準
1. When高レベルレイアウトコンポーネント（`BoxSize`, `BoxMargin`, `BoxPadding`, `FlexContainer`, `FlexItem`）のいずれかが変更された場合、レイアウトシステムは、ECSの`Changed<T>`クエリで変更を検知し、該当エンティティの`TaffyStyle`を更新すること
2. レイアウトシステムは、変更されたエンティティのみを対象に`TaffyStyle`をtaffyに転送し、変更がないエンティティは処理をスキップすること
3. When `TaffyStyle`が変更された場合、レイアウトシステムは、`taffy.set_style(node_id, style)`を呼び出してtaffyに変更を通知すること
4. レイアウトシステムは、`taffy.set_style()`呼び出しによってtaffy内部で自動的にダーティマークが設定され、親ノードへ再帰的に伝播されることに依存すること
5. レイアウトシステムは、taffyの内蔵キャッシュ機構（`Cache`構造）に依存し、キャッシュヒット時の計算スキップはtaffy内部で自動的に処理されることを前提とすること
6. レイアウトシステムは、影響範囲の決定、親や兄弟への伝播、計算の最適化などをwintf側で独自に実装しないこと
7. When 変更検知システムがいずれかのエンティティで変更を検知した場合、レイアウトシステムは、`taffy.compute_layout(root_node, available_space)`を呼び出すこと（taffy内部でダーティノードのみが再計算される）
8. When 変更が検知されなかった場合、レイアウトシステムは、`taffy.compute_layout()`の呼び出しをスキップすること
9. While レイアウトシステムが初期化中の場合、レイアウトシステムは、全ツリーを計算すること

### Requirement 7: Taffyレイアウトインフラストラクチャ
**目的**: taffyレイアウトシステムを構成するために必要なインフラストラクチャを整備する。

#### 受入基準
1. レイアウトシステムは、taffyの`Taffy`インスタンスをECSリソースとして管理すること
2. レイアウトシステムは、ECSエンティティとtaffyノードIDのマッピングを管理すること（実装方法は設計フェーズで決定）
3. レイアウトシステムは、エンティティからtaffyノードIDへの検索を効率的に行えること
4. When エンティティが削除された場合、レイアウトシステムは、対応するtaffyノードを削除すること
5. レイアウトシステムは、taffyレイアウト計算の実行順序を制御するためのシステムセットを定義すること
6. レイアウトシステムは、`ecs/layout/systems.rs`に既存のArrangement伝播システムとの統合を実装すること
7. レイアウトシステムは、ルートウィンドウのサイズ変更をtaffyに通知すること
8. レイアウトシステムは、デバッグ用のレイアウト可視化機能を提供すること（オプション）

### Requirement 8: ビルドおよび動作検証
**目的**: taffyレイアウト統合後も既存のビルドとサンプルアプリケーションが正常に動作することを保証する。既存の`simple_window.rs`を新しい高レベルレイアウトコンポーネントに移行し、レイアウトシステムの動作を検証する。

#### 受入基準
1. レイアウトシステムは、`cargo build --all-targets`コマンドが正常に完了すること
2. レイアウトシステムは、`cargo test --all-targets`コマンドが正常に完了すること
3. レイアウトシステムは、`simple_window.rs`の手動`Arrangement`設定を高レベルレイアウトコンポーネント（`BoxSize`、`BoxMargin`、`BoxPadding`、`FlexContainer`、`FlexItem`）による宣言的な記述に移行すること
4. レイアウトシステムは、移行後の`simple_window.rs`で`Arrangement`コンポーネントを手動設定せず、レイアウト計算システムが自動的に`Arrangement`を更新することを検証すること
5. When `simple_window.rs`の移行版を実行した場合、移行前と同じビジュアル結果が得られること
6. When `simple_window.rs`で高レベルコンポーネント（例: `BoxSize`、`BoxMargin`）を動的に変更した場合、レイアウトが正しく再計算され、画面に反映されること
7. レイアウトシステムは、`simple_window.rs`の階層構造（Window → Rectangle → Rectangle → Label）で、親子関係に基づくレイアウト計算が正しく機能することを検証すること

### Requirement 9: ユニットテストによる統合品質保証
**目的**: taffyレイアウト統合のwintf側の責務（入力パラメーター転送、変更検知、出力変換）を網羅的にテストし、回帰を防止する。taffyの内部実装（レイアウトアルゴリズム、キャッシュ機構）はtaffy側のスコープとしてテスト対象外とする。

#### 受入基準
1. レイアウトシステムは、高レベルコンポーネント（`BoxSize`、`BoxMargin`、`BoxPadding`、`FlexContainer`、`FlexItem`）から`TaffyStyle`への変換が正確であることをユニットテストで検証すること
2. レイアウトシステムは、各高レベルコンポーネントのすべてのフィールドが`TaffyStyle`に正しく反映されることをテストすること（変換漏れの検出）
3. レイアウトシステムは、ECS階層変更（エンティティ追加、削除、`ChildOf`変更）時にtaffyツリーが正しく同期されることをユニットテストで検証すること
4. レイアウトシステムは、増分計算の変更検知が正しく機能し、以下のシナリオで`compute_layout()`が呼び出されること/されないことをテストすること:
   - 高レベルコンポーネントが変更された場合: 呼び出される
   - 高レベルコンポーネントが変更されていない場合: 呼び出されない
   - `ChildOf`リレーションが変更された場合: 呼び出される
5. レイアウトシステムは、`TaffyComputedLayout`から`Arrangement`への座標変換（位置、サイズ）が正確であることをユニットテストで検証すること
6. レイアウトシステムは、境界値シナリオ（空ツリー、単一ノード、深い階層、多数の兄弟ノード）でクラッシュせず正常動作することをテストすること
7. レイアウトシステムは、エンティティ削除時にtaffyノードが正しくクリーンアップされることをテストすること（メモリリーク防止）
8. レイアウトシステムは、既存のテスト規約（`tests/`ディレクトリ配置、`*_test.rs`命名、`#[test]`属性）に従ってテストを実装すること
9. レイアウトシステムは、各テストケースが独立して実行可能であり、テスト間で状態が共有されないことを保証すること
