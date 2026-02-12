# Gap Analysis: dpi-coordinate-transform-survey

> 分析日: 2026-02-11

---

## 1. 分析サマリー

- **Tier 0 の最大ブロッカー**: event-drag-system の「1.25倍速バグ」が Phase 4-7 を完全にブロック。静的コード解析では、ドラッグ → SetWindowPos の数値上の値は正しい（ログで確認済み）にもかかわらず、実際のウィンドウ移動が 1.25 倍速くなる。**Win32 API レベルのランタイムデバッグが必須**
- **座標系混在の構造的設計課題**: `BoxStyle.inset`（物理ピクセル）と `BoxStyle.size`（DIP）の混在が根本原因。この混在は意図的だが、ドラッグ変換チェーンを複雑にしバグの温床
- **Matrix3x2 乗算順序の潜在バグ**: `From<Arrangement> for Matrix3x2` で `translation * scale` は `M31 = offset * scale` を生むが、bounds は手動計算で正しい。現時点では影響は限定的だが、transform 値を直接使う将来の機能で問題化する
- **DPI 二重管理**: `WinState` trait（レガシー）と ECS `DPI` コンポーネントが共存。整理優先度は低いが技術的負債
- **既存基盤は健全**: `dpi-propagation` 仕様で実装された DPI → Arrangement.scale 伝播は正しく動作しており、座標系モデルの基盤自体は修正可能

---

## 2. Requirement-to-Asset Map

### Requirement 1: 座標系インベントリ（As-Is 調査）

| 技術的ニーズ                 | 既存アセット                                        | ギャップ                                           |
| ---------------------------- | --------------------------------------------------- | -------------------------------------------------- |
| 全座標値コンポーネントの列挙 | 各ファイルに分散                                    | **Missing**: 統一された一覧表なし                  |
| BoxStyle 混在の判定          | handlers.rs L227-261 で明示コメント                 | **Constraint**: 意図的設計。変更は大規模リファクタ |
| WinState vs DPI 二重管理     | win_state.rs / window.rs                            | **Known**: 非 ECS 版デモでのみ使用                 |
| Win32 API 入出力座標系       | process_singleton.rs L64 で Per-Monitor v2 設定済み | **Missing**: API ごとの挙動ドキュメント            |

**調査で判明した主要な座標系コンポーネント一覧**:

| コンポーネント/フィールド           | 座標系                                         | ファイル         |
| ----------------------------------- | ---------------------------------------------- | ---------------- |
| `BoxStyle.inset`                    | 物理ピクセル（スクリーン座標）                 | layout/mod.rs    |
| `BoxStyle.size`                     | DIP（論理ピクセル）                            | layout/mod.rs    |
| `WindowPos.position`                | 物理ピクセル（クライアント領域スクリーン座標） | window.rs        |
| `WindowPos.size`                    | 物理ピクセル                                   | window.rs        |
| `Arrangement.offset`                | Taffy単位（Window=物理px, Widget=DIP）         | arrangement.rs   |
| `Arrangement.scale`                 | DPIスケール比率                                | arrangement.rs   |
| `Arrangement.size`                  | Taffy単位（DIP）                               | arrangement.rs   |
| `GlobalArrangement.bounds`          | 物理ピクセル（スクリーン座標）                 | arrangement.rs   |
| `GlobalArrangement.transform`       | 累積行列（M11=scale, M31=offset×scale ⚠）      | arrangement.rs   |
| `DPI.dpi_x/dpi_y`                   | DPI値（96=100%）                               | window.rs        |
| `Monitor.dpi`                       | DPI値                                          | monitor.rs       |
| `PhysicalPoint`                     | 物理ピクセル（スクリーン座標）                 | hit_test.rs      |
| `DragEvent.position/start_position` | 物理ピクセル（スクリーン座標）                 | drag/            |
| `DraggingState.initial_inset`       | 物理ピクセル（BoxStyle.insetの初期値）         | drag/dispatch.rs |

### Requirement 2: DPI値フロー追跡

