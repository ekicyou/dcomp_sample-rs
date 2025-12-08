# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | event-drag-system 要件定義書 |
| **Version** | 0.1 (Draft) |
| **Date** | 2025-12-08 |
| **Parent Spec** | wintf-P0-event-system |
| **Author** | AI-DLC System |

---

## Introduction

本仕様書は wintf フレームワークにおけるドラッグシステムの要件を定義する。親仕様「wintf-P0-event-system」の Requirement 5（ドラッグイベント）および Requirement 6（ウィンドウドラッグ移動）の実装を担当し、エンティティのドラッグ操作とウィンドウ全体の移動機能を提供する。

### 背景

デスクトップマスコットアプリケーションでは、キャラクターをドラッグして自由に配置する操作が必須機能である。本仕様は、マウスボタンが押下されてから移動、そして解放されるまでの一連のドラッグ操作を管理し、エンティティレベルのドラッグイベントとウィンドウレベルの位置更新を統合的に提供する。

### スコープ

**含まれるもの**:
- ドラッグイベント（DragStart, Drag, DragEnd）の発火と管理
- ドラッグ閾値に基づくドラッグ開始判定
- ドラッグ中の位置情報追跡（開始位置、現在位置、差分）
- ドラッグキャンセル機能（Escキー等）
- ウィンドウドラッグによるリアルタイム位置更新
- マルチモニター環境でのウィンドウ移動サポート
- ドラッグ可能性の制御機能（有効/無効切り替え）

**含まれないもの**:
- ドロップターゲット検出（Drag & Drop機能は将来拡張）
- ドラッグプレビュー表示（カーソル形状変更等）
- ドラッグによるエンティティの親子関係変更
- タッチデバイスのドラッグジェスチャー

### 親仕様からの要件マッピング

本仕様は以下の親要件に対応する：
- **Requirement 5**: ドラッグイベント
- **Requirement 6**: ウィンドウドラッグ移動

---

## Requirements

### Requirement 1: ドラッグ状態管理

**Objective:** 開発者として、エンティティのドラッグ状態を追跡したい。それによりドラッグ中の処理やキャンセル処理を適切に実装できる。

#### Acceptance Criteria

1. **The** Drag System **shall** エンティティごとのドラッグ状態（非ドラッグ、ドラッグ準備、ドラッグ中）を管理する
2. **When** 任意のマウスボタン（左/右/中）が押下された時, **the** Drag System **shall** ドラッグ準備状態に遷移する
3. **When** ドラッグ準備状態でマウスが移動し閾値を超えた時, **the** Drag System **shall** ドラッグ中状態に遷移する
4. **When** マウスボタンが解放された時, **the** Drag System **shall** 非ドラッグ状態に遷移する
5. **The** Drag System **shall** 現在ドラッグ中のエンティティとボタン種別を一意に識別できる
6. **The** Drag System **shall** ウィンドウエンティティのコンポーネントで、ボタンごとのドラッグ有効/無効を設定可能にする

---

### Requirement 2: ドラッグ開始イベント

**Objective:** 開発者として、ドラッグ操作の開始を検知したい。それによりドラッグ開始時の初期化処理を実装できる。

#### Acceptance Criteria

1. **When** マウスボタンが押下されてから閾値（デフォルト5物理ピクセル）を超えて移動した時, **the** Drag System **shall** DragStart イベントを発火する
2. **When** DragStartイベント発火時, **the** Drag System **shall** SetCapture APIを呼び出してマウスキャプチャを開始する
3. **The** Drag System **shall** ドラッグ開始位置（画面座標・ローカル座標）とマウスボタン種別をイベント情報に含める
4. **The** Drag System **shall** ドラッグ対象エンティティの識別子をイベント情報に含める
5. **The** Drag System **shall** ドラッグ閾値をエンティティごとに設定可能にする
6. **When** ドラッグ閾値が0に設定された場合, **the** Drag System **shall** マウスボタン押下直後にDragStartを発火する
7. **The** Drag System **shall** ドラッグ開始判定にユークリッド距離（√(dx²+dy²)）を使用する

---

### Requirement 3: ドラッグ中イベント

