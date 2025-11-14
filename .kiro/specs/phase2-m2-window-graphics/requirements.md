# Requirements: Phase 2 Milestone 2 - WindowGraphics + Visual作成

**Feature ID**: `phase2-m2-window-graphics`  
**Version**: 1.0  
**Last Updated**: 2025-11-14

---

## Introduction

本仕様は、Phase 2 Milestone 2「WindowGraphics + Visual作成」の要件を定義します。各ウィンドウに独立したDirectCompositionグラフィックスリソース（WindowGraphics）を作成し、ウィンドウごとのVisualツリーのルートを確立します。

このマイルストーンは、Phase 2「はじめての描画」の2番目のステップであり、Milestone 1で初期化したGraphicsCoreを基盤として、ウィンドウ単位のグラフィックス環境を構築します。

### 前提条件
- **Milestone 1完了**: `phase2-m1-graphics-core` - GraphicsCoreが正常に初期化されていること
- **Phase 1完了**: ウィンドウシステムが実装され、WindowHandleコンポーネントが利用可能であること

### スコープ外
- 描画処理の実装（Milestone 3で対応）
- 子Visual要素の管理（Milestone 4で対応）
- Surfaceの作成と描画コンテンツ（Milestone 3で対応）

---

## Requirements

### Requirement 1: ウィンドウグラフィックスリソース管理

**目的:** システム管理者として、各ウィンドウに独立したグラフィックスリソースを提供することで、ウィンドウごとの描画環境を確立できるようにしたい

#### Acceptance Criteria

1. When WindowHandleコンポーネントが存在するウィンドウエンティティが検出される, the Graphics Initialization System shall ウィンドウに対してWindowGraphicsコンポーネントを作成する

2. The WindowGraphics component shall IDCompositionTargetとID2D1DeviceContextを保持する

3. When WindowGraphicsを作成する, the Graphics Initialization System shall GraphicsCoreリソースのdesktopデバイスを使用してIDCompositionTargetを生成する

4. When IDCompositionTargetを作成する, the Graphics Initialization System shall create_target_for_hwnd APIを使用し、WindowHandleのhwndを指定する

5. When IDCompositionTargetを作成する, the Graphics Initialization System shall topmost引数にfalseを指定する

6. When WindowGraphicsコンポーネントが作成される, the Graphics Initialization System shall 作成成功をログに出力する

7. If WindowGraphics作成中にエラーが発生する, then the Graphics Initialization System shall エラー内容をログに出力し、そのエンティティに対するWindowGraphics作成をスキップする

8. The Graphics Initialization System shall Query<(&WindowHandle, Without<WindowGraphics>)>を使用して未作成ウィンドウを検出する

### Requirement 2: ルートVisual作成と設定

**目的:** システム管理者として、各ウィンドウにDirectCompositionのVisualツリーのルートを設定することで、将来の描画コンテンツの親要素として機能させたい

#### Acceptance Criteria

1. When WindowGraphicsコンポーネントが存在するウィンドウが検出される, the Visual Creation System shall ウィンドウに対してVisualコンポーネントを作成する

2. The Visual component shall IDCompositionVisual3インスタンスを保持する

3. When Visualを作成する, the Visual Creation System shall GraphicsCoreのdcompデバイスのcreate_visual()メソッドを呼び出す

4. When Visualが作成される, the Visual Creation System shall WindowGraphicsのIDCompositionTargetに対してset_root()を呼び出す

5. When set_root()を呼び出す, the Visual Creation System shall 作成したIDCompositionVisual3を引数として渡す

6. When Visualコンポーネントが作成される, the Visual Creation System shall 作成成功をログに出力する

7. If Visual作成中にエラーが発生する, then the Visual Creation System shall エラー内容をログに出力し、そのエンティティに対するVisual作成をスキップする

8. The Visual Creation System shall Query<(&WindowHandle, &WindowGraphics, Without<Visual>)>を使用して未作成ウィンドウを検出する

### Requirement 3: コンポーネント統合とクエリ対応

**目的:** 開発者として、ウィンドウのグラフィックスリソースとVisualに統一的にアクセスできることで、後続の描画処理を効率的に実装できるようにしたい

#### Acceptance Criteria

1. The wintf library shall WindowGraphicsコンポーネントを`crates/wintf/src/ecs/graphics.rs`に定義する

2. The wintf library shall Visualコンポーネントを`crates/wintf/src/ecs/graphics.rs`に定義する

3. The wintf library shall WindowGraphicsコンポーネントをpublicとして公開する

4. The wintf library shall Visualコンポーネントをpublicとして公開する

