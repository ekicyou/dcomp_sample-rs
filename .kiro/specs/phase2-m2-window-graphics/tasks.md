# 実装タスク: Phase 2 Milestone 2 - WindowGraphics + Visual作成

**Feature ID**: `phase2-m2-window-graphics`  
**Last Updated**: 2025-11-14

---

## 実装計画

### タスク概要
- 合計: 7個の主要タスク、17個のサブタスク
- 要件カバレッジ: 全6要件（Requirement 1-6）
- 平均タスク時間: 1-3時間/サブタスク
- 並列実行: 可能なタスクに`(P)`マーク付与

---

## タスクリスト

- [ ] 1. WindowGraphicsコンポーネントの実装
- [ ] 1.1 (P) WindowGraphics構造体を定義する
  - IDCompositionTargetとID2D1DeviceContextの2つのフィールドを持つ
  - Send + Syncトレイトを実装する（unsafe impl）
  - Debug派生トレイトを追加する
  - _Requirements: 1.2, 1.6, 3.1, 3.3, 4.5_

- [ ] 1.2 (P) WindowGraphicsのアクセスメソッドを実装する
  - IDCompositionTargetへの参照を取得するメソッドを提供する
  - ID2D1DeviceContextへの参照を取得するメソッドを提供する
  - _Requirements: 3.6, 3.8_

- [ ] 2. Visualコンポーネントの実装
- [ ] 2.1 (P) Visual構造体を定義する
  - IDCompositionVisual3のフィールドを持つ
  - Send + Syncトレイトを実装する（unsafe impl）
  - Debug派生トレイトを追加する
  - _Requirements: 2.2, 3.2, 3.4, 4.5_

- [ ] 2.2 (P) Visualのアクセスメソッドを実装する
  - IDCompositionVisual3への参照を取得するメソッドを提供する
  - _Requirements: 3.7_

- [ ] 3. create_window_graphicsシステムの実装
- [ ] 3.1 create_window_graphicsシステムを実装する
  - Query<(Entity, &WindowHandle), Without<WindowGraphics>>でウィンドウを検出する
  - GraphicsCoreリソースからdesktopデバイスとd2dデバイスを取得する
  - create_target_for_hwnd APIでIDCompositionTargetを作成する（topmost=true）
  - create_device_contextでID2D1DeviceContextを作成する（D2D1_DEVICE_CONTEXT_OPTIONS_NONE）
  - WindowGraphicsコンポーネントをCommandsで挿入する
  - _Requirements: 1.1, 1.3, 1.4, 1.5, 1.8, 5.1, 5.3_

- [ ] 3.2 create_window_graphicsのエラーハンドリングを実装する
  - GraphicsCoreが存在しない場合は警告ログを出力してスキップする
  - create_target_for_hwnd失敗時はエラーログを出力する（Entity ID, HWND, HRESULTを含む）
  - create_device_context失敗時はエラーログを出力する（Entity ID, HRESULTを含む）
  - エラー時もパニックせず処理を継続する
  - _Requirements: 1.7, 5.5, 6.3, 6.5_

- [ ] 3.3 create_window_graphicsのログ出力を実装する
  - WindowGraphics作成開始時にEntity IDとHWNDをログ出力する
  - IDCompositionTarget作成成功をログ出力する
  - ID2D1DeviceContext作成成功をログ出力する
  - WindowGraphics作成完了をeprintln!で出力する
  - _Requirements: 1.6, 6.1, 6.4_

- [ ] 4. create_window_visualシステムの実装
- [ ] 4.1 create_window_visualシステムを実装する
  - Query<(Entity, &WindowHandle, &WindowGraphics), Without<Visual>>でウィンドウを検出する
  - GraphicsCoreリソースからdcompデバイスを取得する
  - create_visualでIDCompositionVisual3を作成する
  - WindowGraphicsのtarget.set_root()でビジュアルをルートに設定する
  - VisualコンポーネントをCommandsで挿入する
  - _Requirements: 2.1, 2.3, 2.4, 2.5, 2.8, 5.2, 5.4_

- [ ] 4.2 create_window_visualのエラーハンドリングを実装する
  - create_visual失敗時はエラーログを出力する（Entity ID, HRESULTを含む）
  - set_root失敗時はエラーログを出力する（Entity ID, HRESULTを含む）
  - エラー時もパニックせず処理を継続する
  - _Requirements: 2.7, 6.3, 6.5_