| 技術的ニーズ             | 既存アセット                       | ギャップ                                                             |
| ------------------------ | ---------------------------------- | -------------------------------------------------------------------- |
| DPI取得→消費の全経路追跡 | window.rs, handlers.rs, systems.rs | **Completed**: 本分析で完全追跡済み                                  |
| DpiChangeContext 検証    | window.rs L25-78                   | **Low Risk**: スレッドローカル設計は堅牢                             |
| 二重スケーリング検証     | update_arrangements_system         | **Verified**: Window の Mul パスでは parent_scale=1.0 のため問題なし |
| Monitor DPI 未反映       | monitor.rs                         | **Known Gap**: 情報保持のみ、Arrangement に未反映                    |

**DPI値フロー（追跡完了）**:

```
GetDpiForWindow(hwnd) → DPI component (u16,u16)
                            ↓
                    update_arrangements_system
                    [DPI.scale_x() → Arrangement.scale.x]
                            ↓
                    propagate_global_arrangements
                    [parent.transform * child_matrix → GlobalArrangement]
                            ↓
        ┌───────────────────┼──────────────────────┐
        ↓                   ↓                      ↓
visual_property_sync   sync_window_pos        render_surface
[offset*scale→Visual]  [bounds→WindowPos]     [scale→DC transform]
```

### Requirement 3: ドラッグ座標変換チェーン評価（As-Is / To-Be 対比）

| 技術的ニーズ                 | 既存アセット              | ギャップ                                              |
| ---------------------------- | ------------------------- | ----------------------------------------------------- |
| 変換チェーン全ステップ追跡   | drag/, layout/, graphics/ | **Completed**: 本分析で全10ステップ追跡               |
| 二重スケーリング経路検証     | —                         | **Verified**: 静的解析では二重スケーリング未検出      |
| 初回DPI固定化変数            | —                         | **Unknown**: 静的解析では検出不可。ランタイム確認必須 |
| sync_window_arrangement 評価 | world.rs L360-363         | **Known**: 無効化済み。有効化は座標系統一後           |

**完全なドラッグ変換チェーン（数値トレース）**:

条件: DPI=120 (scale 1.25), LayoutRoot原点=(0,0), Window初期位置=(300,200)物理px

```
Step 1: WM_MOUSEMOVE lParam → client(x,y) → screen(x,y) [物理px]
Step 2: delta = current_screen - prev_screen [物理px]
Step 3: DragEvent { position: screen_pos, start_position: ... } [物理px]
Step 4: initial_inset = BoxStyle.inset = (300,200) [物理px]
Step 5: new_left = 300 + delta_x(10) = 310 → BoxStyle.inset.left = Px(310) [物理px]
Step 6: Taffy: layout.location.x = 310 [Taffyは単位非認識、値をそのまま使用]
Step 7: Arrangement { offset:(310,...), scale:(1.25,...), size:(640,480) }
Step 8: propagate: parent_scale=1.0, bounds.left = 0+310*1.0 = 310 [物理px] ✓
Step 9: WindowPos.position.x = 310 [物理px] ✓
Step10: AdjustWindowRectExForDpi(DPI=current) → SetWindowPos ← ✓ ???
```

**静的解析の結論**: 数値上は正しい。「ログでは1px、実際には1.25px」の乖離はコード上に存在しない。

> **※本仕様のスコープ注記**: 上記変換チェーンの分析は As-Is の現状記録として保持する。本仕様の目的はバグの原因特定ではなく、「あるべき姿」の座標系設計を固めること。現行チェーンと To-Be アーキテクチャのコンフリクト箇所を指摘することが report.md での役割。

#### 📐 セッション内調査で判明した追加知見

**ドラッグハンドラの座標系混在問題**:

`taffy_flex_demo.rs` の `on_container_drag` 関数（L864-940）において、以下の座標系混在が確認された：

```
DragEvent.position (PhysicalPoint, 物理px) - DragEvent.start_position (PhysicalPoint, 物理px)
    = delta (物理px)
         ↓
initial_inset (DIP, BoxStyle.insetの初期値) + delta (物理px)
    = new_inset ← ❗ DIP + 物理pxの混在!
         ↓
BoxStyle.inset = Px(new_inset)  [単位非認識で保存]
         ↓
Taffy layout → Arrangement.offset = new_inset [そのまま使用]
         ↓
GlobalArrangement: bounds.left = parent_origin + offset * parent_scale(1.0)
    = new_inset [数値上は正しいが、DIP/物理の境界が曖昧]
```

