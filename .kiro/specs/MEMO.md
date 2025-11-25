# AIに与える指示など

## 初期化
```sh
/kiro-spec-init "
BoxSize,BoxMargin,BoxPaddingなど、build_taffy_styles_systemに関わるクエリが巨大になってきて性能不安がある。本質的にレイアウト入力の論理コンポーネントは分離している意義があまりない。そのためコンポーネントを1つにまとめてBoxStyleにしてしまうほうがよいのではないか？各フィールドをOption<BoxSize>などにして、従来コンポーネントだったBoxSizeなどはコンポーネントでなくする。実装可否判断を含め検討せよ。
"
```

## インタラクティブ確認
上記2点の指摘について、ご意見や代替案があればお聞かせください：

1. flex_grow/flex_shrinkはOption<f32>のまま進めるか、taffyデフォルト値を持つf32に変更するか？
   ＞ flex_grow/flex_shrinkとは？内容を聞いてから確認したい。

2. LayoutRootのみのエンティティではBoxStyleを必須にするか、空スタイル適用で許容するか？
　　＞ LayoutRootは現在Desktopエンティティで、レイアウト要素として絶対座標での矩形を持つべき。現在も持ってないか？