# ドキュメント構成の再編成

## 実施日
2025-11-01

## 再編成の理由

1. **論理的な順序**: ECSの基礎概念を最初に説明すべき
2. **明確な章立て**: 部を設けて内容を整理
3. **検討資料の分離**: 設計文書と検討資料を明確に分ける

## 変更内容

### ファイル名の変更

| 旧ファイル名 | 新ファイル名 | 変更理由 |
|------------|------------|---------|
| 02-ecs-components.md | 01-ecs-components.md | ECS基礎を最初に |
| 01-widget-tree.md | 02-widget-tree.md | ECS理解後にツリー構造 |
| 13-system-separation.md | 03-system-separation.md | 基礎部分に統合 |
| 03-layout-system.md | 04-layout-system.md | 番号調整 |
| 08-layout-details.md | 05-layout-details.md | レイアウト章を連続に |
| 04-visual-directcomp.md | 06-visual-directcomp.md | 番号調整 |
| 05-update-flow.md | 07-update-flow.md | 番号調整 |
| 06-event-system.md | 08-event-system.md | 番号調整 |
| 09-hit-test.md | 09-hit-test.md | 変更なし |
| 07-ui-elements.md | 10-ui-elements.md | 番号調整 |
| 10-usage-examples.md | 11-usage-examples.md | 番号調整 |
| 11-visual-optimization.md | 12-visual-optimization.md | 番号調整 |
| 12-dependency-properties.md | reference/12-dependency-properties.md | 参考資料に移動 |

### 新しい構成

#### 第1部: bevy_ecs基礎
1. **01-ecs-components.md** - ECS概念、Entity、Component、System
2. **02-widget-tree.md** - UIツリーの表現、Parent/Children
3. **03-system-separation.md** - 各システムの責務と統合

**目的**: bevy_ecsの基本概念を理解する

#### 第2部: UIシステム実装
4. **04-layout-system.md** - レイアウトコンポーネントの定義
5. **05-layout-details.md** - Measure/Arrangeパス
6. **06-visual-directcomp.md** - 描画システムとの統合
7. **07-update-flow.md** - フレーム更新の流れ

**目的**: UIシステムの実装を理解する

#### 第3部: インタラクション
8. **08-event-system.md** - マウス・キーボードイベント処理
9. **09-hit-test.md** - 座標からEntityを検索

**目的**: ユーザーインタラクションの処理を理解する

#### 第4部: UI要素と使用例
10. **10-ui-elements.md** - Container、TextBlock、Imageなど
11. **11-usage-examples.md** - サンプルコード

**目的**: 実際のUI構築方法を学ぶ

#### 第5部: 最適化（参考）
12. **12-visual-optimization.md** - Visual作成の最適化

**目的**: パフォーマンス最適化の参考

### 参考資料（reference/）

- **12-dependency-properties.md** - WPFの依存関係プロパティとbevy_ecsの比較検討
  - 検討段階の資料
  - 設計には採用されていない

## 章タイトルの統一

すべての章に以下の形式でタイトルと説明を追加：

```markdown
# 第X章: タイトル

この章では、XXXについて説明します。
```

## 利点

1. **学習順序が明確**: 基礎 → 実装 → インタラクション → 使用例
2. **検索性向上**: 章番号と部で構造が明確
3. **設計文書の明確化**: 検討資料と設計文書を分離
4. **一貫性**: すべての章が統一されたフォーマット

## 従来の問題点

- ECSの基礎説明が2番目にあった
- 章の順序が論理的でなかった
- 検討資料（dependency-properties）が設計文書に混在していた
- システム設計が最後（13章）にあった

## 新構成での読み方

### 初学者向け
1. 第1部（第1-3章）を読んでbevy_ecsを理解
2. 第2部（第4-7章）でUIシステムを理解
3. 第4部（第10-11章）でサンプルコードを実践

### 実装者向け
1. 第1部でECSを確認
2. 第2部で詳細な実装を学習
3. 第3部でインタラクション処理を実装
4. 第5部で最適化を検討

### リファレンスとして
- 各章が独立しているため、必要な章を参照可能
- 章番号と部で素早く目的の内容を発見

## 今後の管理

- 新しい章を追加する場合は適切な部に配置
- 検討資料はreferenceディレクトリに配置
- 章番号は論理的な順序を維持
