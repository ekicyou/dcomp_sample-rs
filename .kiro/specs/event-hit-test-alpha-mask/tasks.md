# Implementation Plan

## Task Format

- [ ] Major task description
- [ ] X.X Sub-task description
  - Detail items
  - _Requirements: X_

---

## Tasks

- [ ] 1. WIC ピクセル取得APIの追加
- [ ] 1.1 (P) IWICBitmapSource拡張トレイトを実装する
  - WIC BitmapSourceから画像サイズを取得する機能を追加
  - WIC BitmapSourceからピクセルデータをバッファにコピーする機能を追加
  - 既存のWICトレイト（WICImagingFactoryExt等）と同じパターンでunsafe呼び出しをラップ
  - _Requirements: 3_

- [ ] 2. AlphaMaskデータ構造の実装
- [ ] 2.1 (P) αマスク構造体を実装する
  - ビットパック形式（1ビット/ピクセル）でマスクデータを保持する構造体を作成
  - MSBファーストのビットオーダーで8ピクセル単位の行アラインメントを実装
  - PBGRA32形式のピクセルデータからα値を抽出し、閾値128で2値化してマスクを生成
  - 指定座標がヒット対象か判定するメソッドを実装（範囲外はfalse）
  - マスクの幅・高さを取得するアクセサを提供
  - _Requirements: 2_

- [ ] 2.2 (P) BitmapSourceResourceにαマスクフィールドを追加する
  - 既存リソース構造体にαマスクをOption型で保持するフィールドを追加
  - αマスクへの参照を返すアクセサメソッドを追加
  - αマスクを設定するミューテータメソッドを追加（非同期生成完了時用）
  - 新規リソース生成時はαマスクをNoneで初期化
  - _Requirements: 2_

- [ ] 3. HitTestMode拡張とAPIの追加
- [ ] 3.1 (P) HitTestModeにAlphaMaskバリアントを追加する
  - ヒットテストモード列挙型にαマスク判定用のバリアントを追加
  - 既存のNone/Boundsバリアントの動作を維持
  - _Requirements: 1_

- [ ] 3.2 (P) HitTestコンポーネントにalpha_mask()コンストラクタを追加する
  - αマスクモードでHitTestを生成するコンストラクタメソッドを追加
  - 既存のnone()/bounds()メソッドと同じパターンで実装
  - _Requirements: 1, 6_

- [ ] 4. αマスク非同期生成システムの実装
- [ ] 4.1 αマスク生成システムを実装する
  - BitmapSourceResourceまたはHitTestの変更を検知するECSシステムを追加
  - HitTestMode::AlphaMaskが設定されている場合のみ処理を実行
  - 既にαマスクが生成済みの場合は処理をスキップ
  - WintfTaskPoolを使用して非同期でαマスク生成タスクをスポーン
  - WIC拡張トレイトでピクセルデータを取得し、αマスクを生成
  - 生成完了時にCommand経由でBitmapSourceResourceにαマスクを設定
  - WIC CopyPixels失敗時はerrorログを出力し、Boundsフォールバック
  - _Requirements: 3_

- [ ] 4.2 αマスク生成システムをスケジュールに登録する
  - WidgetGraphicsスケジュールに新システムを追加
  - draw_bitmap_sourcesシステムの直後に実行されるよう順序を指定
  - _Requirements: 3_

- [ ] 5. hit_test_entity関数のAlphaMask対応
- [ ] 5.1 hit_test_entityにαマスク判定分岐を追加する
  - HitTestMode::AlphaMaskの場合の処理分岐を追加
  - まず矩形判定を行い、範囲外なら早期リターン
  - 矩形内ならBitmapSourceResourceからαマスクを取得
  - αマスクが存在しない場合は矩形判定と同等の結果を返す（フォールバック）
  - スクリーン座標をマスク座標に変換（四捨五入で正確に）
  - αマスクのis_hitメソッドで最終判定
  - _Requirements: 4, 5_

- [ ] 6. 統合テストと動作確認
- [ ] 6.1 αマスク生成の単体テストを作成する
  - 透明部分を含むテストデータでマスク生成を検証
  - 閾値128での2値化が正しく動作することを確認
  - ビットパックのビットオーダー（MSBファースト）を検証
  - 境界値（α=127, 128）での動作を確認
  - _Requirements: 2, 3_

- [ ] 6.2 ヒット判定の統合テストを作成する
  - αマスクを使用したヒット判定が正しく動作することを検証
  - 透明領域でヒットしないことを確認
  - 不透明領域でヒットすることを確認
  - スケーリング時の座標変換が正しく動作することを確認
  - αマスク未生成時のBoundsフォールバックを検証
  - _Requirements: 4, 5_

- [ ] 6.3* デモアプリケーションでの動作確認用コードを追加する
  - 透明部分を含む画像をHitTest::alpha_mask()で表示するサンプルを追加
  - クリック時に透明部分が透過することを視覚的に確認可能にする
  - _Requirements: 1, 6_

---

## Requirements Coverage

| Requirement | Tasks |
|-------------|-------|
| 1 | 3.1, 3.2, 6.3 |
| 2 | 2.1, 2.2, 6.1 |
| 3 | 1.1, 4.1, 4.2, 6.1 |
| 4 | 5.1, 6.2 |
| 5 | 5.1, 6.2 |
| 6 | 3.2, 6.3 |

**All 6 requirements covered.**

---

## Parallel Execution Notes

以下のタスクは並列実行可能:
- **1.1, 2.1, 2.2, 3.1, 3.2**: 独立したコンポーネント実装、相互依存なし

依存関係により順次実行が必要:
- **4.1**: 1.1（WIC拡張）、2.1（AlphaMask）、2.2（リソース拡張）、3.1/3.2（HitTest拡張）に依存
- **4.2**: 4.1に依存
- **5.1**: 2.1（AlphaMask.is_hit）、2.2（リソースアクセサ）、3.1（HitTestMode）に依存
- **6.x**: 実装完了後に実行
