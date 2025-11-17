# Requirements: phase4-mini-horizontal-text

**Feature ID**: `phase4-mini-horizontal-text`  
**Created**: 2025-11-17  
**Status**: Requirements Generated

---

## Introduction

本要件定義は、DirectWriteを統合し横書きテキストレンダリングの最小実装を行う`phase4-mini-horizontal-text`機能の詳細要件を定義する。縦書きテキスト実装への第一歩として、DirectWriteの基本統合、横書きテキスト表示、Labelウィジットの基本実装を対象とする。

本機能は「モチベーションGO!ルート（ルートA）」として位置づけられ、既存のDirectComposition/Direct2Dベースの描画システムにテキスト描画機能を追加する。

### 重要な設計方針

**本実装は横書きテキストに焦点を当てるが、近い将来の縦書き対応（Phase 7）を前提として設計する。** そのため、以下の点に留意する：

- API設計は横書き専用にせず、将来の方向指定拡張を考慮
- コンポーネント構造は縦書き固有のプロパティ追加に対応可能な設計
- テキストレイアウト生成ロジックは方向性に依存しない形で実装
- 命名規則は「Horizontal」等の方向を含めず、汎用的な名称を使用（例：`Label`であり`HorizontalLabel`ではない）

---

## Requirements

### Requirement 1: DirectWrite Factory統合

**Objective:** システム開発者として、DirectWrite APIをwintfフレームワークに統合したい。これにより、テキストレンダリングのための基盤が準備される。

#### Acceptance Criteria

1. When GraphicsCoreが初期化される時、wintfシステムはIDWriteFactory7インスタンスを作成して保持しなければならない
2. When DirectWriteファクトリー作成に失敗した時、wintfシステムは適切なエラーメッセージを返さなければならない
3. The wintfシステムは、GraphicsCoreリソース内でIDWriteFactory7へのスレッドセーフなアクセスを提供しなければならない

---

### Requirement 2: テキストフォーマット作成

**Objective:** システム開発者として、テキストのフォント・サイズ・スタイルを指定できるようにしたい。これにより、多様なテキスト表現が可能になる。

#### Acceptance Criteria

1. When Labelコンポーネントにフォントファミリー名が指定された時、wintfシステムはIDWriteTextFormatを作成しなければならない
2. When フォントサイズが指定された時、wintfシステムは指定されたサイズ（pt単位）でIDWriteTextFormatを作成しなければならない
3. The wintfシステムは、最低限「メイリオ」フォントファミリーをサポートしなければならない
4. The wintfシステムは、フォントサイズ範囲として8.0pt～72.0ptをサポートしなければならない
5. If 指定されたフォントファミリーが存在しない場合、wintfシステムはシステムデフォルトフォントにフォールバックしなければならない

---

### Requirement 3: テキストレイアウト生成

**Objective:** システム開発者として、テキスト文字列からレンダリング可能なレイアウトオブジェクトを生成したい。これにより、実際の描画準備が整う。

#### Acceptance Criteria

1. When LabelコンポーネントのtextフィールドにUTF-8文字列が設定された時、wintfシステムはIDWriteTextLayoutを作成しなければならない
2. The wintfシステムは、ASCII文字列と日本語文字列の両方からIDWriteTextLayoutを生成できなければならない
3. When TextLayoutが作成された時、wintfシステムはそれをTextLayoutコンポーネントとしてキャッシュしなければならない
4. When Labelコンポーネントが変更(Changed<Label>)された時、wintfシステムは対応するTextLayoutを再生成しなければならない
5. If TextLayout生成に失敗した場合、wintfシステムはエラーログを出力し、該当エンティティの描画をスキップしなければならない

---

### Requirement 4: Labelウィジットコンポーネント

**Objective:** アプリケーション開発者として、ECSエンティティにLabelコンポーネントを追加することで、テキストを表示したい。これにより、宣言的なテキスト表示が可能になる。

#### Acceptance Criteria

1. The wintfシステムは、以下のフィールドを持つLabelコンポーネントを提供しなければならない:
   - `text: String` (表示するテキスト)
   - `font_family: String` (フォントファミリー名)
   - `font_size: f32` (フォントサイズ pt単位)
   - `color: D2D1_COLOR_F` (テキスト色)
   - `x: f32` (X座標)
   - `y: f32` (Y座標)
