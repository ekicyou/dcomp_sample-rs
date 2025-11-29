# Research Notes: multimonitor-resize-flicker

## Investigation Summary

本ドキュメントは、マルチモニターDPI変更時のちらつき問題に関する調査結果と設計判断の根拠を記録する。

## Technology Alignment

### bevy_ecs Changed<T>検知

**調査内容:** `Changed<T>`クエリの動作確認

**結果:**
- `Changed<T>`は同一tick内での変更を検知
- コンポーネントへの代入で変更検知が発火
- `bypass_change_detection()`で変更検知をスキップ可能

**影響:** `WindowPosChanged`フラグのリセット時は変更検知を発火させない必要がある場合がある（現設計では不要）

### SparseSetストレージ

**調査内容:** `#[component(storage = "SparseSet")]`の適用性

**結果:**
- エンティティ数が少なく、頻繁にアクセスされるコンポーネントに適切
- Windowエンティティは通常1-10個程度
- `WindowPosChanged`はWindowエンティティのみに配置

**決定:** `WindowPosChanged`に`SparseSet`ストレージを使用

### スレッドローカル vs Resource

**調査内容:** `DpiChangeContext`と`SetWindowPosCommand`の配置場所

**選択肢:**
1. ECS Resource として World 内に配置
2. スレッドローカル変数として World 外に配置

**分析:**
| 観点 | Resource | Thread-local |
|------|----------|--------------|
| World借用中のアクセス | 不可 | 可能 |
| 型安全性 | 高い | RefCell使用 |
| ECS統合 | ネイティブ | 外部 |

**決定:** Thread-local を採用（World借用競合の根本原因が World アクセスのため）

## Boundary Decisions

### WndProc ↔ ECS World 境界

**課題:** WndProc は Win32 から同期的に呼び出され、ECS World の借用状態を制御できない

**決定:**
1. DPI情報の伝達はスレッドローカルで行う（World借用不要）
2. SetWindowPos はキューに追加し、tick後に実行（World借用解放後）
3. `WindowPosChanged`フラグはWorld借用可能時にのみ設定

**根拠:** World借用状態に依存しない設計により、再入時のエラーを防止

### apply_window_pos_changes システム境界

**課題:** 既存システムは直接`SetWindowPos`を呼び出しており、再入を引き起こす

**決定:**
1. `SetWindowPos`呼び出しをキュー追加に変更
2. フラグチェックを追加（`WindowPosChanged`）
3. 既存のエコーバック判定は残す（last_sent_*フィールド）

**根拠:** 
- キュー方式で再入を完全に防止
- フラグ方式で1回目のエコーバックを即座に抑制
- 既存エコーバック判定は通常操作時のフィルタリングに有効

## API Investigation

### DefWindowProcW と WM_DPICHANGED

**調査内容:** `DefWindowProcW`呼び出し時の`SetWindowPos`発行タイミング

**結果:**
- `DefWindowProcW(hwnd, WM_DPICHANGED, wparam, lparam)`内で同期的に`SetWindowPos`が呼ばれる
- `suggested_rect`（lparam）がそのまま使用される
- `WM_WINDOWPOSCHANGED`は`SetWindowPos`内で同期的に発火

**決定:** `DpiChangeContext`を`DefWindowProcW`呼び出し前に設定する

### SetWindowPos と WM_WINDOWPOSCHANGED

**調査内容:** `SetWindowPos`から`WM_WINDOWPOSCHANGED`への遷移

**結果:**
- `SetWindowPos`は同期的に`WM_WINDOWPOSCHANGED`を送信
- `WM_WINDOWPOSCHANGING`も先に送信されるが、本機能では使用しない
- `SWP_NOSENDCHANGING`フラグで`WM_WINDOWPOSCHANGING`を抑制可能（本機能では不要）

**決定:** 同期的な再入を前提として設計

## Performance Considerations

### キュー方式のオーバーヘッド

**調査内容:** `Vec`へのpush/popコスト

**結果:**
- 通常1-2個のコマンドのみ（ウィンドウ数が少ない）
- アロケーションは初回のみ（`Vec`は再利用）
- tick頻度は60Hz程度（VSYNCベース）

**決定:** パフォーマンス影響は無視できるレベル

### Changed<T>クエリのコスト

**調査内容:** `Changed<WindowPos>`と`WindowPosChanged`フラグの両方をチェックするコスト

**結果:**
- `Changed<T>`は既存で使用済み
- `WindowPosChanged`追加チェックはbool比較のみ
- クエリ対象はWindowエンティティのみ（少数）

**決定:** 追加コストは無視できる

## Risk Assessment

### リスク1: 同一tick内の複数WM_WINDOWPOSCHANGED

**シナリオ:** ウィンドウ移動＋リサイズが同時に発生

**影響:** 2回目以降のフラグ設定は既にtrue（問題なし）

**対策:** 不要（フラグ方式で自然に対応）

### リスク2: アプリ変更と同tick内のWM_WINDOWPOSCHANGED

**シナリオ:** アプリがBoxStyleを変更→同tick内でWM_WINDOWPOSCHANGEDが発火

**影響:** アプリの変更が次tickまで遅延

**対策:** Known Limitation として文書化（稀なケースで許容可能）

### リスク3: flushタイミングのミス

**シナリオ:** World借用中にflush_window_pos_commands()を呼んでしまう

**影響:** 二重借用エラー

**対策:** 
- `VsyncTick`トレイト実装でのみflush呼び出し
- コードレビューで確認

## Deprecated Approaches

### PostMessage方式（現行実装）

**廃止理由:**
1. 非同期のため`WM_WINDOWPOSCHANGED`処理時に旧DPIを使用
2. メッセージキューへの依存（タイミング不確定）
3. コード複雑性の増加

### 物理座標ベースのエコーバック検知（REQ-003, REQ-004）

**廃止理由:**
1. 丸め誤差が蓄積する前にフラグで抑制可能
2. 座標比較のロジックが複雑
3. シンプルなboolフラグで十分

### ドラッグ中フラグ（REQ-003）

**廃止理由:**
1. `WindowPosChanged`フラグで代替可能
2. ドラッグ状態の追跡が不要に
3. ユーザー操作種別に依存しない設計

## References

- [WM_DPICHANGED](https://learn.microsoft.com/en-us/windows/win32/hidpi/wm-dpichanged)
- [SetWindowPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos)
- [bevy_ecs Component Storage](https://docs.rs/bevy_ecs/latest/bevy_ecs/component/index.html)