**この知見の意味**:
- `BoxStyle.inset` が「統一された座標系」を持たないため、アプリケーション側のハンドラが座標系を意識する責務を負っている
- To-Be アーキテクチャで `BoxStyle.inset` を DIP 統一すれば、ドラッグハンドラ側では `delta / dpi_scale` の変換が必要になる
- または、フレームワーク側が入力座標（物理px）→ 内部座標（DIP）の変換を透過的に行う設計が望ましい
- **Req 3 AC 1 と Req 4 AC 2-3 の両方に直結する知見**

`DraggingState.initial_inset` の取得元: `drag/dispatch.rs` L98-107 で `BoxStyle.inset` から直接読み取り。この値は DIP 単位だが、ドラッグデルタは物理px。この不整合が、To-Be では解消されるべきコンフリクトの一つ。

#### 🔴 有力な新仮説: `WM_WINDOWPOSCHANGED` エコーバック問題

ドラッグフレームでの実行順序を精査した結果、以下のシナリオが判明:

```
Frame N:
 ① ドラッグハンドラ: BoxStyle.inset = 310
 ② Layout → PostLayout: WindowPos = 310 → SetWindowPos(310+border)
 ③ WM_WINDOWPOSCHANGED エコーバック:
    - WindowPosChanged=true（抑制用フラグ）
    - WindowPos = window_to_client(returned_pos) → 可能性: DPI依存で微差
    - BoxStyle.inset = client_pos（= 310 であるべき）
    - flush_window_pos_commands()（空のはず）
    - WindowPosChanged=false
```

ここで重要なのは、`WM_WINDOWPOSCHANGED` でのウィンドウ座標→クライアント座標逆変換 (`window_to_client_coords`) が `AdjustWindowRectExForDpi` に依存しており、このAPI の DPI 引数は `GetDpiForWindow(hwnd)` から取得される。**もし `GetDpiForWindow` が一時的に古い DPI を返す場合、逆変換結果にズレが生じ、次フレームの BoxStyle.inset が汚染される**。

ただし、この仮説だけでは「常に1.25倍」を説明できない。

#### 🔴 最有力仮説: LayoutRoot の仮想デスクトップ座標

`initialize_layout_root` で LayoutRoot の `BoxStyle.inset` に仮想デスクトップの左上座標 `(vx, vy)` が設定される（物理ピクセル）。LayoutRoot の `Arrangement.scale = (1.0, 1.0)` であるため、LayoutRoot の bounds は正しい。

しかし、**LayoutRoot を基準とした Window の offset が物理ピクセルの場合、Window の `Arrangement.scale = DPI` が offse にかかる `child_matrix` の `M31 = offset * DPI` を生み**、`result_transform` に伝播される。

`bounds` は手動計算で正しいが、**もし `result_transform.M31` が `bounds.left` の代わりに使われる箇所がある場合、1.25倍の位置ズレが発生する**。

→ **Research Needed**: `GlobalArrangement.transform.M31/M32` を直接参照している箇所の網羅的調査

### Requirement 4: あるべき座標変換アーキテクチャ（To-Be）

| 技術的ニーズ                     | 既存アセット                          | ギャップ                                       |
| -------------------------------- | ------------------------------------- | ---------------------------------------------- |
| WPF/WinUI3 DIP モデル調査        | —                                     | **Research Needed**: 設計フェーズで実施        |
| DIP 統一方針の評価               | Arrangement.size は DIP 統一済み      | **Gap**: BoxStyle.inset が物理px               |
| BoxStyle 座標系統一              | —                                     | **Design Needed**: inset を DIP に変換する設計 |
| Arrangement/WindowPos 関係再定義 | sync_window_arrangement... (無効化中) | **Design Needed**: 座標系統一後に有効化        |
| DPI 変更時再計算フロー           | DpiChangeContext                      | **Research Needed**: 最適な処理順序            |

### Requirement 5: ギャップ分析と優先度

