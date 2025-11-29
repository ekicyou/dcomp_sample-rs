# Requirements: Phase 2 Milestone 3 - 初めての描画

**Feature ID**: `phase2-m3-first-rendering`  
**Version**: 1.0  
**Last Updated**: 2025-11-14

---

## Introduction

本仕様は、Phase 2 Milestone 3「初めての描画（●■▲）」の要件を定義します。DirectComposition + Direct2Dの描画パイプライン全体を動作させ、ウィンドウに透過背景で赤い円●、緑の四角■、青い三角▲を描画します。

これはPhase 2の最重要マイルストーンであり、**初めて視覚的な結果が確認できる**重要なステップです。

### 前提条件
- **Milestone 1完了**: `phase2-m1-graphics-core` - GraphicsCoreが正常に初期化されていること
- **Milestone 2完了**: `phase2-m2-window-graphics` - WindowGraphicsとVisualコンポーネントが作成されていること
- WindowHandleコンポーネントとWindowPosコンポーネントが利用可能であること

### スコープ外
- 子Visual要素の管理（Milestone 4で対応）
- デバイスロスト対応（将来の拡張）
- テキスト描画（Phase 4で対応）
- レイアウトシステム（将来の拡張）

---

## Requirements

### Requirement 1: Surfaceコンポーネントの作成と管理

**目的:** システム管理者として、各ウィンドウに描画可能なSurfaceリソースを提供することで、DirectCompositionでの描画を可能にしたい

#### Acceptance Criteria

1. When WindowGraphicsとVisualコンポーネントが存在するウィンドウエンティティが検出される, the Surface Creation System shall ウィンドウに対してSurfaceコンポーネントを作成する

2. The Surface component shall IDCompositionSurfaceのフィールドを保持する

3. When Surfaceを作成する, the Surface Creation System shall WindowPosコンポーネントからウィンドウのサイズ（width, height）を取得する

4. When ウィンドウサイズが取得できない, the Surface Creation System shall デフォルトサイズ（800x600）を使用する

5. When IDCompositionSurfaceを作成する, the Surface Creation System shall GraphicsCoreのdcompデバイスのcreate_surface()メソッドを使用する

6. When IDCompositionSurfaceを作成する, the Surface Creation System shall ピクセルフォーマットにDXGI_FORMAT_B8G8R8A8_UNORMを指定する

7. When IDCompositionSurfaceを作成する, the Surface Creation System shall アルファモードにDXGI_ALPHA_MODE_PREMULTIPLIEDを指定する

8. When IDCompositionSurfaceが作成される, the Surface Creation System shall Surfaceコンポーネントを作成してエンティティに追加する

9. When Surfaceが作成される, the Surface Creation System shall VisualコンポーネントのIDCompositionVisual3にSetContent()を呼び出してSurfaceを設定する

10. When Surfaceコンポーネントが作成される, the Surface Creation System shall 作成成功をログに出力する

11. If Surface作成中にエラーが発生する, then the Surface Creation System shall エラー内容をログに出力し、そのエンティティに対するSurface作成をスキップする

12. The Surface Creation System shall Query<(Entity, &WindowGraphics, &Visual, Option<&WindowPos>), Without<Surface>>を使用して未作成ウィンドウを検出する

### Requirement 2: 描画処理の実装

**目的:** 開発者として、ウィンドウに図形を描画することで、DirectComposition + Direct2Dの描画パイプラインの動作を確認したい

#### Acceptance Criteria

1. When Surfaceコンポーネントが存在するウィンドウが検出される, the Rendering System shall Surfaceが追加されたフレームのみ描画処理を実行する

2. When 描画を開始する, the Rendering System shall SurfaceのIDCompositionSurface.BeginDraw()を呼び出してID2D1DeviceContextを取得する

3. When BeginDraw()を呼び出す, the Rendering System shall updateRectパラメータにNoneを指定する

4. When ID2D1DeviceContextが取得される, the Rendering System shall device_context.BeginDraw()を呼び出す

5. When BeginDraw()が完了する, the Rendering System shall device_context.Clear()で透明色（rgba: 0.0, 0.0, 0.0, 0.0）をクリアする

6. When クリアが完了する, the Rendering System shall 赤い円を描画する（FillEllipse）

7. When 赤い円を描画する, the Rendering System shall 中心座標（100.0, 100.0）、半径（50.0, 50.0）を指定する

8. When 赤い円を描画する, the Rendering System shall 赤色のブラシ（rgba: 1.0, 0.0, 0.0, 1.0）を使用する

9. When 赤い円が描画される, the Rendering System shall 緑の四角を描画する（FillRectangle）

10. When 緑の四角を描画する, the Rendering System shall 矩形領域（left: 200.0, top: 50.0, right: 300.0, bottom: 150.0）を指定する

11. When 緑の四角を描画する, the Rendering System shall 緑色のブラシ（rgba: 0.0, 1.0, 0.0, 1.0）を使用する

