# Requirements Document

## Project Description (Input)
pastaエンジンにおいて、会話行の「＠会話」から呼び出される要素について、以下の優先順位で決定してほしい。

1. ローカル関数（名称完全一致）
2. グローバル関数（名称完全一致）
3. ローカルとグローバルの単語辞書より、名称が前方一致するキーの全要素から一覧を作りシャッフル、１つずつ取り出す。同じ検索キーの検索結果はキャッシュして要素が残っていればpullしていく。

## Requirements
<!-- Will be generated in /kiro:spec-requirements phase -->