| 技術的ニーズ                           | 既存アセット                  | ギャップ                                   |
| -------------------------------------- | ----------------------------- | ------------------------------------------ |
| 優先度マトリクス                       | —                             | **Missing**: 本分析で一次版を作成（下記）  |
| Quick Fix vs Architectural Fix         | —                             | **Design Needed**                          |
| dpi-propagation vs P1-dpi-scaling 差分 | 完了済み spec / backlog spec  | **Analyzable**: 下記に記載                 |
| WinState/DPI 統合方針                  | Req 1 AC 3 に統合済み         | **Low Priority**: 非 ECS デモ廃止時に実施 |
| ~~transform/ 廃止判定~~              | 本番使用ゼロ確認済み         | ✅ **解決済み**: 安全に削除可。要件から除外 |

### Requirement 6: 成果物レポート

| 技術的ニーズ   | 既存アセット | ギャップ                                 |
| -------------- | ------------ | ---------------------------------------- |
| report.md 作成 | —            | **Implementation**: タスクフェーズで作成 |

---

## 3. 実装アプローチ評価

### Option A: Quick Fix（最小修正でドラッグバグ解消）

**対象**: Requirement 3（1.25倍速バグ）のみ

1. ランタイムデバッグで `SetWindowPos` の実引数を直接ログ出力し、Win32 API に渡される値を特定
2. `GlobalArrangement.transform.M31/M32` を直接参照している箇所を修正（もしあれば）
3. ドラッグパスに限定した座標変換の修正

**Trade-offs**:
- ✅ 最速でドラッグバグを解消、event-drag-system Phase 4-7 のブロック解除
- ✅ 他の機能への影響最小
- ❌ 座標系混在の根本問題は温存
- ❌ 将来的に同種のバグが再発するリスク

**Effort**: S (1-3日)  
**Risk**: Medium（ランタイムデバッグの結果次第）

### Option B: Architectural Fix（BoxStyle 座標系統一）

**対象**: Requirement 1, 3, 4

1. `BoxStyle.inset` を DIP に統一（Window含む全エンティティ）
2. 物理ピクセル↔DIP 変換を明示的なレイヤーに分離
3. `sync_window_arrangement_from_window_pos` を有効化
4. Matrix3x2 乗算順序を `scale * translation` に修正

**Trade-offs**:
- ✅ 座標系の根本問題を解消
- ✅ 今後の DPI 関連バグを予防
- ✅ `wintf-P1-dpi-scaling` の基盤を固める
- ❌ 大規模リファクタ（影響箇所多数）
- ❌ テスト期待値の修正も必要
- ❌ ドラッグバグ単体の解消に時間がかかる

**Effort**: L (1-2週間)  
**Risk**: High（広範な変更、リグレッションのリスク）

### Option C: Hybrid（Quick Fix → 段階的統一）

**推奨アプローチ**

Phase 1: ランタイムデバッグで1.25倍速バグの正確な原因を特定・修正（S: 1-3日）
Phase 2: 調査レポートを report.md に集約（S: 1-2日）
Phase 3: BoxStyle 座標系統一の設計を `wintf-P1-dpi-scaling` に統合（Design: M）
Phase 4: 段階的に実装（`wintf-P1-dpi-scaling` のタスクとして）

**Trade-offs**:
- ✅ ドラッグバグを迅速に解消しつつ根本改善にも着手
- ✅ リスク分散：段階的に変更を導入
- ✅ 調査成果を P1 仕様の設計根拠として活用
- ❌ 全体完了までの期間は Option B より長い

**Effort**: S+M (1週間+α)  
**Risk**: Low（段階的アプローチでリスク分散）

---

## 4. ギャップ優先度マトリクス（一次版）

