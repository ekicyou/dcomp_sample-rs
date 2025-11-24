# AIに与える指示など

## 初期化
```sh
/kiro-spec-init "
BoxSize,BoxMargin,BoxPaddingなど、build_taffy_styles_systemに関わるクエリが巨大になってきて性能不安がある。本質的にレイアウト入力の論理コンポーネントは分離している意義があまりない。そのためコンポーネントを1つにまとめてBoxStyleにしてしまうほうがよいのではないか？各フィールドをOption<BoxSize>などにして、従来コンポーネントだったBoxSizeなどはコンポーネントでなくする。実装
"
```

## 要件不安への回答