5. When ウィンドウのグラフィックスリソースが完全に初期化される, the wintf library shall Query<(&WindowHandle, &WindowGraphics, &Visual)>で全要素を取得可能にする

6. The WindowGraphics component shall ID2D1DeviceContextへのアクセスメソッドを提供する

7. The Visual component shall IDCompositionVisual3への参照を取得するメソッドを提供する

8. The WindowGraphics component shall IDCompositionTargetへの参照を取得するメソッドを提供する

### Requirement 4: リソースライフサイクル管理

**目的:** システム管理者として、ウィンドウの破棄時にグラフィックスリソースも適切に解放されることで、メモリリークを防止したい

#### Acceptance Criteria

1. When WindowHandleコンポーネントが削除される, the ECS framework shall 同一エンティティのWindowGraphicsとVisualコンポーネントも自動的に削除する

2. The WindowGraphics component shall DropトレイトでCOMオブジェクトの解放を自動実行する

3. The Visual component shall DropトレイトでCOMオブジェクトの解放を自動実行する

4. When ウィンドウが破棄される, the wintf library shall リソース解放をログに出力する

5. The WindowGraphics and Visual components shall Send + Syncトレイトを実装する

### Requirement 5: システム実行順序と実装場所

**目的:** システム管理者として、グラフィックスリソースの初期化順序が適切に制御されることで、依存関係エラーを防止したい

#### Acceptance Criteria

1. The wintf library shall WindowGraphics作成システムを`crates/wintf/src/ecs/graphics.rs`に実装する

2. The wintf library shall Visual作成システムを`crates/wintf/src/ecs/graphics.rs`に実装する

3. The Graphics Initialization System shall ensure_graphics_coreシステムの後に実行される

4. The Visual Creation System shall Graphics Initialization Systemの後に実行される

5. When GraphicsCoreが存在しない, the Graphics Initialization System shall 処理をスキップし警告をログに出力する

6. The ECS schedule shall グラフィックス初期化システムをUpdateステージに配置する

7. The wintf library shall システム実行順序をドキュメントに明記する

### Requirement 6: エラーハンドリングとログ出力

**目的:** 開発者として、グラフィックス初期化の成否を明確に把握できることで、問題の診断とデバッグを効率化したい

#### Acceptance Criteria

1. When WindowGraphics作成が開始される, the Graphics Initialization System shall エンティティIDとhwndをログに出力する

2. When Visual作成が開始される, the Visual Creation System shall エンティティIDをログに出力する

3. If COM APIがエラーを返す, then the system shall HRESULTコードを含む詳細なエラーメッセージをログに出力する

4. When グラフィックスリソース作成が完了する, the system shall 成功メッセージをeprintln!マクロで出力する

5. The system shall エラー発生時もパニックせず、そのウィンドウのリソース作成のみをスキップする

---

## Non-Functional Requirements

### Performance
- WindowGraphicsとVisual作成は1フレーム以内に完了すること
- 複数ウィンドウの並行初期化をサポートすること

### Reliability
- COM APIエラー時もアプリケーション全体がクラッシュしないこと
- リソースリークが発生しないこと（Drop実装で保証）

### Maintainability
- COM APIラッパーは`crates/wintf/src/com/dcomp.rs`に集約すること
- ECSコンポーネント（WindowGraphics, Visual）は`crates/wintf/src/ecs/graphics.rs`に配置すること
- ECSシステム（create_window_graphics, create_window_visual）は`crates/wintf/src/ecs/graphics.rs`に配置すること
- システム実装は明確な責務分離を維持すること
- モジュール構造はプロジェクトのレイヤードアーキテクチャに従うこと

---

## Glossary

- **WindowGraphics**: ウィンドウごとのDirectCompositionターゲットとD2Dデバイスコンテキストを保持するECSコンポーネント
- **Visual**: DirectCompositionのVisualツリーのルートノードを表すECSコンポーネント
- **GraphicsCore**: アプリケーション全体で共有されるグラフィックスデバイスとファクトリを保持するリソース
- **IDCompositionTarget**: DirectCompositionの描画ターゲット（hwndに紐付く）
- **IDCompositionVisual3**: DirectCompositionのビジュアル要素（ツリー構造の1ノード）

---

## References

- `.kiro/steering/tech.md` - Technology Stack
- `.kiro/steering/structure.md` - Project Structure
- `.kiro/specs/phase2-m1-graphics-core/` - 前提となるMilestone 1
- `crates/wintf/src/com/dcomp.rs` - DirectComposition APIラッパー
- `crates/wintf/src/ecs/graphics.rs` - GraphicsCoreリソース定義

---

_要件定義フェーズ完了。設計フェーズへ進むには承認が必要です。_
