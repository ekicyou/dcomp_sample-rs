# Requirements Document

## Project Description (Input)

taffyを導入し、レイアウト構造の基礎を実現する。
1. 名称変更「BoxStyle,BoxComputedLayout」⇒「TaffyStyle,TaffyComputedLayout」
2. TaffyStyleを組み立てるためのコンポーネント(positionなどの内部要素をジャンル別に分解したもの、または分解せずに利用？)
3. (2)で用意した個別要素をTaffyStyleに組み立てるシステム
4. widgetツリーからTaffyStyleを組み立てて、taffyにツリー計算させるシステム。出力はBoxComputedLayoutに行う
5. 更新されたBoxComputedLayoutより、Arrangementを更新するシステム
6. レイアウト計算を増分的に行い、毎回全部やり直すことがないようにする仕組みの構築。
7. その他taffyレイアウトを構成するために必要なシステム。

## Requirements
<!-- Will be generated in /kiro-spec-requirements phase -->
