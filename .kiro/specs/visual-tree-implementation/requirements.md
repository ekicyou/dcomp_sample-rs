# Requirements: visual-tree-implementation

**Feature ID**: `visual-tree-implementation`  
**Created**: 2025-11-17  
**Status**: Requirements Generated (Simplified)

---

## Introduction

本要件定義は、DirectCompositionを使用したビジュアルツリー構造の実装を定義する。現在のwintfフレームワークは、基本的なグラフィックスリソース（WindowGraphics、Visual、Surface）を持つが、IDCompositionVisualの階層構造（親子関係）は未実装である。

本機能により、ECSのEntity親子関係（Parent/Children）をそのままIDCompositionVisualの階層にマッピングし、DirectCompositionの階層的合成機能を活用して、効率的な描画とトランスフォーム、部分更新を実現する。

### 現状分析

**既存実装**:
- ✅ GraphicsCore: DirectComposition Device統合済み
- ✅ WindowGraphics: Window単位のIDCompositionTarget保持
- ✅ Visual: IDCompositionVisual3のラッパーコンポーネント（単体のみ、階層なし）
- ✅ Surface: IDCompositionSurfaceの管理
- ✅ Rectangle、Label: GraphicsCommandListでフラット描画

**未実装**:
- ❌ IDCompositionVisualの親子関係（AddVisual）
- ❌ Entity階層に基づくビジュアルツリー構築
- ❌ 階層的なオフセット（親Visualからの相対位置）
- ❌ ルートVisualの管理とSetRoot

### 設計原則

1. **Entity階層をそのままVisual階層にマッピング**: ECSのParent/ChildrenがIDCompositionVisualの親子関係に対応
2. **描画Widgetのみがvisualを持つ**: Rectangle、Label等の描画コンポーネントを持つEntityのみVisual作成
3. **変更検知による自動更新**: Changed<Parent>、Changed<Visual>で階層とトランスフォームを自動更新
4. **型安全性**: unsafeコードはCOMラッパー層に隔離

---

## Requirements

### Requirement 1: IDCompositionVisualの親子関係構築

**Objective:** システム開発者として、Entity階層に基づいてIDCompositionVisualの親子関係を構築したい。これにより、DirectCompositionの階層的合成機能を活用できる。

#### Acceptance Criteria

1. When EntityにVisualコンポーネントが追加される時、wintfシステムはIDCompositionVisual3インスタンスを作成しなければならない
2. When EntityがParentコンポーネントを持ち、親EntityもVisualコンポーネントを持つ時、wintfシステムは親Visual.AddVisual()で子Visualを追加しなければならない
3. When 親EntityがVisualを持たない時、wintfシステムは祖先Entityを遡ってVisualを持つ最初のEntityを見つけ、そこに子Visualを追加しなければならない
4. The wintfシステムは、WindowエンティティのVisualをルートVisualとして識別しなければならない
5. When Visualコンポーネントが作成された時、wintfシステムはCOM参照カウントを適切に管理しなければならない

---

### Requirement 2: ビジュアルツリー構築システム

**Objective:** システム開発者として、Entity階層の変更時に自動的にIDCompositionVisualの親子関係を更新したい。これにより、手動でのツリー管理が不要になる。

#### Acceptance Criteria (R2)

1. The wintfシステムは、Drawスケジュール内でビジュアルツリー構築システム（build_visual_tree）を提供しなければならない
2. When 新しいVisualコンポーネントが追加された時（Added<Visual>）、wintfシステムは親Entityを検索してAddVisualを呼び出さなければならない
3. When ParentコンポーネントがChanged<Parent>である時、wintfシステムはVisualの親子関係を再構築しなければならない
4. When Visualコンポーネントが削除された時、wintfシステムは親VisualからRemoveVisualを呼び出さなければならない
5. The wintfシステムは、build_visual_treeをdraw_rectangles、draw_labelsの後かつrender_surfaceの前に実行しなければならない

---

### Requirement 3: 描画Widgetへの自動Visual割り当て

**Objective:** アプリケーション開発者として、描画コンポーネントを追加したら自動的にVisualが作成されるようにしたい。これにより、明示的なVisual管理が不要になる。

#### Acceptance Criteria (R3)

1. When EntityにRectangleまたはLabelコンポーネントが追加され、Visualを持たない時、wintfシステムはVisualコンポーネントを自動追加しなければならない
2. The wintfシステムは、ensure_visualシステムをDrawスケジュールの最初に実行しなければならない
3. When GraphicsCommandListがエンティティから削除された時、wintfシステムは対応するVisualコンポーネントも削除しなければならない
4. When Visualコンポーネントが削除された時、wintfシステムはIDCompositionVisualのCOM参照を解放しなければならない
5. When エンティティがdespawnされる時、wintfシステムはbevy_ecsのon_removeフックでVisualリソースをクリーンアップしなければならない

---

### Requirement 4: 階層的オフセット

**Objective:** アプリケーション開発者として、Widgetの位置（Rectangle.x/y、Label.x/y）を親Visualからの相対座標として扱いたい。これにより、親が移動すれば子も自動的に追従する。

#### Acceptance Criteria (R4)

1. The wintfシステムは、Visualコンポーネントにoffset_x、offset_yフィールドを提供しなければならない
2. When Rectangle.xまたはLabel.xが変更された時、wintfシステムは対応するVisual.offset_xを更新しなければならない
3. When Visual.offset_xまたはoffset_yが変更された時、wintfシステムはIDCompositionVisual::SetOffsetXとSetOffsetYを呼び出さなければならない
4. The wintfシステムは、apply_visual_transformsシステムでChanged<Rectangle>、Changed<Label>を検知してオフセットを同期しなければならない
5. The wintfシステムは、オフセット値をピクセル単位の浮動小数点数（f32）で管理しなければならない

