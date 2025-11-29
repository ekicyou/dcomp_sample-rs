# 調査・設計判断ログ

---
**機能**: `taffy-layout-integration`
**調査範囲**: Extension（既存システム拡張）
**主な発見事項**:
  - taffyの高レベルAPI（TaffyTree）が内蔵キャッシュと増分計算を提供
  - wintfは既に階層伝播システム（Arrangement + GlobalArrangement）を持つ
  - 透過的なラッパーコンポーネント（`#[repr(transparent)]`）パターンが確立済み
---

## 調査ログ

### taffy APIとキャッシュ機構

- **背景**: taffy 0.9.1の増分計算とキャッシュ戦略を理解する必要があった
- **調査情報源**: 
  - https://docs.rs/taffy/latest/taffy/ - 公式ドキュメント
  - TaffyTreeの高レベルAPIドキュメント
- **発見事項**:
  - TaffyTree::compute_layout()は内部でダーティトラッキングとキャッシュを自動管理
  - set_style()呼び出しで自動的にノードをダーティマーク（親へ再帰伝播）
  - wintf側は変更検知と値転送のみ担当すればよい
  - キャッシュヒット時の計算スキップはtaffy内部で自動処理
- **影響**:
  - **設計判断**: wintf側でダーティ伝播ロジックを実装しない
  - **責務分離**: wintf = 変更検知・値転送、taffy = レイアウト計算・キャッシュ
  - **増分計算戦略**: ECS Changed<T>クエリで高レベルコンポーネント変更を検知 → TaffyStyleを更新 → taffy.set_style()呼び出し → compute_layout()実行

### 既存のArrangement階層伝播システム

- **背景**: 既存のLayout Systemとの統合方法を決定する必要があった
- **調査情報源**: 
  - `crates/wintf/src/ecs/layout/systems.rs` - 既存システム実装
  - `crates/wintf/src/ecs/common/tree_system.rs` - 汎用階層伝播
  - `crates/wintf/examples/simple_window.rs` - 現在の使用例
- **発見事項**:
  - Arrangement（ローカル）+ GlobalArrangement（ワールド座標）の二層構造
  - ChildOf + Children によるECS階層構造
  - propagate_global_arrangementsシステムが既に存在
  - Arrangementは手動設定されており、レイアウト計算なし
- **影響**:
  - **統合戦略**: TaffyComputedLayoutの計算結果をArrangementに反映
  - **システム順序**: taffy計算 → Arrangement更新 → GlobalArrangement伝播
  - **後方互換性**: 既存のpropagation systemをそのまま利用可能

### 高レベルコンポーネント設計

- **背景**: taffyを隠蔽し、ユーザー向けの直感的なAPIを提供する必要があった
- **調査情報源**:
  - taffyのStyle構造体フィールド定義
  - Webフレームワーク（React Native, Flutter）のレイアウトAPI
  - wintfの既存コンポーネント命名規則
- **発見事項**:
  - taffyは豊富な型定義を提供（Dimension, LengthPercentage等）
  - 同時変更が多いプロパティをグループ化すると変更検知効率が向上
  - BoxSize, BoxMargin, BoxPaddingは独立して変更される可能性が高い
  - FlexContainer（コンテナー）とFlexItem（アイテム）は役割が異なる
- **影響**:
  - **コンポーネント分割**: サイズ・余白・Flexを別コンポーネントに
  - **型のre-export**: ユーザーがuse taffy::を書かなくて済むようにする
  - **フィールド名統一**: taffyと同じ名称（width, height, direction等）を使用
  - **Option<T>利用**: 未指定時はtaffyデフォルトにフォールバック

### ECSとtaffyツリーのマッピング

- **背景**: EntityとtaffyノードIDの双方向マッピング管理方法を決定する必要があった
- **調査情報源**:
  - bevy_ecsのEntityの型定義と特性
  - taffyのNodeId型（Copy, Hash, Eq実装済み）
  - Rustの標準コレクション（HashMap）
- **発見事項**:
  - EntityはCopy + Hash + Eq実装済みでHashMapのキーとして利用可能
  - NodeIdもCopy + Hash + Eq実装済み
  - 双方向マッピングが必要（Entity → NodeId、NodeId → Entity）
  - エンティティ削除時のクリーンアップが必須
- **影響**:
  - **マッピング戦略**: HashMap<Entity, NodeId> + HashMap<NodeId, Entity>の二つを保持
  - **リソース管理**: TaffyTreeとマッピングを単一のECSリソースに格納
  - **削除ハンドリング**: RemovedComponents<T>で検知し、taffyノードも削除

