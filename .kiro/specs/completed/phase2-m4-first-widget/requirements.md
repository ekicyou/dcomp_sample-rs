# Requirements: Phase 2 Milestone 4 - 初めてのウィジット

**Feature ID**: `phase2-m4-first-widget`  
**Version**: 2.0  
**Last Updated**: 2025-11-14

---

## Introduction

本仕様は、Phase 2 Milestone 4「初めてのウィジット」の要件を定義します。Widgetの基本構造（Rectangle、GraphicsCommandList）を確立し、DrawスケジュールでのCommandList生成と描画ワークフローを実装します。

これはPhase 2の最終マイルストーンであり、**Widget描画の基盤が完成する**重要なステップです。

### アーキテクチャの方針

**シンプルなスタート**: このマイルストーンでは、VisualツリーやEntity階層構造は構築しません。WindowエンティティにRectangleコンポーネントを直接追加することで、Widget関係の基本構造のみを確立します。

**Entity構成**: 
```
Window Entity {
    Window, WindowHandle, WindowGraphics, Visual, Surface,
    Rectangle,           // NEW! Widget コンポーネント
    GraphicsCommandList, // NEW! 描画命令
}
```

### 前提条件
- **Milestone 1完了**: `phase2-m1-graphics-core` - GraphicsCoreが正常に初期化されていること
- **Milestone 2完了**: `phase2-m2-window-graphics` - WindowGraphicsとVisualコンポーネントが作成されていること
- **Milestone 3完了**: `phase2-m3-first-rendering` - Surfaceへの描画が正常に動作していること

### スコープ外
- Visualツリー構築（次フェーズ）
- Entity階層構造（ChildOf/Children）
- Transform伝播システムとの統合
- 複数のWidget Entity管理
- レイアウトシステム
- イベント処理（Phase 6で対応）

---

## Requirements

### Requirement 1: Rectangleコンポーネントの定義

**目的:** 開発者として、四角形の描画情報を宣言的に定義することで、Widget描画の基盤を確立したい

#### Acceptance Criteria

1. The Rectangle component shall 位置情報（x, y）を保持する

2. The Rectangle component shall サイズ情報（width, height）を保持する

3. The Rectangle component shall 塗りつぶし色（Color）を保持する

4. The Rectangle component shall Component, Debug, Cloneトレイトを派生する

5. The Rectangle component shall crates/wintf/src/ecs/widget/shapes/rectangle.rsに配置される

6. The widget::shapes module shall pub mod rectangle; pub use rectangle::Rectangle; でエクスポートする

7. The Rectangle struct shall f32型で座標とサイズを保持する

8. The Color type shall D2D1_COLOR_Fと互換性を持つ（rgba: f32 x 4）

### Requirement 2: GraphicsCommandListコンポーネントの定義

**目的:** システム管理者として、Direct2Dの描画命令を保持することで、効率的な描画パイプラインを実現したい

#### Acceptance Criteria

1. The GraphicsCommandList component shall ID2D1CommandListのフィールドを保持する

2. The GraphicsCommandList component shall Component, Debugトレイトを派生する

3. The GraphicsCommandList component shall Send + Syncトレイトを実装する（unsafe impl）

4. The GraphicsCommandList component shall crates/wintf/src/ecs/graphics/command_list.rsに配置される

5. The GraphicsCommandList component shall ID2D1CommandListへの参照を取得するメソッドを提供する

6. The GraphicsCommandList struct shall ID2D1CommandListのライフタイムをwindows-rsのスマートポインターで管理する

### Requirement 3: graphics.rsのモジュール化

**目的:** アーキテクトとして、graphics.rsを責務ごとにモジュール分割することで、保守性と可読性を向上させたい

#### Acceptance Criteria

1. The graphics module shall graphics.rsをgraphics/ディレクトリに変換する

2. When モジュール化する, the graphics module shall 以下のファイル構造を持つ:
   - `graphics/mod.rs` - 公開API + Re-exports
   - `graphics/core.rs` - GraphicsCore リソース
   - `graphics/components.rs` - WindowGraphics, Visual, Surface コンポーネント
   - `graphics/command_list.rs` - GraphicsCommandList コンポーネント
   - `graphics/systems.rs` - 描画システム群（create_window_graphics, create_window_visual, create_window_surface, render_window, render_surface, commit_composition）