**Objective:** 開発者として、ドラッグ中のマウス移動を追跡したい。それによりリアルタイムなドラッグフィードバックを実装できる。

#### Acceptance Criteria

1. **While** ドラッグ中状態である間, **the** Drag System **shall** マウス移動時に Drag イベントを継続的に発火する
2. **The** Drag System **shall** 現在のマウス位置（画面座標・ローカル座標）をイベント情報に含める
3. **The** Drag System **shall** ドラッグ開始位置からの差分（dx, dy）をイベント情報に含める
4. **The** Drag System **shall** 前フレームからの移動量をイベント情報に含める
5. **The** Drag System **shall** ドラッグ経過時間をイベント情報に含める

---

### Requirement 4: ドラッグ終了イベント

**Objective:** 開発者として、ドラッグ操作の完了を検知したい。それによりドラッグ完了時の後処理を実装できる。

#### Acceptance Criteria

1. **When** ドラッグ中にマウスボタンが解放された時, **the** Drag System **shall** DragEnd イベントを発火する
2. **When** DragEndイベント発火時, **the** Drag System **shall** ReleaseCapture APIを呼び出してマウスキャプチャを解放する
3. **The** Drag System **shall** 最終マウス位置（画面座標・ローカル座標）をイベント情報に含める
4. **The** Drag System **shall** ドラッグ開始位置から最終位置までの総移動量をイベント情報に含める
5. **The** Drag System **shall** ドラッグ操作が正常終了か、キャンセルかを識別できる
6. **When** DragEnd イベント発火後, **the** Drag System **shall** ドラッグ状態をクリアする

---

### Requirement 5: ドラッグキャンセル

**Objective:** ユーザーとして、ドラッグ操作を途中でキャンセルしたい。それによりドラッグによる意図しない変更を回避できる。

#### Acceptance Criteria

1. **When** ドラッグ中にEscキーが押された時, **the** Drag System **shall** ドラッグをキャンセルする
2. **When** ドラッグがキャンセルされた時, **the** Drag System **shall** DragEnd イベントをキャンセルフラグ付きで発火する
3. **When** ドラッグキャンセル時, **the** Drag System **shall** ReleaseCapture APIを呼び出してマウスキャプチャを解放する
4. **The** Drag System **shall** プログラムからのドラッグキャンセル要求を受け付ける
5. **When** ドラッグ対象エンティティが削除された時, **the** Drag System **shall** 自動的にドラッグをキャンセルする
6. **When** ウィンドウがフォーカスを失った時, **the** Drag System **shall** ドラッグをキャンセルするオプションを提供する

---

### Requirement 6: ウィンドウドラッグ移動

**Objective:** ユーザーとして、キャラクターをドラッグしてウィンドウ全体を移動させたい。それによりデスクトップ上の好きな位置にキャラクターを配置できる。

#### Acceptance Criteria

1. **When** ウィンドウエンティティがドラッグイベントをキャプチャした時, **the** Drag System **shall** そのウィンドウのOffset（WindowPos）を更新する
2. **While** ドラッグ中, **the** Drag System **shall** マウスカーソルに追従してウィンドウOffsetをリアルタイム更新する
3. **The** Drag System **shall** ウィンドウ位置の更新にWin32 API（SetWindowPos等）を使用する
4. **The** Drag System **shall** ドラッグによるウィンドウ移動の有効/無効をウィンドウエンティティのコンポーネントで設定可能にする
5. **The** Drag System **shall** ウィンドウエンティティのみをドラッグ移動対象とし、子ウィジェットの個別移動は対象外とする
6. **When** ドラッグイベントがキャプチャされない場合, **the** Drag System **shall** ウィンドウ移動を実行しない

---

### Requirement 7: イベント配信とバブリング/トンネリング

**Objective:** 開発者として、ドラッグイベントの伝播を制御したい。それにより親要素による介入や子要素による伝播停止を実装できる。

#### Acceptance Criteria

