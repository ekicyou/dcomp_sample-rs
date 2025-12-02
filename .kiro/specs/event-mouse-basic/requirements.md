# Requirements Document

## Project Description (Input)
wintf-P0-event-system の孫仕様：マウス基本イベント処理

親仕様の Requirement 3（マウスクリックイベント）と Requirement 4（マウスホバーイベント）を実装する。
クリック、ダブルクリック、右クリック、ホバー（Enter/Leave/Move）イベントを処理する。

### 対応する親仕様の要件
- **Requirement 3**: マウスクリックイベント
- **Requirement 4**: マウスホバーイベント

### 親仕様からのAcceptance Criteria
- 左クリック: MouseDown/MouseUp/Click イベント
- 右クリック: RightClick イベント
- ダブルクリック: DoubleClick イベント
- クリック位置（画面座標、ローカル座標）
- クリック領域名
- MouseEnter/MouseLeave イベント
- 継続的な MouseMove イベント
- カーソル移動速度（撫でる操作検出用）
- 適切な順序での Enter/Leave イベント発火

### event-hit-test からの引き継ぎ事項

`event-hit-test` 仕様で実装されたヒットテストAPIを本仕様で統合する必要がある。
以下は `event-hit-test` でスコープ外とされた項目であり、本仕様で対応が必要：

| 引き継ぎ項目 | 説明 | 優先度 |
|-------------|------|--------|
| **ecs_wndproc 統合** | `WM_MOUSEMOVE`, `WM_LBUTTONDOWN` 等のハンドラから `hit_test` API を呼び出す | 必須 |
| **ローカル座標変換** | `GlobalArrangement.transform.inverse()` によるエンティティローカル座標への変換 | 必須 |
| **hit_test_detailed** | ローカル座標付きヒット結果を返す関数（`HitTestResult { entity, local_point }`） | 必須 |
| **キャッシュ機構** | 座標が同一の場合は前回結果を返す、`ArrangementTreeChanged` でキャッシュ無効化 | オプション |

**参照**: `.kiro/specs/event-hit-test/requirements.md` の Requirement 5, 7, 8

## Requirements
<!-- Will be generated in /kiro-spec-requirements phase -->