3. The graphics/mod.rs shall すべての公開型とシステムをre-exportする

4. When モジュール化する, the refactoring shall 既存のコードを移動するのみで、機能を変更しない

5. When モジュール化する, the refactoring shall 既存のテストがすべてパスすることを確認する

6. The graphics/core.rs shall GraphicsCore構造体とensure_graphics_coreシステムを含む

7. The graphics/components.rs shall WindowGraphics, Visual, Surface構造体とそれらのアクセサメソッドを含む

8. The graphics/systems.rs shall すべての描画システム関数を含む

9. When 他モジュールからimportする, the usage shall `use crate::ecs::graphics::*;` で既存と同じように動作する

10. The refactoring shall Phase 2-M4の他の機能実装の前に完了する

### Requirement 4: draw_rectanglesシステムの実装

**目的:** 開発者として、RectangleコンポーネントからGraphicsCommandListを自動生成することで、描画処理を自動化したい

#### Acceptance Criteria

1. When Rectangleコンポーネントが変更される, the draw_rectangles system shall Query<(Entity, &Rectangle), Changed<Rectangle>>を使用して変更を検出する

2. When Rectangleが検出される, the draw_rectangles system shall GraphicsCoreのd2d_factoryからID2D1CommandListを作成する

3. When CommandListを作成する, the draw_rectangles system shall factory.CreateCommandList()を呼び出す

4. When CommandListが作成される, the draw_rectangles system shall command_list.Open()でID2D1DeviceContextを取得する

5. When DeviceContextが取得される, the draw_rectangles system shall dc.BeginDraw()を呼び出す

6. When 描画命令を記録する, the draw_rectangles system shall Rectangleの座標・サイズ・色に基づいてFillRectangle()を呼び出す

7. When 描画命令の記録が完了する, the draw_rectangles system shall dc.EndDraw()を呼び出す

8. When EndDraw()が完了する, the draw_rectangles system shall command_list.Close()を呼び出す

9. When CommandListが完成する, the draw_rectangles system shall GraphicsCommandListコンポーネントをエンティティに挿入または更新する

10. When システムが実行される, the draw_rectangles system shall 開始時にEntity IDをログに出力する

11. When CommandListが生成される, the draw_rectangles system shall 成功をログに出力する（Entity ID、Rectangle情報を含む）

12. If CommandList生成中にエラーが発生する, then the draw_rectangles system shall エラー内容をログに出力し、そのエンティティの処理をスキップする

13. If GraphicsCoreが存在しない, then the draw_rectangles system shall 警告ログを出力して処理全体をスキップする

14. The draw_rectangles system shall Drawスケジュールで実行される

15. The draw_rectangles system shall crates/wintf/src/ecs/widget/shapes/rectangle.rsに配置される

### Requirement 5: render_surfaceシステムの実装

**目的:** システム管理者として、GraphicsCommandListをSurfaceに描画することで、Widgetを画面に表示したい

#### Acceptance Criteria

1. When GraphicsCommandListコンポーネントが存在する, the render_surface system shall Query<(Entity, &GraphicsCommandList, &Surface)>を使用して対象エンティティを検出する

2. When 描画を開始する, the render_surface system shall Changed<GraphicsCommandList>またはChanged<Surface>をトリガーとして実行する

3. When 描画を開始する, the render_surface system shall Surface.BeginDraw()でID2D1DeviceContextを取得する

4. When DeviceContextが取得される, the render_surface system shall dc.Clear()で透明色（rgba: 0.0, 0.0, 0.0, 0.0）をクリアする

5. When Clearが完了する, the render_surface system shall dc.DrawImage()でCommandListを描画する

6. When DrawImage()を呼び出す, the render_surface system shall GraphicsCommandListから取得したID2D1CommandListを引数として渡す

7. When CommandListの描画が完了する, the render_surface system shall dc.EndDraw()を呼び出す

8. When dc.EndDraw()が完了する, the render_surface system shall Surface.EndDraw()を呼び出す

9. When システムが実行される, the render_surface system shall 開始時にEntity IDをログに出力する

