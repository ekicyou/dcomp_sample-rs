# 実装タスク: Phase 2 Milestone 3 - 初めての描画

**Feature ID**: `phase2-m3-first-rendering`  
**Last Updated**: 2025-11-14

---

## 実装計画

### タスク概要
- 合計: 6個の主要タスク、15個のサブタスク
- 要件カバレッジ: 全8要件（Requirement 1-8）
- 平均タスク時間: 1-3時間/サブタスク
- 並列実行: 可能なタスクに`(P)`マーク付与

---

## タスクリスト

### 1. Surfaceコンポーネントの実装

- [ ] 1.1 (P) Surface構造体を定義する
  - IDCompositionSurfaceフィールドを持つ
  - Send + Syncトレイトを実装する（unsafe impl）
  - Debug派生トレイトを追加する
  - _Requirements: 1.2, 8.1, 8.2, 8.5_

- [ ] 1.2 (P) Surfaceのアクセスメソッドを実装する
  - IDCompositionSurfaceへの参照を取得するメソッドを提供する
  - _Requirements: 8.3_

### 2. create_window_surfaceシステムの実装

- [ ] 2.1 create_window_surfaceシステムを実装する
  - Query<(Entity, &WindowGraphics, &Visual, Option<&WindowPos>), Without<Surface>>でウィンドウを検出する
  - WindowPosからウィンドウサイズを取得する（なければデフォルト800x600）
  - GraphicsCoreのdcompデバイスからIDCompositionSurfaceを作成する
  - ピクセルフォーマットにDXGI_FORMAT_B8G8R8A8_UNORMを指定する
  - アルファモードにDXGI_ALPHA_MODE_PREMULTIPLIEDを指定する
  - VisualのIDCompositionVisual3にSetContent()でSurfaceを設定する
  - SurfaceコンポーネントをCommandsで挿入する
  - _Requirements: 1.1, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 1.12_

- [ ] 2.2 create_window_surfaceのエラーハンドリングを実装する
  - GraphicsCoreが存在しない場合は警告ログを出力してスキップする
  - create_surface失敗時はエラーログを出力する（Entity ID, Size, HRESULTを含む）
  - set_content失敗時はエラーログを出力する（Entity ID, HRESULTを含む）
  - エラー時もパニックせず処理を継続する
  - _Requirements: 1.11, 7.5_

- [ ] 2.3 create_window_surfaceのログ出力を実装する
  - Surface作成開始時にEntity IDとサイズをログ出力する
  - IDCompositionSurface作成成功をログ出力する
  - Visual.SetContent()成功をログ出力する
  - Surface作成完了をeprintln!で出力する
  - _Requirements: 1.10, 7.1, 7.4_

### 3. render_windowシステムの実装（描画処理）

- [ ] 3.1 render_windowシステムの基本構造を実装する
  - Query<&Surface, Added<Surface>>でSurfaceが追加されたウィンドウのみを検出する
  - GraphicsCoreリソースから描画に必要なリソースを取得する
  - IDCompositionSurface.BeginDraw(None)でDeviceContextを取得する
  - device_context.BeginDraw()を呼び出す
  - device_context.EndDraw()を呼び出す
  - IDCompositionSurface.EndDraw()を呼び出す
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.17, 2.18, 2.20_

- [ ] 3.2 透明背景のクリア処理を実装する
  - device_context.Clear()で透明色（rgba: 0.0, 0.0, 0.0, 0.0）をクリアする
  - _Requirements: 2.5_

- [ ] 3.3 赤い円の描画を実装する
  - CreateSolidColorBrush()で赤色ブラシ（rgba: 1.0, 0.0, 0.0, 1.0）を作成する
  - FillEllipse()で円を描画する（中心: 100.0, 100.0、半径: 50.0, 50.0）
  - ブラシ作成失敗時はエラーログを出力して図形をスキップする
  - _Requirements: 2.6, 2.7, 2.8, 3.1, 3.2, 3.6, 7.7_

- [ ] 3.4 緑の四角の描画を実装する
  - CreateSolidColorBrush()で緑色ブラシ（rgba: 0.0, 1.0, 0.0, 1.0）を作成する
  - FillRectangle()で矩形を描画する（left: 200.0, top: 50.0, right: 300.0, bottom: 150.0）
  - ブラシ作成失敗時はエラーログを出力して図形をスキップする
  - _Requirements: 2.9, 2.10, 2.11, 3.1, 3.3, 3.6, 7.7_