| #   | ギャップ項目                             | 影響度     | 修正コスト | ブロック仕様           | 優先度           |
| --- | ---------------------------------------- | ---------- | ---------- | ---------------------- | ---------------- |
| G1  | 1.25倍速バグ根本原因（ランタイム特定要） | **High**   | **Low**    | event-drag-system P4-7 | 🔴 **最優先**     |
| G2  | BoxStyle.inset/size 座標系混在           | **High**   | **High**   | wintf-P1-dpi-scaling   | 🟡 高（P1で対応） |
| G3  | Matrix3x2 乗算順序（translation*scale）  | **Medium** | **Low**    | 潜在的                 | 🟡 中             |
| G4  | transform.M31 参照箇所の確認             | **High**   | **Low**    | G1の原因の可能性       | 🔴 **最優先**     |
| G5  | sync_window_arrangement 無効化           | **Medium** | **Medium** | 座標系統一が前提       | 🟢 低（G2後）     |
| G6  | Monitor.dpi の Arrangement 未反映        | **Low**    | **Low**    | —                      | 🟢 低             |
| G7  | WinState/DPI 二重管理（Req 1 AC 3 に統合）    | **Low**    | **Low**    | —                      | 🟢 低             |
| G8  | ~~transform/ 非推奨モジュール残存~~       | **Low**    | **Low**    | —                      | ✅ 解決済（本番使用ゼロ確認、安全に削除可） |

---

## 5. dpi-propagation vs wintf-P1-dpi-scaling 差分

| 項目                          | dpi-propagation（完了）           | wintf-P1-dpi-scaling（バックログ） | ステータス                   |
| ----------------------------- | --------------------------------- | ---------------------------------- | ---------------------------- |
| Per-Monitor DPI Aware v2 宣言 | ✅                                 | 要件に含む                         | 実装済み                     |
| GetDpiForWindow DPI 取得      | ✅                                 | 要件に含む                         | 実装済み                     |
| DPI → Arrangement.scale 伝播  | ✅                                 | 要件に含む                         | 実装済み                     |
| WM_DPICHANGED 処理            | ✅                                 | 要件に含む                         | 実装済み                     |
| 論理/物理変換 API             | ✅ DPI::to_physical_*/to_logical_* | 要件に含む                         | 実装済み                     |
| DPI 変更イベント              | ✅ Changed<DPI>                    | 要件に含む                         | 実装済み                     |
| **DPI 変更時リソース再作成**  | —                                 | 要件に含む                         | ❌ 未実装                     |
| **ちらつきなし滑らかな更新**  | —                                 | 要件に含む                         | ❌ 未検証                     |
| **BoxStyle 座標系統一**       | —                                 | 暗黙的に必要                       | ❌ 未実装                     |
| **マルチモニター DPI 切替**   | —                                 | 要件に含む                         | ⚠ 動作するが1.25倍速バグあり |

**評価**: P1 仕様の要件の多くは dpi-propagation で実装済み。未実装は「リソース再作成」「ちらつき抑制」「座標系統一」の3点であり、現実的なスコープ。

---

## 6. 🔍 最重要 Research Needed 項目

### RN-1: SetWindowPos の実引数ランタイム確認（G1 直結）
- `flush_window_pos_commands()` 内で `SetWindowPos` に渡される最終引数をログ出力
- 期待値（bounds.left + border offset）と実値を比較
- **なぜ**: 「ログ上は正しいのに実際は1.25倍」の謎を解明する唯一の手段

### RN-2: GlobalArrangement.transform.M31 の参照箇所（G4 直結）
- `transform.M31` / `offset_x()` / `offset()` を参照する全コードパスの確認
- もし `bounds.left` の代わりに `transform.M31` を使用する箇所があれば、`offset * DPI_scale` の値が使われ 1.25 倍ズレが発生する
- **なぜ**: `From<Arrangement> for Matrix3x2` の `translation * scale` 順序により M31 = offset * scale

### RN-3: WPF/WinUI3 座標系モデルの設計調査（G2 設計用）
- WPF: 全座標が DIP。Transform はレンダリング段階で適用
- WinUI3: Scale プロパティと独立した座標系
- wintf への適用パターン検討

---

## 7. 推奨事項

### 設計フェーズへの引き継ぎ

1. **ランタイムデバッグを最優先タスクとして設計に組み込む**
   - 静的解析では 1.25 倍速バグの根本原因を完全に特定できなかった
   - `SetWindowPos` の実引数、`From<Arrangement> for Matrix3x2` の transform.M31 使用箇所の確認が必須

2. **Option C（Hybrid）アプローチを推奨**
   - Quick Fix でドラッグバグを解消 → レポート作成 → P1 設計に反映

3. **レポート（Requirement 6）は調査と並行して執筆**
   - 本 gap analysis の内容をベースに report.md を構成
   - ランタイムデバッグ結果を追記

---

