# AIに与える指示など

## 初期化
```sh
/kiro-spec-init "
仮想デスクトップとディスプレイをエンティティ管理する。
仮想デスクトップをルートとしたツリーを構築し、
仮想デスクトップを頂点としたtaffyレイアウト計算を可能にする。
"
```

1. VirtualDesktop,Monitorコンポーネントを定義
2. エンティティ階層　VirtualDesktop　⇒Monitor　⇒Window
3. taffyレイアウトのルート要素をVirtualDesktopとする。