- [ ] 3.5 青い三角の描画を実装する
  - CreatePathGeometry()でPathGeometryを作成する
  - Open()でGeometrySinkを取得する
  - BeginFigure()で第1頂点（350.0, 50.0）を設定し、D2D1_FIGURE_BEGIN_FILLEDを指定する
  - AddLine()で第2頂点（425.0, 150.0）を追加する
  - AddLine()で第3頂点（275.0, 150.0）を追加する
  - EndFigure()でD2D1_FIGURE_END_CLOSEDを指定して図形を閉じる
  - Close()でGeometrySinkを確定する
  - CreateSolidColorBrush()で青色ブラシ（rgba: 0.0, 0.0, 1.0, 1.0）を作成する
  - FillGeometry()でPathGeometryを描画する
  - PathGeometry作成失敗時はエラーログを出力して三角形をスキップする
  - _Requirements: 2.12, 2.13, 2.14, 2.15, 2.16, 3.1, 3.4, 3.6, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 4.9, 4.10, 4.11, 4.12, 7.6_

- [ ] 3.6 render_windowのエラーハンドリングとログ出力を実装する
  - GraphicsCoreが存在しない場合は処理をスキップする
  - 描画処理開始時にEntity IDをログ出力する
  - EndDraw()失敗時はエラーログを出力する（Entity ID, HRESULTを含む）
  - 描画処理完了時に成功メッセージをログ出力する
  - エラー発生時もパニックせず、そのウィンドウの描画をスキップして処理を継続する
  - _Requirements: 2.19, 7.2, 7.3, 7.4, 7.5_

### 4. Direct2D COM APIラッパーの拡張

- [ ] 4.1 (P) D2D1DeviceContextExt::fill_ellipseを実装する
  - FillEllipse()をラップするメソッドを追加する
  - _Requirements: 2.6_

- [ ] 4.2 (P) D2D1DeviceContextExt::fill_rectangleを実装する
  - FillRectangle()をラップするメソッドを追加する
  - _Requirements: 2.9_

- [ ] 4.3 (P) D2D1DeviceContextExt::fill_geometryを実装する
  - FillGeometry()をラップするメソッドを追加する
  - _Requirements: 2.12_

- [ ] 4.4 (P) D2D1FactoryExt::create_path_geometryを実装する
  - CreatePathGeometry()をラップするメソッドを追加する
  - _Requirements: 4.1_

### 5. スケジュール構成の更新

- [ ] 5.1 world.rsにRenderスケジュールを追加する
  - Renderスケジュールラベルを定義する
  - schedules.insert(Schedule::new(Render))で登録する
  - try_tick_worldでRenderスケジュールをPostLayoutの後に実行する
  - スケジュール説明コメントを追加する
  - _Requirements: 6.1, 6.7, 6.9_

- [ ] 5.2 システムをスケジュールに登録する
  - create_window_surfaceシステムをPostLayoutに登録する
  - create_window_surfaceをcreate_window_visualの後に実行するよう依存関係を設定する
  - render_windowシステムをRenderスケジュールに登録する
  - 既存のcommit_compositionシステムはCommitCompositionスケジュールに配置されていることを確認する
  - _Requirements: 6.2, 6.3, 6.4, 6.5, 6.6, 6.8_

### 6. 統合テストの実装

- [ ]* 6.1 simple_window.rsを拡張してSurface作成テストを追加する
  - 1ウィンドウを作成してrun_schedule_once()を実行する
  - Query<(&WindowHandle, &WindowGraphics, &Visual, &Surface)>で全コンポーネントの存在を検証する
  - テスト結果をprintln!で出力する
  - _Requirements: 8.6_

- [ ]* 6.2 描画結果の手動確認手順をドキュメント化する
  - ウィンドウ起動手順を記載する
  - 透明背景の確認方法（デスクトップが透ける）を記載する
  - 赤い円●の確認ポイント（中心: 100,100, 半径: 50）を記載する
  - 緑の四角■の確認ポイント（200,50 - 300,150）を記載する
  - 青い三角▲の確認ポイント（頂点: 350,50 - 425,150 - 275,150）を記載する

---

## 実装順序