- [ ] 4.3 create_window_visualのログ出力を実装する
  - Visual作成開始時にEntity IDをログ出力する
  - IDCompositionVisual3作成成功をログ出力する
  - SetRoot成功をログ出力する
  - Visual作成完了をeprintln!で出力する
  - _Requirements: 2.6, 6.2, 6.4_

- [ ] 5. commit_compositionシステムの実装
- [ ] 5.1 commit_compositionシステムを実装する
  - GraphicsCoreリソースからdcompデバイスを取得する
  - dcomp.commit()を呼び出してDirectCompositionの変更を確定する
  - Commit開始と完了をログ出力する
  - エラー時はHRESULTを含むログを出力する
  - _Requirements: (設計決定事項、research.mdに記載)_

- [ ] 6. システム登録とスケジュール配置
- [ ] 6.1 world.rsにCommitCompositionスケジュールを追加する
  - CommitCompositionスケジュールラベルを定義する
  - schedules.insert(Schedule::new(CommitComposition))で登録する
  - try_tick_worldでCommitCompositionスケジュールを最後に実行する
  - スケジュール説明コメントを追加する
  - _Requirements: (設計決定事項、research.mdに記載)_

- [ ] 6.2 PostLayoutスケジュールにグラフィックスシステムを登録する
  - create_window_graphicsシステムをPostLayoutに登録する
  - create_window_visualシステムをPostLayoutに登録し、.after(create_window_graphics)で依存関係を設定する
  - _Requirements: 5.3, 5.4, 5.6_

- [ ] 6.3 CommitCompositionスケジュールにcommit_compositionを登録する
  - commit_compositionシステムをCommitCompositionスケジュールに登録する
  - _Requirements: (設計決定事項、research.mdに記載)_

- [ ] 7. テストの実装
- [ ] 7.1 simple_window.rsを拡張してテストコードを追加する
  - env_loggerを初期化する（debug levelログ出力）
  - 1ウィンドウを作成してrun_schedule_once()を実行する
  - GraphicsCoreリソースの存在を検証する
  - Query<(Entity, &WindowHandle, &WindowGraphics, &Visual)>で全コンポーネントの存在を検証する
  - COMオブジェクトの有効性（!is_invalid()）を検証する
  - テスト結果をprintln!で出力する
  - _Requirements: (Testing Strategy)_

- [ ] 7.2 (P) multi_window_test.rsを作成する
  - 3つのウィンドウを作成する
  - run_schedule_once()後に3つのエンティティが全コンポーネントを持つことを検証する
  - 各エンティティのCOMオブジェクトが有効であることを検証する
  - テスト成功メッセージを出力する
  - _Requirements: (Testing Strategy)_

- [ ]* 7.3 (P) graphics_core_test.rsにコンポーネントテストを追加する
  - GraphicsCoreからDeviceContext作成テストを追加する
  - GraphicsCoreからVisual作成テストを追加する
  - 各テストでCOMオブジェクトの有効性を検証する
  - _Requirements: (Testing Strategy)_

---

## 要件カバレッジ