10. When 描画が完了する, the render_surface system shall 成功をログに出力する（Entity IDを含む）

11. If 描画中にエラーが発生する, then the render_surface system shall エラー内容をログに出力し、そのエンティティの描画をスキップする

12. If GraphicsCoreが存在しない, then the render_surface system shall 警告ログを出力して処理全体をスキップする

13. The render_surface system shall Renderスケジュールで実行される

14. The render_surface system shall Query<(Entity, &GraphicsCommandList, &Surface), Or<(Changed<GraphicsCommandList>, Changed<Surface>)>>を使用する

15. The render_surface system shall crates/wintf/src/ecs/graphics/systems.rsに配置される

### Requirement 6: 既存render_windowシステムの削除

**目的:** アーキテクトとして、Phase 2-M3の描画テストシステムをスケジュールから削除し、新しいWidget描画パイプラインに完全移行したい

#### Acceptance Criteria

1. The render_window system shall Renderスケジュールからの登録を削除する

2. The render_window system shall 削除後はWithout<GraphicsCommandList>フィルターでクリア処理のみを実行する

3. When GraphicsCommandListが存在しない, the render_window system shall 透明色（rgba: 0.0, 0.0, 0.0, 0.0）でクリアするのみ

4. The render_shapes helper function shall コードは残すが、呼び出しを削除する（将来の参考コードとして保持）

5. The create_triangle_geometry helper function shall コードは残すが、呼び出しを削除する（将来の参考コードとして保持）

6. The render_window system shall 将来的にWithout<GraphicsCommandList>を持つ特殊なWidgetのための予約システムとして残す

7. The Phase 2-M3 drawing code shall 参考実装として graphics.rs（またはgraphics/systems.rs）に残す

### Requirement 7: 既存render_windowシステムとの共存

### Requirement 7: render_surfaceとrender_windowの分離

**目的:** アーキテクトとして、新しいrender_surfaceシステムと既存のrender_windowシステムを明確に分離することで、段階的な移行を可能にしたい

#### Acceptance Criteria

1. The render_window system shall GraphicsCommandListコンポーネントを持たないエンティティのみを処理する

2. The render_window system shall Query<(Entity, &Surface), (Added<Surface>, Without<GraphicsCommandList>)>を使用する

3. The render_surface system shall GraphicsCommandListコンポーネントを持つエンティティのみを処理する

4. The render_surface system shall Query<(Entity, &GraphicsCommandList, &Surface), Added<Surface>>を使用する

5. When 両システムが実行される, the ECS scheduler shall 同一エンティティに対して両方のシステムが実行されないことを保証する

### Requirement 8: COM APIラッパーの拡張

**目的:** 開発者として、ID2D1CommandList関連のCOM APIを安全に使用することで、実装を簡素化したい

#### Acceptance Criteria

1. The D2D1FactoryExt trait shall create_command_list()メソッドを提供する

2. When create_command_list()が呼ばれる, the D2D1FactoryExt shall unsafe { self.CreateCommandList() } を実行する

3. The D2D1CommandListExt trait shall open()メソッドを提供する

4. When open()が呼ばれる, the D2D1CommandListExt shall unsafe { self.Open() } を実行してID2D1DeviceContextを返す

5. The D2D1CommandListExt trait shall close()メソッドを提供する

6. When close()が呼ばれる, the D2D1CommandListExt shall unsafe { self.Close() } を実行する

7. The D2D1DeviceContextExt trait shall draw_image()メソッドを提供する

8. When draw_image()が呼ばれる, the D2D1DeviceContextExt shall unsafe { self.DrawImage(image, None, None, D2D1_INTERPOLATION_MODE_LINEAR, D2D1_COMPOSITE_MODE_SOURCE_OVER) } を実行する

9. All COM API wrappers shall crates/wintf/src/com/d2d/mod.rsに配置される

### Requirement 9: エラーハンドリングとログ出力

**目的:** 開発者として、詳細なログ出力により、Widget描画の動作を理解し、問題を迅速に診断したい

#### Acceptance Criteria

1. When システムが実行される, the All Systems shall 開始時にシステム名とEntity IDをログに出力する

2. When COM API呼び出しが成功する, the All Systems shall 成功メッセージをログに出力する