### エラーハンドリング戦略

- **背景**: taffyのcompute_layout()がResult<(), TaffyError>を返す
- **調査情報源**:
  - taffyのエラー型定義
  - wintfの既存エラーハンドリングパターン
- **発見事項**:
  - taffyのエラーは主にノードIDの不整合（存在しないノード参照）
  - 正常な使用ではエラーは発生しない（防御的プログラミングで回避可能）
  - ログ出力とデフォルト値適用で継続運用可能
- **影響**:
  - **エラー処理**: エラー発生時はログ出力 + デフォルトレイアウト適用
  - **防御戦略**: エンティティ削除時に必ずtaffyノードも削除
  - **パニック回避**: システムはエラーでパニックせず、継続実行

## アーキテクチャパターン評価

### 選択肢1: タイトカップリング（taffy直接露出）
- **説明**: TaffyStyleを公開APIとしてそのまま利用
- **利点**: 実装が単純、taffyの全機能に即座にアクセス可能
- **欠点**: taffy依存が公開APIに漏れる、将来の差し替えが困難、学習コストが高い
- **評価**: ❌ ライブラリAPIとして不適切

### 選択肢2: ファサードパターン（高レベルコンポーネント）
- **説明**: taffy詳細を隠蔽した高レベルコンポーネントを提供
- **利点**: ユーザーフレンドリー、taffy依存を内部実装に隔離、段階的機能追加が容易
- **欠点**: ラッパーレイヤーの実装コスト、全機能の即時提供は不可
- **評価**: ✅ **採用** - ライブラリAPIとして最適、長期保守性が高い

### 選択肢3: ハイブリッド（両方を公開）
- **説明**: 高レベルコンポーネントと低レベルTaffyStyleの両方を公開
- **利点**: 柔軟性が高い、上級ユーザーは直接taffyにアクセス可能
- **欠点**: API複雑化、一貫性の欠如、メンテナンス負担増
- **評価**: △ 将来の拡張オプションとして保留

## 設計判断

### 判断: 高レベルコンポーネント優先、TaffyStyleは内部実装

- **背景**: Requirements 2と3が、taffy隠蔽と高レベルAPIの提供を要求
- **検討した代替案**:
  1. TaffyStyleを直接公開 - シンプルだが学習コストが高い
  2. 高レベルコンポーネントのみ公開 - ユーザーフレンドリーだが実装コストあり
- **選択したアプローチ**: ファサードパターン（選択肢2）
  - BoxSize, BoxMargin, BoxPadding, FlexContainer, FlexItemを公開
  - TaffyStyleは内部実装として隠蔽
  - taffy型（Dimension等）はre-exportでユーザー利用可能にする
- **根拠**:
  - steering/product.mdが「日本語縦書きを必要とするデスクトップアプリ開発者」をターゲットに設定
  - Flexboxを直接理解しているユーザーは少数派
  - UIフレームワークとしての使いやすさを優先
  - 将来のレイアウトエンジン差し替え（例: Morphorm）の余地を残す
- **トレードオフ**:
  - **利点**: 直感的なAPI、taffy依存の隔離、段階的機能拡張が可能
  - **欠点**: ラッパー実装コスト、taffyの全機能に即座にアクセスできない
- **フォローアップ**: 
  - 未対応機能（Gap, Position, MinSize等）のニーズを使用状況から判断
  - v2で追加の高レベルコンポーネントを検討

### 判断: 変更検知ベースの増分計算

- **背景**: Requirement 6が増分レイアウト計算を要求
- **検討した代替案**:
  1. 毎フレーム全ツリー計算 - シンプルだが非効率
  2. wintf側で独自のダーティトラッキング - 複雑で重複実装
  3. Changed<T>クエリ + taffyの内蔵キャッシュ活用 - 効率的かつシンプル
- **選択したアプローチ**: 選択肢3
  - ECSのChanged<BoxSize>, Changed<BoxMargin>等で変更検知
  - 変更があった場合のみTaffyStyleを再構築し、taffy.set_style()呼び出し
  - taffy内部のダーティ伝播とキャッシュに依存
  - 変更がない場合はcompute_layout()をスキップ
- **根拠**:
  - steering/product.mdが「デスクトップマスコットアプリケーション」を主用途に設定
  - UIは大半の時間静止しており、変更は稀
  - 変更検知ベースの設計が最も効率的
  - taffyが既にキャッシュ機構を持つため、重複実装を避ける
- **トレードオフ**:
  - **利点**: 静止時のCPU負荷ゼロ、taffyのキャッシュを最大活用
  - **欠点**: 初回構築時のオーバーヘッド（許容範囲）