| 要件ID | タスク | 状態 |
|--------|--------|------|
| Requirement 1.1 | 3.1 | ✅ |
| Requirement 1.2 | 1.1 | ✅ |
| Requirement 1.3 | 3.1 | ✅ |
| Requirement 1.4 | 3.1 | ✅ |
| Requirement 1.5 | 3.1 | ✅ |
| Requirement 1.6 | 1.1, 3.3 | ✅ |
| Requirement 1.7 | 3.2 | ✅ |
| Requirement 1.8 | 3.1 | ✅ |
| Requirement 2.1 | 4.1 | ✅ |
| Requirement 2.2 | 2.1 | ✅ |
| Requirement 2.3 | 4.1 | ✅ |
| Requirement 2.4 | 4.1 | ✅ |
| Requirement 2.5 | 4.1 | ✅ |
| Requirement 2.6 | 4.3 | ✅ |
| Requirement 2.7 | 4.2 | ✅ |
| Requirement 2.8 | 4.1 | ✅ |
| Requirement 3.1 | 1.1 | ✅ |
| Requirement 3.2 | 2.1 | ✅ |
| Requirement 3.3 | 1.1 | ✅ |
| Requirement 3.4 | 2.1 | ✅ |
| Requirement 3.5 | 7.1 | ✅ |
| Requirement 3.6 | 1.2 | ✅ |
| Requirement 3.7 | 2.2 | ✅ |
| Requirement 3.8 | 1.2 | ✅ |
| Requirement 4.1 | (ECS自動削除) | ✅ |
| Requirement 4.2 | (windows-rs自動Release) | ✅ |
| Requirement 4.3 | (windows-rs自動Release) | ✅ |
| Requirement 4.4 | (将来実装) | ⚠️ |
| Requirement 4.5 | 1.1, 2.1 | ✅ |
| Requirement 5.1 | 3.1 | ✅ |
| Requirement 5.2 | 4.1 | ✅ |
| Requirement 5.3 | 3.1, 6.2 | ✅ |
| Requirement 5.4 | 4.1, 6.2 | ✅ |
| Requirement 5.5 | 3.2 | ✅ |
| Requirement 5.6 | 6.2 | ✅ |
| Requirement 5.7 | (ドキュメント) | ✅ |
| Requirement 6.1 | 3.3 | ✅ |
| Requirement 6.2 | 4.3 | ✅ |
| Requirement 6.3 | 3.2, 4.2 | ✅ |
| Requirement 6.4 | 3.3, 4.3 | ✅ |
| Requirement 6.5 | 3.2, 4.2 | ✅ |

---

## 実装順序

### フェーズ1: コンポーネント定義（並列実行可能）
1. タスク1.1, 1.2 - WindowGraphicsコンポーネント
2. タスク2.1, 2.2 - Visualコンポーネント

### フェーズ2: システム実装（順次実行）
3. タスク3.1, 3.2, 3.3 - create_window_graphicsシステム
4. タスク4.1, 4.2, 4.3 - create_window_visualシステム（3の後）
5. タスク5.1 - commit_compositionシステム

### フェーズ3: スケジュール統合（順次実行）
6. タスク6.1 - CommitCompositionスケジュール追加
7. タスク6.2 - PostLayoutスケジュール登録
8. タスク6.3 - CommitCompositionスケジュール登録

### フェーズ4: テスト実装（並列実行可能）
9. タスク7.1 - simple_window.rs拡張
10. タスク7.2 - multi_window_test.rs作成
11. タスク7.3 - graphics_core_test.rs拡張（オプション）

---

## 依存関係

- タスク3.x（create_window_graphics）: タスク1.x（WindowGraphics）に依存
- タスク4.x（create_window_visual）: タスク2.x（Visual）とタスク3.x（create_window_graphics）に依存
- タスク6.2, 6.3: タスク3.x, 4.x, 5.1（全システム実装）に依存
- タスク7.x（テスト）: タスク6.x（システム登録）に依存

---

## 品質チェックリスト

### コード品質
- [ ] すべてのコンポーネントがSend + Syncを実装
- [ ] エラーハンドリングでパニックが発生しない
- [ ] COMオブジェクトのライフタイムがwindows-rsで適切に管理される
- [ ] ログ出力がEntity ID、HWND、HRESULTを含む

### アーキテクチャ
- [ ] PostLayoutスケジュールに正しく配置（UISetupではない）
- [ ] create_window_visualがcreate_window_graphicsの後に実行される
- [ ] CommitCompositionスケジュールが全スケジュールの最後に実行される
- [ ] レイヤードアーキテクチャ（structure.md）に準拠

### テスト
- [ ] simple_window.rsで基本動作確認
- [ ] multi_window_test.rsで複数ウィンドウ検証
- [ ] ログ出力で詳細な実行トレースが確認可能

---

## 次のステップ

タスクが承認されたら、以下のコマンドで実装フェーズに進みます:

```bash
# 特定のタスクを実行
/kiro-spec-impl phase2-m2-window-graphics 1.1

# 複数タスクを実行
/kiro-spec-impl phase2-m2-window-graphics 1.1,1.2

# すべてのタスクを実行（非推奨、コンテキスト肥大化のため）
/kiro-spec-impl phase2-m2-window-graphics
```

**重要**: タスク間でコンテキストをクリアすることを推奨します。