### フェーズ1: コンポーネント定義とCOMラッパー（並列実行可能）
1. タスク1.1, 1.2 - Surfaceコンポーネント
2. タスク4.1, 4.2, 4.3, 4.4 - Direct2D COM APIラッパー

### フェーズ2: Surface作成システム（順次実行）
3. タスク2.1 - create_window_surfaceシステム基本実装
4. タスク2.2 - エラーハンドリング
5. タスク2.3 - ログ出力

### フェーズ3: 描画システム（順次実行）
6. タスク3.1 - render_window基本構造
7. タスク3.2 - 透明背景クリア
8. タスク3.3 - 赤い円描画
9. タスク3.4 - 緑の四角描画
10. タスク3.5 - 青い三角描画
11. タスク3.6 - エラーハンドリングとログ出力

### フェーズ4: スケジュール統合（順次実行）
12. タスク5.1 - Renderスケジュール追加
13. タスク5.2 - システム登録

### フェーズ5: テスト（オプション）
14. タスク6.1 - 統合テスト拡張
15. タスク6.2 - 手動確認ドキュメント

---

## 依存関係

- タスク2.x（create_window_surface）: タスク1.x（Surface）に依存
- タスク3.x（render_window）: タスク1.x（Surface）とタスク4.x（COM API）に依存
- タスク5.x（スケジュール）: タスク2.x、3.x（全システム実装）に依存
- タスク6.x（テスト）: タスク5.x（システム登録）に依存

---

## 要件カバレッジ

| 要件ID | タスク | 状態 |
|--------|--------|------|
| Requirement 1.1 | 2.1 | ⏳ |
| Requirement 1.2 | 1.1 | ⏳ |
| Requirement 1.3 | 2.1 | ⏳ |
| Requirement 1.4 | 2.1 | ⏳ |
| Requirement 1.5 | 2.1 | ⏳ |
| Requirement 1.6 | 2.1 | ⏳ |
| Requirement 1.7 | 2.1 | ⏳ |
| Requirement 1.8 | 2.1 | ⏳ |
| Requirement 1.9 | 2.1 | ⏳ |
| Requirement 1.10 | 2.3 | ⏳ |
| Requirement 1.11 | 2.2 | ⏳ |
| Requirement 1.12 | 2.1 | ⏳ |
| Requirement 2.1 | 3.1 | ⏳ |
| Requirement 2.2 | 3.1 | ⏳ |
| Requirement 2.3 | 3.1 | ⏳ |
| Requirement 2.4 | 3.1 | ⏳ |
| Requirement 2.5 | 3.2 | ⏳ |
| Requirement 2.6 | 3.3, 4.1 | ⏳ |
| Requirement 2.7 | 3.3 | ⏳ |
| Requirement 2.8 | 3.3 | ⏳ |
| Requirement 2.9 | 3.4, 4.2 | ⏳ |
| Requirement 2.10 | 3.4 | ⏳ |
| Requirement 2.11 | 3.4 | ⏳ |
| Requirement 2.12 | 3.5, 4.3 | ⏳ |
| Requirement 2.13 | 3.5 | ⏳ |
| Requirement 2.14 | 3.5 | ⏳ |
| Requirement 2.15 | 3.5 | ⏳ |
| Requirement 2.16 | 3.5 | ⏳ |
| Requirement 2.17 | 3.1 | ⏳ |
| Requirement 2.18 | 3.1 | ⏳ |
| Requirement 2.19 | 3.6 | ⏳ |
| Requirement 2.20 | 3.1 | ⏳ |
| Requirement 3.1 | 3.3, 3.4, 3.5 | ⏳ |
| Requirement 3.2 | 3.3 | ⏳ |
| Requirement 3.3 | 3.4 | ⏳ |
| Requirement 3.4 | 3.5 | ⏳ |
| Requirement 3.5 | (フレームごと作成) | ⏳ |
| Requirement 3.6 | 3.3, 3.4, 3.5 | ⏳ |
| Requirement 4.1 | 3.5, 4.4 | ⏳ |
| Requirement 4.2 | 3.5 | ⏳ |
| Requirement 4.3 | 3.5 | ⏳ |
| Requirement 4.4 | 3.5 | ⏳ |
| Requirement 4.5 | 3.5 | ⏳ |
| Requirement 4.6 | 3.5 | ⏳ |
| Requirement 4.7 | 3.5 | ⏳ |
| Requirement 4.8 | 3.5 | ⏳ |
| Requirement 4.9 | 3.5 | ⏳ |
| Requirement 4.10 | 3.5 | ⏳ |
| Requirement 4.11 | 3.5 | ⏳ |
| Requirement 4.12 | 3.5 | ⏳ |
| Requirement 5.1 | (既存システム維持) | ✅ |
| Requirement 5.2 | (既存システム維持) | ✅ |
| Requirement 5.3 | (既存システム維持) | ✅ |
| Requirement 5.4 | (既存システム維持) | ✅ |
| Requirement 5.5 | (既存システム維持) | ✅ |
| Requirement 5.6 | (既存システム維持) | ✅ |
| Requirement 6.1 | 2.1 | ⏳ |
| Requirement 6.2 | 3.1 | ⏳ |
| Requirement 6.3 | 2.1 | ⏳ |
| Requirement 6.4 | 3.1 | ⏳ |
| Requirement 6.5 | 3.1 | ⏳ |
| Requirement 6.6 | 5.2 | ⏳ |
| Requirement 6.7 | 5.1 | ⏳ |
| Requirement 6.8 | 5.2 | ⏳ |
| Requirement 6.9 | 5.1 | ⏳ |
| Requirement 7.1 | 2.3 | ⏳ |
| Requirement 7.2 | 3.6 | ⏳ |
| Requirement 7.3 | 3.6 | ⏳ |
| Requirement 7.4 | 2.3, 3.6 | ⏳ |
| Requirement 7.5 | 2.2, 3.6 | ⏳ |
| Requirement 7.6 | 3.5 | ⏳ |
| Requirement 7.7 | 3.3, 3.4 | ⏳ |
| Requirement 8.1 | 1.1 | ⏳ |
| Requirement 8.2 | 1.1 | ⏳ |
| Requirement 8.3 | 1.2 | ⏳ |
| Requirement 8.5 | 1.1 | ⏳ |
| Requirement 8.6 | 6.1 | ⏳ |