12. When 緑の四角が描画される, the Rendering System shall 青い三角を描画する（FillGeometry）

13. When 青い三角を描画する, the Rendering System shall PathGeometryを使用して三角形を定義する

14. When PathGeometryを作成する, the Rendering System shall GraphicsCoreのd2d_factoryのCreatePathGeometry()メソッドを使用する

15. When 三角形の頂点を定義する, the Rendering System shall 3つの頂点（(350.0, 50.0), (425.0, 150.0), (275.0, 150.0)）を指定する

16. When 青い三角を描画する, the Rendering System shall 青色のブラシ（rgba: 0.0, 0.0, 1.0, 1.0）を使用する

17. When すべての図形が描画される, the Rendering System shall device_context.EndDraw()を呼び出す

18. When device_context.EndDraw()が完了する, the Rendering System shall SurfaceのIDCompositionSurface.EndDraw()を呼び出す

19. If EndDraw()がエラーを返す, then the Rendering System shall エラー内容をログに出力する

20. The Rendering System shall Query<(&Surface, Added<Surface>)>を使用してSurfaceが追加されたウィンドウのみを検出する

### Requirement 3: ブラシリソース管理

**目的:** システム管理者として、描画に必要なブラシリソースを効率的に管理することで、パフォーマンスを維持したい

#### Acceptance Criteria

1. When ブラシが必要になる, the Rendering System shall ID2D1DeviceContext::CreateSolidColorBrush()を使用してブラシを作成する

2. When 赤色ブラシを作成する, the Rendering System shall D2D1_COLOR_F { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }を指定する

3. When 緑色ブラシを作成する, the Rendering System shall D2D1_COLOR_F { r: 0.0, g: 1.0, b: 0.0, a: 1.0 }を指定する

4. When 青色ブラシを作成する, the Rendering System shall D2D1_COLOR_F { r: 0.0, g: 0.0, b: 1.0, a: 1.0 }を指定する

5. The Rendering System shall ブラシをフレームごとに作成し、描画完了後に自動解放する

6. The wintf library shall ブラシ作成の失敗時はエラーログを出力し、その図形の描画をスキップする

### Requirement 4: PathGeometry作成と三角形描画

**目的:** 開発者として、複雑な図形（三角形）を描画することで、PathGeometryの使用方法を確認したい

#### Acceptance Criteria

1. When PathGeometryを作成する, the Rendering System shall GraphicsCoreのd2d_factory.CreatePathGeometry()を呼び出す

2. When PathGeometryが作成される, the Rendering System shall path_geometry.Open()でGeometrySinkを取得する

3. When GeometrySinkが取得される, the Rendering System shall sink.BeginFigure()で図形の開始点を設定する

4. When 開始点を設定する, the Rendering System shall 第1頂点（450.0, 100.0）を指定する

5. When 開始点を設定する, the Rendering System shall D2D1_FIGURE_BEGIN_FILLEDを指定する

6. When 図形の開始が完了する, the Rendering System shall sink.AddLine()で第2頂点（550.0, 250.0）を追加する

7. When 第2頂点が追加される, the Rendering System shall sink.AddLine()で第3頂点（350.0, 250.0）を追加する

8. When すべての頂点が追加される, the Rendering System shall sink.EndFigure()で図形を閉じる

9. When 図形を閉じる, the Rendering System shall D2D1_FIGURE_END_CLOSEDを指定する

10. When 図形が閉じられる, the Rendering System shall sink.Close()でGeometrySinkを確定する

11. When PathGeometryが完成する, the Rendering System shall device_context.FillGeometry()でPathGeometryを描画する

12. If PathGeometry作成中にエラーが発生する, then the Rendering System shall エラーログを出力し、三角形の描画をスキップする

### Requirement 5: Commit処理の実装

**目的:** システム管理者として、DirectCompositionの変更を確定することで、描画内容を画面に反映したい

#### Acceptance Criteria

1. When 毎フレームの描画処理が完了する, the Commit System shall GraphicsCoreのdcompデバイスのCommit()メソッドを呼び出す

2. The Commit System shall すべての描画処理が完了した後に実行される

3. The Commit System shall 専用のCommitCompositionスケジュールで実行される

4. When Commit()が成功する, the Commit System shall 成功をログに出力する

5. If Commit()がエラーを返す, then the Commit System shall HRESULTコードを含むエラーメッセージをログに出力する

6. The Commit System shall GraphicsCoreが存在しない場合は警告ログを出力して処理をスキップする

### Requirement 6: システム実行順序とスケジュール配置

**目的:** システム管理者として、描画関連システムの実行順序が適切に制御されることで、依存関係エラーを防止したい

#### Acceptance Criteria

1. The wintf library shall Surface作成システムを`crates/wintf/src/ecs/graphics.rs`に実装する

2. The wintf library shall 描画システムを`crates/wintf/src/ecs/graphics.rs`に実装する