1. **The** Drag System **shall** 親仕様のイベントシステムのPhase enum（Tunnel/Bubble）を使用する
2. **When** ドラッグイベントが発生した時, **the** Drag System **shall** Tunnelフェーズ（root→sender）とBubbleフェーズ（sender→root）の2フェーズで配信する
3. **When** Tunnelフェーズでハンドラがtrueを返した時, **the** Drag System **shall** イベント伝播を停止する（Bubbleフェーズは実行しない）
4. **When** Bubbleフェーズでハンドラがtrueを返した時, **the** Drag System **shall** それ以降の親への伝播を停止する
5. **The** Drag System **shall** ハンドラに sender（元のヒット対象）と entity（現在処理中のエンティティ）を引数として渡す
6. **When** 子ウィジェットがドラッグイベントを消費した時, **the** Drag System **shall** ウィンドウ移動を実行しない

---

### Requirement 8: マルチモニター対応

**Objective:** ユーザーとして、マルチモニター環境でキャラクターを任意のモニターに移動させたい。それにより複数のディスプレイを有効活用できる。

#### Acceptance Criteria

1. **When** ドラッグがモニター境界を越えた時, **the** Drag System **shall** 正常にウィンドウを移動する
2. **The** Drag System **shall** 仮想スクリーン座標系でのウィンドウ位置計算を行う
3. **When** ドラッグ終了時にウィンドウが画面外に配置された時, **the** Drag System **shall** 可視領域に補正するオプションを提供する
4. **The** Drag System **shall** 高DPI環境における座標変換を正確に行う
5. **When** モニター構成が変更された時, **the** Drag System **shall** ウィンドウ位置を再計算する

---

### Requirement 9: ドラッグ制約

**Objective:** 開発者として、ドラッグ範囲を制限したい。それにより特定領域内でのみドラッグを許可する機能を実装できる。

#### Acceptance Criteria

1. **The** Drag System **shall** 特定のエンティティのGlobalArrangementバウンディングボックスを制約領域として指定できる
2. **When** ドラッグが制約範囲外に到達した時, **the** Drag System **shall** 範囲境界でウィンドウ移動を停止する
3. **The** Drag System **shall** GlobalArrangementのスクリーン物理座標系バウンディングボックスを制約判定に使用する
4. **The** Drag System **shall** 軸ごとのドラッグ制約（水平のみ、垂直のみ）をサポートする
5. **The** Drag System **shall** ドラッグ制約の有効/無効を動的に切り替えられる
6. **When** ドラッグ制約が設定されている場合, **the** Drag System **shall** Dragイベント情報に制約適用後の位置を含める

---

### Requirement 10: ECS統合

**Objective:** 開発者として、ドラッグシステムをECSアーキテクチャに統合したい。それにより既存のwintfパターンと一貫性を保てる。

#### Acceptance Criteria

1. **The** Drag System **shall** ECSシステムとして実装される
2. **The** Drag System **shall** ドラッグ状態をECSコンポーネントとして管理する
3. **The** Drag System **shall** ドラッグイベントをECSリソースとして配信する
4. **The** Drag System **shall** 親仕様のイベントシステムと統合される
5. **When** エンティティが削除された時, **the** Drag System **shall** 関連するドラッグ状態をクリーンアップする

---

### Requirement 11: ドラッグ位置通知

**Objective:** 開発者として、ドラッグによるウィンドウ移動の最終位置を保存したい。それによりアプリケーション再起動時に前回の位置を復元できる。

#### Acceptance Criteria

1. **When** ドラッグが終了した時, **the** Drag System **shall** 最終ウィンドウ位置をイベントとして通知する
2. **The** Drag System **shall** 最終位置の画面座標と仮想スクリーン座標を提供する
3. **The** Drag System **shall** ドラッグによる総移動量を提供する
4. **The** Drag System **shall** ウィンドウが配置されたモニター情報を提供する
5. **When** ドラッグがキャンセルされた場合, **the** Drag System **shall** 位置変更通知を送信しない

---

### Requirement 12: taffy_flex_demo統合

**Objective:** 開発者として、taffy_flex_demoサンプルアプリケーションでドラッグ機能を実際に試したい。それにより実装の動作確認と使用例の提供ができる。

#### Acceptance Criteria

