# Implementation Plan: wintf-P0-image-widget

## 留意事項

- **ログ出力**: 実装時は`wintf-P0-logging-system`完了後のコードベースで行う。設計文書の`eprintln!`は`tracing`マクロに置換する
- **テストリソース**: `crates/wintf/tests/assets/`に配置済み

---

## Tasks

- [ ] 1. インフラストラクチャ基盤の構築
  - WicCoreとWintfTaskPoolの2つの独立Resourceを作成し、非同期処理とWIC画像デコードの基盤を確立する

- [ ] 1.1 (P) WicCore Resource の実装
  - WICファクトリを保持するResourceを作成（Device Lostの影響を受けない独立リソース）
  - IWICImagingFactory2をCoCreateInstanceで初期化
  - Clone + Send + Sync traitを実装
  - factory()アクセサを提供
  - _Requirements: 2.2, 2.4, 2.5_

- [ ] 1.2 (P) WintfTaskPool Resource の実装
  - TaskPoolとmpscチャネルを組み合わせた非同期タスク実行基盤を作成
  - BoxedCommand型エイリアス（Box<dyn Command + Send>）を定義
  - spawn()メソッドでCommandSenderを自動渡しするAPIを実装
  - drain_and_apply()メソッドで受信コマンドをWorldに適用
  - _Requirements: 1.1, 1.2, 1.5, 1.6_

- [ ] 1.3 drain_task_pool_commands システムの実装
  - Inputスケジュールで実行されるシステムを作成
  - WintfTaskPoolをWorldから一時取り出し→drain_and_apply→再挿入
  - _Requirements: 1.7_

---

- [ ] 2. BitmapSourceコンポーネント群の実装
  - 画像ウィジェットの3層コンポーネント構造（論理・CPU・GPU）を構築する

- [ ] 2.1 BitmapSource コンポーネントの実装
  - 画像パスを保持する論理コンポーネントを作成
  - on_add/on_removeフックを設定（hookの中身は後続タスクで実装）
  - new()コンストラクタを提供
  - _Requirements: 2.1, 5.7_

- [ ] 2.2 (P) BitmapSourceResource コンポーネントの実装
  - IWICBitmapSourceを保持するCPUリソースコンポーネントを作成
  - Send + Sync traitを手動実装（WICはthread-free marshaling対応）
  - source()アクセサを提供
  - _Requirements: 2.4, 5.1, 5.2_

- [ ] 2.3 (P) BitmapSourceGraphics コンポーネントの実装
  - Option<ID2D1Bitmap1>を保持するGPUリソースコンポーネントを作成
  - Send + Sync traitを手動実装
  - new()、bitmap()、set_bitmap()、invalidate()、is_valid()メソッドを実装
  - _Requirements: 4.2, 5.1, 5.3_

---

- [ ] 3. 非同期画像読み込み処理の実装
  - WICを使用した画像デコードとECSへのコマンド送信フローを構築する

- [ ] 3.1 パス解決ユーティリティの実装
  - 実行ファイル基準のパス解決関数を作成
  - 絶対パスはそのまま、相対パスは実行ファイルディレクトリを基準に解決
  - _Requirements: 2.7_

- [ ] 3.2 load_bitmap_source 関数の実装
  - WICファクトリを使用して画像ファイルをデコード
  - CreateDecoderFromFilenameでデコーダー作成
  - FormatConverterでPBGRA32に変換（αチャネルがなくても100%不透明として処理）
  - エラー時は明確なエラーメッセージを返す
  - _Requirements: 2.2, 2.3, 2.5, 2.6, 2.7, 2.8, 3.1, 3.2, 3.3_

- [ ] 3.3 InsertBitmapSourceResource Command の実装
  - 非同期読み込み完了時にEntityにBitmapSourceResourceを挿入するCommandを作成
  - Entity存在チェックを含める（読み込み中にdespawnされた場合の対応）
  - _Requirements: 1.4_

---

