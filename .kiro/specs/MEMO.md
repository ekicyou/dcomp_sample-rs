# AIに与える指示など

## 初期化
/kiro-spec-init
EcsWorldへのアクセスをUnsafeCell化して、wndprocの再入に対して自然に処理したい。メッセージループはシングルスレッド動作なので本質的に安全なはずである。

## ログの問題点
- `deferred_surface_creation_system`が動いている形跡がない。

## ログ
```log

```

## 確認