---

## 品質チェックリスト

### コード品質
- [ ] SurfaceコンポーネントがSend + Syncを実装
- [ ] エラーハンドリングでパニックが発生しない
- [ ] COMオブジェクトのライフタイムがwindows-rsで適切に管理される
- [ ] ログ出力がEntity ID、サイズ、HRESULTを含む
- [ ] ブラシは毎フレーム作成・自動解放される

### アーキテクチャ
- [ ] create_window_surfaceがPostLayoutスケジュールに配置
- [ ] render_windowがRenderスケジュールに配置
- [ ] Renderスケジュールがスケジュール実行順序で適切な位置（PostLayoutの後、CommitCompositionの前）
- [ ] create_window_surfaceがcreate_window_visualの後に実行される
- [ ] CommitCompositionスケジュールが全スケジュールの最後に実行される

### 描画品質
- [ ] 透明背景が正しく描画される（デスクトップが透ける）
- [ ] 赤い円が正しい位置とサイズで描画される（中心: 100,100、半径: 50）
- [ ] 緑の四角が正しい位置とサイズで描画される（200,50 - 300,150）
- [ ] 青い三角が正しい頂点で描画される（350,50 - 425,150 - 275,150）

---

## 次のステップ

タスクが承認されたら、以下のコマンドで実装フェーズに進みます:

```bash
# 特定のタスクを実行
/kiro-spec-impl phase2-m3-first-rendering 1.1

# 複数タスクを実行
/kiro-spec-impl phase2-m3-first-rendering 1.1,1.2,4.1,4.2,4.3,4.4

# すべてのタスクを実行（非推奨、コンテキスト肥大化のため）
/kiro-spec-impl phase2-m3-first-rendering
```

**推奨実装順序**:
1. フェーズ1: タスク1.1, 1.2, 4.1, 4.2, 4.3, 4.4（並列実行可能）
2. フェーズ2: タスク2.1, 2.2, 2.3（順次実行）
3. フェーズ3: タスク3.1, 3.2, 3.3, 3.4, 3.5, 3.6（順次実行）
4. フェーズ4: タスク5.1, 5.2（順次実行）
5. フェーズ5: タスク6.1, 6.2（オプション）

**重要**: タスク間でコンテキストをクリアすることを推奨します。