2. The Labelコンポーネントは、bevy_ecsのComponentトレイトを実装しなければならない
3. The Labelコンポーネントは、Default実装を提供し、デフォルト値（"", "メイリオ", 16.0, 黒色, 0.0, 0.0）を設定しなければならない
4. When Labelコンポーネントがエンティティに追加された時、そのエンティティはWindowHandleコンポーネントも持っていなければならない

---

### Requirement 5: TextLayoutコンポーネント（キャッシュ）

**Objective:** システム開発者として、生成したTextLayoutをキャッシュして再利用したい。これにより、パフォーマンスが向上する。

#### Acceptance Criteria

1. The wintfシステムは、IDWriteTextLayoutをラップするTextLayoutコンポーネントを提供しなければならない
2. When Labelコンポーネントが変更されない時、wintfシステムは既存のTextLayoutコンポーネントを再利用しなければならない
3. When Labelコンポーネントが削除された時、wintfシステムは対応するTextLayoutコンポーネントも削除しなければならない
4. The TextLayoutコンポーネントは、COMオブジェクトのライフタイムを適切に管理しなければならない

---

### Requirement 6: draw_labelsシステム実装

**Objective:** システム開発者として、Labelコンポーネントを持つエンティティを自動的に描画したい。これにより、フレームワークが自動的にテキストを描画する。

#### Acceptance Criteria

1. The wintfシステムは、Drawスケジュールで実行されるdraw_labelsシステムを提供しなければならない
2. When draw_labelsシステムが実行される時、LabelとWindowHandleコンポーネントを持つ全てのエンティティをクエリしなければならない
3. When Labelコンポーネントが変更された時、draw_labelsシステムは新しいTextLayoutを生成しなければならない
4. When TextLayoutが存在する時、draw_labelsシステムはID2D1DeviceContextのDrawTextLayoutメソッドを呼び出してテキストを描画しなければならない
5. The draw_labelsシステムは、render_surfaceシステムの前に実行されなければならない
6. When 複数のLabelエンティティが存在する時、draw_labelsシステムは全てのLabelを描画しなければならない

---

### Requirement 7: 横書きテキスト描画

**Objective:** エンドユーザーとして、横書きのテキストが正しく表示されることを期待する。これにより、基本的なテキスト表示機能が完成する。

#### Acceptance Criteria

1. When Labelコンポーネントに"Hello, World!"が設定された時、wintfシステムは指定された座標にASCII文字列を横書きで描画しなければならない
2. When Labelコンポーネントに"こんにちは"が設定された時、wintfシステムは指定された座標に日本語文字列を横書きで描画しなければならない
3. The wintfシステムは、テキストをアンチエイリアシング処理して滑らかに描画しなければならない
4. When テキスト色が指定された時、wintfシステムは指定された色でテキストを描画しなければならない
5. When フォントサイズが指定された時、wintfシステムは指定されたサイズでテキストを描画しなければならない

---

### Requirement 8: 複数Label表示

**Objective:** アプリケーション開発者として、同一ウィンドウ内に複数のLabelを表示したい。これにより、実用的なUIが構築できる。

#### Acceptance Criteria

1. When 単一Windowエンティティに対して複数のLabelエンティティが存在する時、wintfシステムは全てのLabelを描画しなければならない
2. When 各Labelが異なる座標(x, y)を持つ時、wintfシステムはそれぞれ指定された位置にテキストを描画しなければならない
3. When 各Labelが異なる色を持つ時、wintfシステムはそれぞれ指定された色でテキストを描画しなければならない
4. The wintfシステムは、Labelの描画順序を保証しなければならない（エンティティID順）

---

### Requirement 9: パフォーマンス要件

**Objective:** エンドユーザーとして、滑らかなテキスト表示を期待する。これにより、快適なユーザー体験が提供される。

#### Acceptance Criteria

1. The wintfシステムは、10個のLabelを表示した状態で60fps以上のフレームレートを維持しなければならない（Vsync同期により環境によっては60fpsに制限される）
2. When Labelコンポーネントが変更されない時、wintfシステムはTextLayout再生成をスキップしなければならない
3. The wintfシステムは、DrawTextLayout呼び出しをGraphicsCommandListにバッチングしなければならない
4. When ウィンドウが非表示の時、wintfシステムは該当ウィンドウのLabel描画をスキップしなければならない

---

### Requirement 10: エラーハンドリング

**Objective:** システム開発者として、エラーが発生した場合に適切に処理したい。これにより、システムの安定性が向上する。

#### Acceptance Criteria