3. The Surface Creation System shall create_window_visualシステムの後に実行される

4. The Rendering System shall Surface Creation Systemの後に実行される

5. The Commit System shall Rendering Systemの後に実行される

6. The ECS schedule shall Surface作成システムをPostLayoutスケジュールに配置する

7. The ECS schedule shall 描画システムを専用のRenderスケジュールに配置する

8. The ECS schedule shall Commit処理を専用のCommitCompositionスケジュールに配置する

9. The wintf library shall スケジュール実行順序をドキュメントに明記する（PostLayout → Render → CommitComposition）

### Requirement 7: エラーハンドリングとログ出力

**目的:** 開発者として、描画処理の成否を明確に把握できることで、問題の診断とデバッグを効率化したい

#### Acceptance Criteria

1. When Surface作成が開始される, the Surface Creation System shall エンティティIDとサイズをログに出力する

2. When 描画処理が開始される, the Rendering System shall エンティティIDをログに出力する

3. If COM APIがエラーを返す, then the system shall HRESULTコードを含む詳細なエラーメッセージをログに出力する

4. When 描画処理が完了する, the system shall 成功メッセージをログに出力する

5. The system shall エラー発生時もパニックせず、そのウィンドウの描画のみをスキップする

6. When PathGeometry作成が失敗する, the system shall 三角形の描画をスキップして残りの処理を継続する

7. When ブラシ作成が失敗する, the system shall その図形の描画をスキップして残りの処理を継続する

### Requirement 8: コンポーネント統合

**目的:** 開発者として、Surfaceコンポーネントに統一的にアクセスできることで、後続の描画機能を効率的に実装できるようにしたい

#### Acceptance Criteria

1. The wintf library shall Surfaceコンポーネントを`crates/wintf/src/ecs/graphics.rs`に定義する

2. The wintf library shall Surfaceコンポーネントをpublicとして公開する

3. The Surface component shall IDCompositionSurfaceへのアクセスメソッドを提供する

4. The Surface component shall Send + Syncトレイトを実装する

5. When ウィンドウのグラフィックスリソースが完全に初期化される, the wintf library shall Query<(&WindowHandle, &WindowGraphics, &Visual, &Surface)>で全要素を取得可能にする

---

## Non-Functional Requirements

### Performance
- Surface作成は1フレーム以内に完了すること
- 描画処理は60fpsを維持できること（1フレーム < 16.67ms）
- Commit処理は1ms以内に完了すること

### Reliability
- COM APIエラー時もアプリケーション全体がクラッシュしないこと
- EndDraw()のエラー検出が確実に行われること
- リソースリークが発生しないこと（Drop実装で保証）

### Maintainability
- COM APIラッパーは`crates/wintf/src/com/dcomp.rs`と`crates/wintf/src/com/d2d/`に集約すること
- ECSコンポーネント（Surface）は`crates/wintf/src/ecs/graphics.rs`に配置すること
- ECSシステム（create_window_surface, render_window, commit_composition）は`crates/wintf/src/ecs/graphics.rs`に配置すること
- システム実装は明確な責務分離を維持すること
- モジュール構造はプロジェクトのレイヤードアーキテクチャに従うこと

### Usability
- ウィンドウに透過背景で図形が表示されること
- デスクトップが透けて見えること（透過動作の確認）
- 3つの図形（●■▲）が明確に視認できること

---

## Glossary

- **Surface**: DirectCompositionで描画可能なサーフェスを表すECSコンポーネント（IDCompositionSurface）
- **WindowPos**: ウィンドウの位置とサイズを保持するECSコンポーネント
- **ID2D1DeviceContext**: Direct2Dの描画コンテキスト（描画コマンドを発行）
- **ID2D1DeviceContext**: IDCompositionSurface.BeginDraw()から取得される描画コンテキスト
- **IDCompositionSurface**: DirectCompositionのサーフェス（合成ツリーのコンテンツ）
- **PathGeometry**: Direct2Dの複雑な図形定義（三角形など）
- **BeginDraw/EndDraw**: Direct2Dの描画開始/終了API
- **Commit**: DirectCompositionの変更を確定して画面に反映

---

## References

- `.kiro/steering/tech.md` - Technology Stack
- `.kiro/steering/structure.md` - Project Structure
- `.kiro/specs/phase2-m1-graphics-core/` - 前提となるMilestone 1
- `.kiro/specs/phase2-m2-window-graphics/` - 前提となるMilestone 2
- `.kiro/specs/brainstorming-next-features/MILESTONES.md` - マイルストーン全体像
- `crates/wintf/src/com/dcomp.rs` - DirectComposition APIラッパー
- `crates/wintf/src/com/d2d/` - Direct2D APIラッパー
- `crates/wintf/src/ecs/graphics.rs` - GraphicsCore, WindowGraphics, Visual定義

---

_要件定義フェーズ完了。設計フェーズへ進むには承認が必要です。_