- **フォローアップ**: 
  - パフォーマンステストで変更検知のオーバーヘッドを検証
  - 大規模ツリーでの挙動を確認

### 判断: EntityとNodeIdのHashMapマッピング

- **背景**: Requirement 7がECSエンティティとtaffyノードIDのマッピング管理を要求
- **検討した代替案**:
  1. Entityに直接NodeIdをComponentとして持たせる - 高速だが削除時の整合性管理が複雑
  2. HashMap<Entity, NodeId>の単方向マッピング - NodeId → Entity逆引きが低速
  3. 双方向HashMap - メモリは増えるが双方向アクセスが高速
- **選択したアプローチ**: 選択肢3
  - TaffyLayoutResourceに以下を格納:
    - taffy: TaffyTree
    - entity_to_node: HashMap<Entity, NodeId>
    - node_to_entity: HashMap<NodeId, Entity>
  - 追加・削除時に両方のマップを同期更新
- **根拠**:
  - taffy.layout(node_id)の結果をEntityに反映する際、NodeId → Entity逆引きが頻繁
  - HashMap検索はO(1)で十分高速
  - メモリ増加は微小（各エンティティあたり数十バイト）
  - 実装の明確さとメンテナンス性を優先
- **トレードオフ**:
  - **利点**: 双方向アクセスが高速、実装が明確
  - **欠点**: メモリ使用量が若干増加（エンティティ数に比例）
- **フォローアップ**: 
  - 大規模ツリー（1000+エンティティ）でのメモリ使用量を測定
  - 必要に応じてslotmapやgenerationカウンター最適化を検討

### 判断: ルートウィンドウサイズをavailable_spaceとして使用

- **背景**: Requirement 7がルートウィンドウのサイズ変更をtaffyに通知することを要求
- **検討した代替案**:
  1. 固定サイズ（無限大）を使用 - 簡単だがウィンドウ境界を考慮しない
  2. Windowコンポーネントのサイズを取得 - 正確だが結合度が高い
  3. available_spaceパラメータで動的に渡す - 柔軟性が高い
- **選択したアプローチ**: 選択肢3
  - compute_layoutシステムでルートWindowのサイズを取得
  - Size::from_lengths(width, height)でavailable_spaceを構築
  - taffy.compute_layout(root_node, available_space)に渡す
- **根拠**:
  - taffyのcompute_layout APIがavailable_spaceを必須パラメータとして要求
  - ウィンドウサイズ変更時の再レイアウトが自動的に動作
  - Windowコンポーネントは既にサイズ情報を持つ
- **トレードオフ**:
  - **利点**: 正確なレイアウト計算、ウィンドウリサイズ対応
  - **欠点**: ルートウィンドウ検索のオーバーヘッド（許容範囲）
- **フォローアップ**: 
  - マルチウィンドウ時の扱いを明確化（現在はシングルウィンドウ想定）

## リスクと軽減策

### リスク1: 高レベルコンポーネントでtaffyの全機能をカバーできない
- **軽減策**: 
  - 初期リリースではFlexbox基本機能のみサポート（Requirement 3）
  - 将来的な拡張設計（Gap, Position, MinSize等のコンポーネント追加）
  - ユーザーフィードバックに基づく優先順位付け

### リスク2: 変更検知の漏れによるレイアウト更新失敗
- **軽減策**:
  - ユニットテストで変更検知シナリオを網羅（Requirement 9）
  - 初期化フラグで初回全ツリー計算を保証（Requirement 6.9）
  - デバッグログで変更検知とcompute_layout()呼び出しをトレース

### リスク3: EntityとNodeIdのマッピング不整合
- **軽減策**:
  - RemovedComponents<T>で削除を確実に捕捉
  - クリーンアップシステムを専用で実装
  - Debug assertでマッピング整合性を検証

### リスク4: taffyのパフォーマンス問題（大規模ツリー）
- **軽減策**:
  - taffyの内蔵キャッシュに依存（実績のある実装）
  - 変更検知による計算スキップ
  - 必要に応じてベンチマークテストを追加

## 参考資料

- [Taffy公式ドキュメント](https://docs.rs/taffy/latest/taffy/) - レイアウトエンジンAPI
- [Taffy GitHubリポジトリ](https://github.com/DioxusLabs/taffy) - ソースコードと例
- [CSS Flexbox Guide](https://css-tricks.com/snippets/css/a-guide-to-flexbox/) - Flexboxレイアウトの理解
- [bevy_ecs Hierarchy](https://docs.rs/bevy_ecs/latest/bevy_ecs/hierarchy/) - ECS階層システム