1. If DirectWriteファクトリー作成に失敗した場合、wintfシステムはエラーログを出力してGraphicsCore初期化を失敗させなければならない
2. If TextFormat作成に失敗した場合、wintfシステムはエラーログを出力して該当Labelの描画をスキップしなければならない
3. If TextLayout作成に失敗した場合、wintfシステムはエラーログを出力して該当Labelの描画をスキップしなければならない
4. If DrawTextLayout呼び出しに失敗した場合、wintfシステムはエラーログを出力してフレーム描画を継続しなければならない
5. The wintfシステムは、全てのCOM APIエラーをwindows::core::Resultとして伝播しなければならない

---

### Requirement 11: サンプルアプリケーション

**Objective:** アプリケーション開発者として、Labelウィジットの使用例を参照したい。これにより、機能の使い方が理解できる。

#### Acceptance Criteria

1. The wintfプロジェクトは、Labelウィジットを使用したサンプルアプリケーションを提供しなければならない
2. The サンプルアプリケーションは、既存の`simple_window.rs`を拡張するか、`simple_window.rs`を参考にした新規サンプルとして実装しなければならない
3. When サンプルアプリケーションが実行された時、"Hello, World!"と"こんにちは"が表示されなければならない
4. The サンプルアプリケーションは、異なるフォントサイズと色の複数のLabelを表示しなければならない
5. If 新規サンプルとして実装する場合、`examples/`ディレクトリに配置し、`cargo run --example {sample_name}`で実行可能でなければならない
6. The サンプルアプリケーションは、`simple_window.rs`と同様のタイマースレッドパターンを使用して動的なテキスト変更をデモンストレーションすることが望ましい

---

## Non-Functional Requirements

### Performance
- TextLayout生成のキャッシングにより、変更されていないLabelの再描画コストを最小化
- DirectCompositionによるハードウェアアクセラレーション活用

### Maintainability
- COM APIラッパーは`com/dwrite.rs`に集約
- ECSコンポーネントとシステムは`ecs/widget/text/`に配置
- 既存のshapesモジュールとの一貫性を保つ

### Extensibility（拡張性）
- **縦書き対応の前提**: Phase 7での縦書き実装を見据えた設計とする
- API命名は方向性に依存しない汎用的な名称を使用（`draw_labels`であり`draw_horizontal_labels`ではない）
- Labelコンポーネント構造は将来の`writing_mode`フィールド追加に対応可能な設計
- DirectWriteのIDWriteTextLayoutはREADING_DIRECTION/FLOW_DIRECTIONをサポートしており、本実装で使用するAPIは縦書きにも適用可能

### Testability
- TextFormat/TextLayout生成はユニットテスト可能
- サンプルアプリケーションによる統合テスト

### Compatibility
- Windows 10 1809以降（DirectWrite対応）
- Rust 2021 Edition
- bevy_ecs 0.17.2

---

## Out of Scope

以下の機能は本要件の対象外:

- **Button実装**: Phase 7以降で実装
- **複雑なレイアウト**: テキスト折り返しや複数段落は最小限のサポート
- **イベント処理**: クリック等のインタラクションは対象外
- **縦書きテキスト**: **Phase 7で必ず実装予定**
  - 本Phase 4の成果物は縦書き対応の基盤として機能する
  - 縦書き実装時には、Labelコンポーネントへの`writing_mode`または`direction`フィールド追加を想定
  - DirectWriteのIDWriteTextLayoutは縦書きをネイティブサポートしており、本実装で使用するAPIは縦書きにも対応可能
  - API設計は横書き専用にせず、将来の拡張性を考慮すること
- **テキスト編集**: 表示のみ、編集機能は対象外
- **リッチテキスト**: 単一フォーマットのみ、部分的な装飾は対象外

---

## Success Criteria Summary

1. ✅ "Hello, World!"が表示される
2. ✅ 日本語（"こんにちは"）が表示される
3. ✅ フォント・サイズ・色が指定できる
4. ✅ 複数のLabelが同時表示可能
5. ✅ 60fps以上のパフォーマンスを維持（Vsync同期環境）

---

## Next Steps

要件承認後、以下のコマンドで設計フェーズに進む:

```bash
/kiro-spec-design phase4-mini-horizontal-text -y
```

または、要件にフィードバックがある場合:

```bash
/kiro-spec-requirements phase4-mini-horizontal-text
```

---

_Requirements generated on 2025-11-17_