1. **The** Drag System **shall** taffy_flex_demoサンプルにウィンドウドラッグ機能を統合する
2. **When** taffy_flex_demoのウィジェットをドラッグした時, **the** Drag System **shall** ウィンドウ全体を移動する
3. **The** Drag System **shall** taffy_flex_demoでドラッグ可能なウィジェットを視覚的に識別可能にする
4. **The** Drag System **shall** taffy_flex_demoでドラッグ状態をログ出力する（デバッグ用）
5. **When** taffy_flex_demoを実行した時, **the** Drag System **shall** READMEまたはコメントでドラッグ機能の使い方を説明する

---

### Requirement 13: 処理場所と座標系

**Objective:** 開発者として、ドラッグ処理の実装場所を明確にしたい。それにより一貫した座標系とシンプルな実装を実現できる。

#### Acceptance Criteria

1. **The** Drag System **shall** ドラッグ状態管理と閾値判定をwndproc（ウィンドウプロシージャ）内で処理する
2. **The** Drag System **shall** ドラッグ閾値判定に物理ピクセル座標を使用する
3. **The** Drag System **shall** Win32メッセージ（WM_MOUSEMOVE等）から得られる物理ピクセル座標をそのまま使用する
4. **The** Drag System **shall** ドラッグ確定後、DragStartイベントをECSリソースとして配信する
5. **The** Drag System **shall** ウィンドウ移動（SetWindowPos）をwndprocから直接実行する

---

### Requirement 14: マウスキャプチャの使用

**Objective:** 開発者として、ドラッグ操作中のマウスイベント取得を確実にしたい。それにより将来の拡張（ウィンドウ非移動型ドラッグ）にも対応できる。

#### Acceptance Criteria

1. **The** Drag System **shall** ドラッグ開始時にSetCaptureでマウスキャプチャを取得する
2. **While** マウスキャプチャ中, **the** Drag System **shall** ウィンドウ外でもマウスイベントを受信できる
3. **The** Drag System **shall** ドラッグ終了時に必ずReleaseCaptureを呼び出す
4. **When** 予期しないキャプチャ解放（WM_CANCELMODE等）が発生した時, **the** Drag System **shall** ドラッグを自動的にキャンセルする
5. **The** Drag System **shall** マウスキャプチャの使用により、ウィンドウ移動型と非移動型ドラッグの統一的な実装基盤を提供する

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

- ドラッグイベント処理: 16ms以内で完了（60fps維持）
- ウィンドウ位置更新: 1フレーム遅延以内
- ドラッグ状態管理オーバーヘッド: 無視できるレベル（< 0.1ms）
- ドラッグ閾値判定: 物理ピクセル単位で計算（DPIスケーリング非依存）

### NFR-2: レスポンス

- マウス移動からウィンドウ移動まで: 遅延なくリアルタイム追従
- ドラッグ開始判定: マウス移動1フレーム以内
- ドラッグキャンセル: Escキー押下後即座に反映

### NFR-3: 信頼性

- ドラッグイベントの取りこぼしなし
- マウスボタン状態とドラッグ状態の整合性保証
- マルチモニター環境での正確な座標計算

---

## Glossary

| 用語 | 説明 |
|------|------|
| ドラッグ閾値 | ドラッグ開始と判定するための最小移動距離（デフォルト5物理ピクセル、ユークリッド距離） |
| ドラッグ準備状態 | マウスボタン押下後、閾値未達の状態 |
| ドラッグ中状態 | 閾値を超えてドラッグが開始された状態 |
| 仮想スクリーン座標 | マルチモニター環境における全ディスプレイを統合した座標系 |
| ローカル座標 | エンティティ内部の相対座標 |
| イベントキャプチャ | ドラッグイベントを処理する要素が決定されること |
| ウィンドウエンティティ | ウィンドウ全体を表すトップレベルエンティティ（Windowコンポーネント保持） |
| Tunnelフェーズ | イベントが親から子へ伝播するフェーズ（root→sender、WinUI3 PreviewXxx相当） |
| Bubbleフェーズ | イベントが子から親へ伝播するフェーズ（sender→root、WinUI3 Xxx相当） |
| stopPropagation | ハンドラがtrueを返すことでイベント伝播を停止する動作 |
| 物理ピクセル | Win32メッセージおよびヒットテストで使用される座標単位（DPIスケーリング前） |
| wndproc | Win32ウィンドウプロシージャ（メッセージハンドラ） |
| マウスキャプチャ | SetCapture/ReleaseCaptureによるマウスイベントの独占受信（ウィンドウ外でもイベント受信） |
| GlobalArrangement | エンティティのスクリーン物理座標系でのバウンディングボックス（レイアウト計算結果） |