## 実装複雑度と全体リスク

**Effort**: M (3-7日)
- ランタイムデバッグ: S (1-3日)
- レポート作成: S (1-2日)
- 既知ギャップの文書化: S (1日)

**Risk**: Medium
- ランタイムデバッグで原因特定できれば Low に下がる
- 座標系統一は別仕様（P1）にスコープ分離で軽減

**根拠**: 調査仕様のため新規コード実装は少ないが、ランタイムデバッグに不確実性があるため Medium

---

## 8. Requirements Review Log

> レビュー実施日: 2026-02-11

### レビュープロセス

要件定義およびギャップ分析レポートを踏まえ、修正点・疑問点・不安点を以下の3カテゴリに分類して処理した。

### カテゴリ A: 自明な修正（即修正・コミット済み）

| # | 項目 | 修正内容 | コミット |
|---|------|----------|--------|
| A1 | spec.json 未作成 | 調査仕様として spec.json を作成 | 9366eb5 |
| A2 | Req 3 AC 3 の前提バイアス | 「固定化されている変数を特定する」→「固定化の有無を調査する」に中立化 | 9366eb5 |
| A3 | Req 6 AC 3 の Mermaid 強制 | 「全図表にMermaid」→「フロー図にMermaid、一覧表にMarkdown table」に緩和 | 9366eb5 |

### カテゴリ B: 設計判断（設計フェーズで扱う）

| # | 項目 |
|---|------|
| B1 | report.md の最終構造は Req 6 AC 2 のセクション構成に従う |
| B2 | ギャップ分析の移行アプローチ比較を提言として report に含める |
| B3 | G8 (transform/ モジュール) は解決済み。report では簡潔に言及のみ |

### カテゴリ C: 開発者確認ディスカッション

#### 議題 C1: Req 3「根本原因の特定」の完了条件 — ✅ クローズ

**背景**: Req 3 の Objective が「1.25倍速バグの根本原因を特定したい」となっており、ギャップ分析では静的解析のみでは原因確定不可（ランタイムデバッグが必要）という結論だった。

**開発者方針**: 「本仕様は『あるべき姿』を固めることが主目的。バグの原因特定はスコープ外。現行チェーンと To-Be のコンフリクト箇所の指摘は歓迎するが、ブロック要素の解決に固執する必要はない」

**決定事項**:
- 選択肢 A（仮説列挙に留める）を採用
- Req 3 を「ドラッグ座標変換チェーン評価（As-Is / To-Be 対比）」にリフォーカス
- Introduction をバグ起点 → 設計指針起点に修正
- Req 5 AC 2 を Quick Fix vs Arch Fix → 段階的移行 vs 一括移行のロードマップ比較に修正
- Req 6 AC 2 (d) セクション名を「ドラッグ座標変換チェーン評価」に変更
- コミット: 65832b3

---

## 9. セッション再開ガイド

### 現在のフェーズ状態

| 成果物 | 状態 |
|----------|------|
| spec.json | ✅ 作成済み (`phase: requirements-draft`) |
| requirements.md | ✅ 作成済み・レビュー済み（承認待ち） |
| gap-analysis.md | ✅ 作成済み |
| design.md | ❌ 未作成 |
| tasks.md | ❌ 未作成 |
| report.md | ❌ 未作成（最終成果物） |

### 次のアクション

1. 要件定義を承認し、`/kiro-spec-design dpi-coordinate-transform-survey` で設計フェーズに進む
2. 設計フェーズでは以下を重点的に扱う:
   - **Req 4 (To-Be)** が最重要: WPF/WinUI3 の DIP 統一モデル調査と wintf への適用設計
   - **Req 3 (As-Is/To-Be 対比)**: 現行チェーンと理想チェーンの併記・コンフリクト指摘
   - **Req 1, 2 (As-Is)**: インベントリとフロー図はギャップ分析で大部分完了済み、report.md に整形する

### スコープ制約（確定済み）

- ✅ コード修正はスコープ外
- ✅ バグ原因特定はスコープ外
- ✅ 「あるべき姿」の設計指針を固めることが主目的
- ✅ As-Is と To-Be のコンフリクト箇所の指摘はスコープ内
- ✅ 成果物は `report.md`（提言文書）
