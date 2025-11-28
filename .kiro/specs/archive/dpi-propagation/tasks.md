# Implementation Plan

## Task Overview
- **Feature**: dpi-propagation
- **Total Requirements**: 6 (18 acceptance criteria)
- **Estimated Tasks**: 4 major tasks, 9 sub-tasks

---

## Tasks

- [x] 1. DPIコンポーネントの実装
- [x] 1.1 (P) DPI構造体とコンストラクタメソッドの実装
  - DPI値を保持するコンポーネント構造体を定義する
  - SparseSetストレージ戦略を指定する
  - `from_dpi`メソッドで任意のDPI値からインスタンスを作成できるようにする
  - `from_WM_DPICHANGED`メソッドでWin32メッセージパラメータを解析する
  - スケールファクター計算メソッド（DPI / 96.0）を実装する
  - デフォルト値（96, 96）を持つDefault実装を追加する
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [x] 1.2 (P) DPIコンポーネントのpubエクスポート追加
  - モジュールの公開APIにDPIコンポーネントを追加する
  - _Requirements: 5.3_

- [x] 2. WindowHandle作成時のDPI自動付与
- [x] 2.1 WindowHandleフックでDPIコンポーネントを自動挿入する
  - WindowHandle追加時のフック関数を拡張する
  - Win32 APIから現在のウィンドウDPI値を取得する
  - 取得成功時は実際のDPI値で、失敗時はデフォルト96で初期化する
  - コマンドキューを使用してDPIコンポーネントを挿入する
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 3. WM_DPICHANGEDメッセージ処理
- [x] 3.1 ウィンドウプロシージャでDPIコンポーネントを更新する
  - WM_DPICHANGEDメッセージハンドラを実装する
  - wparamから新しいDPI値を抽出する
  - 対象エンティティのDPIコンポーネントを更新する
  - 変更検知のため既存値との比較後に更新する
  - _Requirements: 3.1, 3.2, 3.3_

- [x] 4. DPIからArrangementへの伝搬とコード整理
- [x] 4.1 レイアウトシステムにDPI変更検知を追加する
  - システムのクエリフィルターにDPI変更のOR条件を追加する
  - クエリパラメータにオプショナルなDPI参照を追加する
  - DPI存在時はスケールファクターを、非存在時はデフォルト(1.0, 1.0)を使用する
  - 全フィールド（offset, size, scale）を再計算する
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

- [x] 4.2 (P) 未使用のDpiTransform構造体を削除する
  - DpiTransform構造体定義をwindow.rsから削除する
  - 関連するimplブロックも削除する
  - _Requirements: 5.1, 5.2_

- [x] 4.3 ビルドと既存テストの確認
  - `cargo build`でエラーなくビルドできることを確認する
  - `cargo test`で既存テストが通過することを確認する
  - _Requirements: 6.1, 6.3_

- [x] 4.4* 手動統合テストの実施
  - デュアルモニタ環境でtaffy_flex_demoを実行する
  - 低DPIモニタでウィンドウ作成、初期DPI値を確認する
  - 高DPIモニタへ移動、WM_DPICHANGEDログとArrangement.scale更新を確認する
  - 既存のレイアウト機能が正常に動作することを確認する
  - _Requirements: 6.2_

---

## Requirements Coverage

| Requirement | Tasks |
|-------------|-------|
| 1.1 - DPI型定義 | 1.1 |
| 1.2 - SparseSetストレージ | 1.1 |
| 1.3 - from_dpiメソッド | 1.1 |
| 1.4 - from_WM_DPICHANGEDメソッド | 1.1 |
| 1.5 - scale_x/scale_yメソッド | 1.1 |
| 2.1 - WindowHandle追加時のDPI付与 | 2.1 |
| 2.2 - GetDpiForWindow使用 | 2.1 |
| 2.3 - 取得失敗時デフォルト96 | 2.1 |
| 3.1 - WM_DPICHANGED受信時更新 | 3.1 |
| 3.2 - wparam解析 | 3.1 |
| 3.3 - Changed検知 | 3.1 |
| 4.1 - Or条件フィルター | 4.1 |
| 4.2 - Option<&DPI>パラメータ | 4.1 |
| 4.3 - scale計算にDPI使用 | 4.1 |
| 4.4 - None時デフォルトscale | 4.1 |
| 4.5 - GlobalArrangement継承 | 4.1 |
| 4.6 - 全フィールド再計算 | 4.1 |
| 5.1 - DpiTransform削除 | 4.2 |
| 5.2 - 参照削除 | 4.2 |
| 5.3 - pub use DPI追加 | 1.2 |
| 6.1 - 既存テスト通過 | 4.3 |
| 6.2 - レイアウト機能維持 | 4.4 |
| 6.3 - エラーなしビルド | 4.3 |