---

## Appendix

### A. 関連ドキュメント

- 親仕様: `.kiro/specs/wintf-P0-event-system/requirements.md`
- イベントシステム設計: `doc/spec/08-event-system.md`
- ヒットテスト設計: `doc/spec/09-hit-test.md`

### B. ドラッグイベントデータ構造（参考）

```rust
// 参考実装イメージ（要件定義段階の例示）
struct DragEvent {
    entity: Entity,
    button: MouseButton,          // 左/右/中ボタン
    start_position: Point2D,      // 画面座標
    current_position: Point2D,    // 画面座標
    delta: Vector2D,              // 開始位置からの差分
    delta_from_last: Vector2D,    // 前フレームからの差分
    elapsed_time: Duration,       // 経過時間
    is_cancelled: bool,           // キャンセルフラグ
}

// ウィンドウエンティティのドラッグ設定コンポーネント（例示）
struct DragConfig {
    left_button_enabled: bool,    // 左ボタンでドラッグ可
    right_button_enabled: bool,   // 右ボタンでドラッグ可
    middle_button_enabled: bool,  // 中ボタンでドラッグ可
    drag_threshold: f32,          // ドラッグ閾値（ピクセル）
}

// ドラッグ制約の設定（例示）
struct DragConstraint {
    constraint_entity: Option<Entity>,  // 制約領域を定義するエンティティ
    allow_horizontal: bool,             // 水平方向ドラッグ許可
    allow_vertical: bool,               // 垂直方向ドラッグ許可
}

// 制約適用の例（参考実装イメージ）
fn apply_drag_constraint(
    world: &World,
    window_pos: (i32, i32),
    constraint: &DragConstraint
) -> (i32, i32) {
    if let Some(entity) = constraint.constraint_entity {
        if let Some(global_arr) = world.get::<GlobalArrangement>(entity) {
            let bounds = global_arr.bounds(); // スクリーン物理座標系の矩形
            let (mut x, mut y) = window_pos;
            
            if constraint.allow_horizontal {
                x = x.clamp(bounds.min.x as i32, bounds.max.x as i32);
            }
            if constraint.allow_vertical {
                y = y.clamp(bounds.min.y as i32, bounds.max.y as i32);
            }
            
            return (x, y);
        }
    }
    window_pos
}

// イベント伝播制御の例（参考実装イメージ）
fn on_drag_handler(
    world: &mut World,
    sender: Entity,      // 元のヒット対象
    entity: Entity,      // 現在処理中のエンティティ
    ev: &Phase<DragEvent>
) -> bool {
    match ev {
        Phase::Tunnel(drag_event) => {
            // 親が子のドラッグを事前に介入
            if should_prevent_child_drag(world, entity) {
                return true; // stopPropagation
            }
            false
        }
        Phase::Bubble(drag_event) => {
            // 通常のドラッグ処理
            handle_drag(world, entity, drag_event);
            true // イベント消費、ウィンドウ移動させない
        }
    }
}
```

### C. Win32 API連携

- `SetCapture`: マウスキャプチャ開始（ドラッグ開始時）
- `ReleaseCapture`: マウスキャプチャ解放（ドラッグ終了/キャンセル時）
- `SetWindowPos`: ウィンドウ位置更新
- `GetCursorPos`: カーソル位置取得
- `MonitorFromWindow`: ウィンドウが配置されたモニター取得
- `GetSystemMetrics(SM_XVIRTUALSCREEN/SM_YVIRTUALSCREEN)`: 仮想スクリーン座標取得