---

### Requirement 5: ルートVisual管理

**Objective:** システム開発者として、Windowエンティティをビジュアルツリーのルートとして管理したい。これにより、ウィンドウ単位で独立したビジュアルツリーが構築される。

#### Acceptance Criteria (R5)

1. When WindowGraphicsコンポーネントが初期化される時、wintfシステムは対応するWindowエンティティにVisualコンポーネントを追加しなければならない
2. When WindowエンティティのVisualが作成された時、wintfシステムはIDCompositionTarget::SetRootを呼び出してルートVisualとして設定しなければならない
3. The wintfシステムは、init_window_visualシステムでWindowエンティティ（Parentを持たない）を検索してルートVisualを作成しなければならない
4. When 子EntityがWindowの直接の子である時、wintfシステムは子VisualをルートVisualに追加しなければならない
5. The wintfシステムは、複数ウィンドウが存在する時、各ウィンドウで独立したビジュアルツリーを管理しなければならない

---

### Requirement 6: VisualへのSurface設定

**Objective:** システム開発者として、描画されたSurfaceをVisualに設定したい。これにより、Rectangle、Label等の描画内容がビジュアルツリーに統合される。

#### Acceptance Criteria (R6)

1. When SurfaceコンポーネントとVisualコンポーネントが同一エンティティに存在する時、wintfシステムはIDCompositionVisual::SetContentでSurfaceを設定しなければならない
2. The wintfシステムは、render_surfaceシステムでGraphicsCommandListをSurfaceにコミットした後、SetContentを呼び出さなければならない
3. When Changed<Surface>が検知された時、wintfシステムはSetContentを再実行しなければならない
4. The wintfシステムは、Surfaceが無効化された時（Surface.invalidate()）、SetContentにNULLを渡してクリアしなければならない
5. The wintfシステムは、エンティティごとにSurfaceとVisualの1対1対応を維持しなければならない

---

### Requirement 7: 変更検知と効率的更新

**Objective:** システム開発者として、変更があったVisualのみを更新したい。これにより、不要なDirectComposition API呼び出しを削減してパフォーマンスが向上する。

#### Acceptance Criteria (R7)

1. The wintfシステムは、Changed<Parent>、Changed<Visual>フィルターで変更されたエンティティのみをクエリしなければならない
2. When 親子関係が変更された時、wintfシステムは該当するVisualのみAddVisual/RemoveVisualを呼び出さなければならない
3. When Visualのオフセットが変更された時、wintfシステムは該当するVisualのみSetOffsetX/SetOffsetYを呼び出さなければならない
4. The wintfシステムは、変更のないVisualに対してDirectComposition APIを呼び出してはならない
5. The wintfシステムは、CommitCompositionスケジュールでIDCompositionDevice::Commitを1回呼び出して全変更をコミットしなければならない

---

### Requirement 8: エラーハンドリング

**Objective:** システム開発者として、ビジュアルツリー構築時のエラーを適切に処理したい。これにより、部分的なエラーでもシステム全体が停止しない堅牢性が実現される。

#### Acceptance Criteria (R8)

1. When IDCompositionVisual3作成に失敗した時、wintfシステムはeprintln!でエラーログを出力し、該当エンティティをスキップしなければならない
2. When AddVisual呼び出しに失敗した時、wintfシステムはエラーログを出力し、Visualを孤立状態で保持しなければならない
3. If 親Entityが存在しないまたはVisualを持たない時、wintfシステムは警告ログを出力し、ルートVisualへの追加を試みなければならない
4. The wintfシステムは、windows::core::Resultのエラーをログ出力しなければならない
5. When エラーが発生した時、wintfシステムは該当エンティティのみスキップして他のエンティティの処理を継続しなければならない

---

### Requirement 9: サンプルアプリケーション

**Objective:** アプリケーション開発者として、ビジュアルツリーの使用例を参照したい。これにより、階層構造の利点と実装方法が理解できる。

#### Acceptance Criteria (R9)

1. The wintfシステムは、visual_tree_demo.rsサンプルアプリケーションを提供しなければならない
2. When サンプルが実行される時、wintfシステムは2階層以上のEntity親子関係を持つUIを表示しなければならない
3. The サンプルは、親RectangleEntityの座標変更時に子LabelEntityが追従する動作を示さなければならない
4. The サンプルは、実行時の動的なEntity追加・削除の例を含まなければならない
5. The サンプルは、cargo run --example visual_tree_demoで実行可能でなければならない

---

### Requirement 10: パフォーマンス要件

**Objective:** エンドユーザーとして、滑らかなUI表示を期待する。これにより、快適なユーザー体験が提供される。

#### Acceptance Criteria (R10)

1. The wintfシステムは、50個のVisual（RectangleまたはLabel）を持つビジュアルツリーで60fps以上を維持しなければならない
2. When 変更がないフレームでは、wintfシステムはCommit以外のDirectComposition APIを呼び出してはならない
3. The wintfシステムは、ビジュアルツリー構築（build_visual_tree）を1フレームあたり5ms以内で完了しなければならない
4. When 一部のEntityのみ変更された時、wintfシステムは該当するVisualのみを更新しなければならない
5. The wintfシステムは、COM参照カウント管理によりメモリリークを発生させてはならない

---

_Requirements generated on 2025-11-17_
