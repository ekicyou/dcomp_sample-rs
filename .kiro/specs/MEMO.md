# AIに与える指示など

## 初期化
/kiro-spec-init
「cargo run --example taffy_flex_demo」デモにて、ウィンドウをマウスで画面移動させている間、ウィンドウ内の描画が行われずブロックされているように見える。メッセージの投入状況を調査し、場合によってはWM_VSYNC処理をメッセージからtick_countフラグに変更したうえで値の変化でworld tickを呼び出すようにして、メッセージループ処理よりVSYNCのワールド更新を優先させるように変更して欲しい。

## ログの問題点
- `deferred_surface_creation_system`が動いている形跡がない。

## ログ
```log

```

## 確認