3. When COM API呼び出しが失敗する, the All Systems shall エラー内容（Entity ID、HRESULT）をログに出力する

4. When エラーが発生する, the All Systems shall パニックせず、そのエンティティの処理をスキップして継続する

5. When GraphicsCoreが存在しない, the All Systems shall 警告ログを出力し、処理全体をスキップする

6. When Rectangleの情報をログ出力する, the draw_rectangles system shall 座標、サイズ、色を含む

### Requirement 10: 統合テストとsimple_window.rsの更新

**目的:** 開発者として、動作するサンプルを見ることで、Widget描画の使い方を理解したい

#### Acceptance Criteria

1. The simple_window example shall 1つ目のウィンドウに赤い四角を設定する

2. When 赤い四角を設定する, the example shall Rectangle { x: 100.0, y: 100.0, width: 200.0, height: 150.0, color: RED }を定義する

3. The simple_window example shall 2つ目のウィンドウに青い四角を設定する

4. When 青い四角を設定する, the example shall Rectangle { x: 150.0, y: 150.0, width: 180.0, height: 120.0, color: BLUE }を定義する

5. When ウィンドウを表示する, the example shall 1つ目のウィンドウに赤い四角が表示されることを確認する

6. When ウィンドウを表示する, the example shall 2つ目のウィンドウに青い四角が表示されることを確認する

7. When 動作を確認する, the example shall エラーなく実行されることを確認する

8. The example shall Rectangleコンポーネントの使用例をコメントで説明する

9. The example shall Phase 2-M3で追加した描画テストシステムのスケジュール登録を削除する（render_windowの登録を削除）

10. The example shall Surface検証コードを削除する（GraphicsCommandListに置き換わるため）

11. The Phase 2-M3 test code shall コード自体は参考実装として graphics.rs に残す

---

## Non-Functional Requirements

### Performance
- CommandListの生成はRectangle変更時のみ実行し、60fps（16.67ms/frame）を維持すること
- 描画処理は初回のみ実行し、リソースを効率的に使用すること

### Maintainability
- Widget関連コンポーネントはcrates/wintf/src/ecs/widget/に配置すること
- shapes配下に基本描画要素（Rectangle等）を配置すること
- COM APIラッパーはcrates/wintf/src/com/d2d/mod.rsに配置すること

### Extensibility
- 将来的にEllipse、Path等の他のShape追加が容易な構造とすること
- 将来的にEntity階層構造への移行が容易な設計とすること

---

## Traceability Matrix

| Requirement | Component/System | Priority |
|-------------|------------------|----------|
| Req 1 | Rectangle Component | Must |
| Req 2 | GraphicsCommandList Component | Must |
| Req 3 | graphics.rs Refactoring | Must |
| Req 4 | draw_rectangles System | Must |
| Req 5 | render_surface System | Must |
| Req 6 | render_window Cleanup | Must |
| Req 7 | System Separation | Must |
| Req 8 | COM API Wrappers | Must |
| Req 9 | Error Handling | Must |
| Req 10 | Integration Test | Should |

---

## Glossary

- **Widget**: ユーザーインターフェースの構成要素（Rectangle、Ellipse等）
- **Shape**: 基本描画要素（Rectangle、Ellipse、Path等）
- **CommandList**: Direct2DのID2D1CommandList。描画命令を記録するオブジェクト
- **Draw スケジュール**: ECSのスケジュール。描画命令の生成を行う
- **Render スケジュール**: ECSのスケジュール。Surfaceへの実際の描画を行う
- **Changed<T>**: Bevyのフィルター。コンポーネントTが変更されたエンティティのみを検出する
- **Added<T>**: Bevyのフィルター。コンポーネントTが追加されたエンティティのみを検出する

---

## References

- `.kiro/specs/phase2-m3-first-rendering/requirements.md` - 前提となるMilestone 3の要件
- [Direct2D Command Lists](https://learn.microsoft.com/en-us/windows/win32/direct2d/direct2d-command-lists)
- [WinUI 3 Shapes](https://learn.microsoft.com/en-us/windows/apps/design/controls/shapes)

---

_Requirements phase completed. Ready for design phase._