- [ ] 4. on_addフック処理の実装
  - BitmapSource追加時の自動セットアップと非同期読み込み起動を実装する

- [ ] 4.1 on_bitmap_source_add フックの実装
  - DeferredWorldを使用してVisual + BitmapSourceGraphicsを自動挿入
  - WicCoreをcloneして取得
  - WintfTaskPoolを使用して非同期読み込みタスクを起動
  - パス解決→load_bitmap_source→InsertBitmapSourceResource送信の流れを実装
  - _Requirements: 1.1, 1.3, 5.5, 5.6_

- [ ] 4.2 on_bitmap_source_remove フックの実装
  - エンティティ削除時のクリーンアップ処理（必要に応じて）
  - COMオブジェクトはDropで自動解放されるため、明示的な処理は最小限
  - _Requirements: 5.6_

---

- [ ] 5. 描画システムの実装
  - BitmapSourceResourceからD2D Bitmapを生成し、GraphicsCommandListに描画コマンドを出力する

- [ ] 5.1 draw_bitmap_sources システムの実装
  - BitmapSourceResource + BitmapSourceGraphicsを持つEntityをクエリ
  - BitmapSourceGraphics.is_valid() == falseの場合、D2D Bitmapを生成
  - GraphicsCore.device_context()でCreateBitmapFromWicBitmapを呼び出し
  - 生成したBitmapをBitmapSourceGraphics.set_bitmap()で保存
  - GraphicsCommandListに描画コマンドを出力（OFFSET(0,0)から描画）
  - _Requirements: 4.1, 4.3, 4.4, 4.5, 4.8_

- [ ] 5.2 invalidate_dependent_components システムへの統合
  - 既存のinvalidate_dependent_componentsシステムにBitmapSourceGraphicsを追加
  - Device Lost時にBitmapSourceGraphics.invalidate()が呼ばれるようにする
  - _Requirements: 4.7_

---

- [ ] 6. モジュール構成とエクスポートの整備
  - bitmap_sourceモジュールをwidget階層に統合し、公開APIを整える

- [ ] 6.1 モジュール構造の構築
  - ecs/widget/bitmap_source/ディレクトリを作成
  - mod.rs、bitmap_source.rs、resource.rs、systems.rsに分割
  - 公開APIの再エクスポートを設定
  - _Requirements: 5.7, 6.1, 6.2, 6.3, 6.4_

- [ ] 6.2 スケジュール登録の実装
  - drain_task_pool_commandsをInputスケジュールに登録
  - draw_bitmap_sourcesをPostLayoutスケジュールに登録（calculate_arrangementの後）
  - _Requirements: 1.7, 4.8_

---

- [ ] 7. 統合テストの実装
  - 各コンポーネントとシステムの動作を検証するテストを作成する

- [ ] 7.1 (P) ユニットテストの実装
  - BitmapSourceコンポーネントのpath保持確認
  - BitmapSourceResourceのSend/Sync trait確認
  - WintfTaskPoolのchannelドレイン動作確認
  - _Requirements: 2.1, 5.2_

- [ ] 7.2 画像読み込みテストの実装
  - test_8x8_rgba.png（αチャネル付き）の正常読み込み確認
  - test_8x8_rgb.bmp（αなし）のPBGRA32変換確認
  - invalid.binのエラーハンドリング確認
  - CARGO_MANIFEST_DIR基準のテスト用パス解決を使用
  - _Requirements: 2.2, 2.3, 2.8, 2.9_

---

## Requirements Coverage

| Requirement | Tasks |
|-------------|-------|
| 1 (非同期読み込み) | 1.2, 1.3, 3.3, 4.1 |
| 2 (静止画像読み込み) | 1.1, 2.1, 2.2, 3.1, 3.2 |
| 3 (透過処理) | 3.2 |
| 4 (D2D描画) | 2.3, 5.1, 5.2 |
| 5 (ECS統合) | 2.1, 2.2, 2.3, 4.1, 4.2, 6.1, 6.2 |
| 6 (将来拡張性) | 6.1 |
