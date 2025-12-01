# Requirements Document

## Project Description (Input)
wintf-P0-event-system の孫仕様：ドラッグシステム

親仕様の Requirement 5（ドラッグイベント）と Requirement 6（ウィンドウドラッグ移動）を実装する。
エンティティのドラッグ操作とウィンドウ全体の移動機能を提供する。

### 対応する親仕様の要件
- **Requirement 5**: ドラッグイベント
- **Requirement 6**: ウィンドウドラッグ移動

### 親仕様からのAcceptance Criteria
- DragStart/Drag/DragEnd イベント
- ドラッグ開始位置と現在位置の差分
- ドラッグ対象エンティティ識別
- ドラッグ閾値（例: 5ピクセル）によるDragStart発火
- ドラッグキャンセル（Escキー等）
- ウィンドウ位置のリアルタイム更新
- マルチモニター環境サポート
- ウィンドウ移動の有効/無効切り替え
- ドラッグ終了時の最終位置通知

## Requirements
<!-- Will be generated in /kiro-spec-requirements phase -->
