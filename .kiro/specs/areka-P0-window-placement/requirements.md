# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka-window-placement 要件定義書 |
| **Version** | 0.1 (Draft) |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Author** | AI-DLC System |

---

## Introduction

本仕様書は「areka」プラットフォームにおけるウィンドウ配置システムの要件を定義する。デスクトップマスコットアプリケーション特有の配置ルール（タスクバー張り付き、画面端配置、マルチモニター対応）を提供する。

### 背景

デスクトップマスコットアプリケーションでは、キャラクターの配置位置がユーザー体験に大きく影響する。従来の「伺か」では、キャラクターはデスクトップの右下、タスクバーの上に「立っている」ように配置されることが一般的だった。この「居場所」の概念は、キャラクターの存在感を演出する重要な要素である。

本仕様では、以下の配置機能を定義する：
- タスクバーへの張り付き（タスクバーの上に立つ）
- 画面端への配置（左端、右端、上端、下端）
- 自由配置（任意の座標）
- マルチモニター対応
- 複数キャラクター間の相対配置

### スコープ

**含まれるもの**:
- キャラクターウィンドウの配置ルール
- タスクバー位置の検出と張り付き
- 画面端へのスナップ配置
- マルチモニター対応（モニター間移動、モニター固定）
- 複数キャラクター間の相対位置管理
- 配置の保存・復元

**含まれないもの**:
- ウィンドウの描画（wintf で対応）
- ドラッグ移動のイベント処理（wintf-event-system で対応）
- Z-order管理（常に最前面は wintf で対応）

### 親仕様からの要件マッピング

本仕様は以下の親要件に対応する：
- **Requirement 1.4**: キャラクターをデスクトップの任意の位置に配置できる
- **Requirement 1.5**: 複数キャラクター間の相対位置関係を定義できる
- **Requirement 1.7**: マルチモニター環境ですべてのモニターにまたがって配置できる
- **Requirement 9.3**: キャラクターの表示位置を記憶し、次回起動時に復元する
- **Requirement 16.6**: キャラクターの移動範囲を制限できる（特定のモニター、画面端のみ等）

---

## Requirements

### Requirement 1: 配置モード

**Objective:** ユーザーとして、キャラクターの配置方法を選択したい。それにより自分の好みに合った配置でキャラクターを表示できる。

#### Acceptance Criteria

1. **The** Placement System **shall** 以下の配置モードを提供する：
   - `Free`: 自由配置（任意の座標）
   - `ScreenEdge`: 画面端張り付き
   - `TaskbarDock`: タスクバー張り付き
2. **The** Placement System **shall** 配置モードを動的に切り替えられる
3. **When** 配置モードが変更された時, **the** Placement System **shall** キャラクターを新しい配置ルールに従って再配置する
4. **The** Placement System **shall** デフォルトの配置モードを設定できる
5. **The** Placement System **shall** キャラクターごとに異なる配置モードを設定できる

---

### Requirement 2: タスクバー張り付き

**Objective:** ユーザーとして、キャラクターをタスクバーの上に「立たせたい」。それによりキャラクターが自然にデスクトップに存在する感覚を得られる。

#### Acceptance Criteria

1. **The** Placement System **shall** タスクバーの位置（上下左右）を検出する
2. **The** Placement System **shall** タスクバーの高さ/幅を検出する
3. **When** タスクバー張り付きモードの場合, **the** Placement System **shall** キャラクターをタスクバーの上（または横）に配置する
4. **When** タスクバーの位置が変更された時, **the** Placement System **shall** キャラクターの位置を自動調整する
5. **When** タスクバーが自動非表示の場合, **the** Placement System **shall** 画面端を基準に配置する
6. **The** Placement System **shall** タスクバー上の水平位置（左寄せ/中央/右寄せ/任意）を指定できる

---

### Requirement 3: 画面端配置

**Objective:** ユーザーとして、キャラクターを画面の端に配置したい。それによりデスクトップの作業領域を最大限確保できる。

#### Acceptance Criteria

1. **The** Placement System **shall** 画面端（上/下/左/右/四隅）への配置をサポートする
2. **When** 画面端配置モードの場合, **the** Placement System **shall** 指定された端にキャラクターをスナップする
3. **The** Placement System **shall** 画面端からのオフセット（マージン）を指定できる
4. **When** 画面解像度が変更された時, **the** Placement System **shall** キャラクターの位置を自動調整する
5. **The** Placement System **shall** 画面端での水平/垂直位置を指定できる

---

### Requirement 4: 自由配置

**Objective:** ユーザーとして、キャラクターを任意の位置に配置したい。それにより自分の好きな場所にキャラクターを置ける。

#### Acceptance Criteria

1. **The** Placement System **shall** 任意の座標（x, y）にキャラクターを配置できる
2. **When** ユーザーがドラッグ移動した時, **the** Placement System **shall** 新しい位置を保存する
3. **The** Placement System **shall** 座標系（スクリーン座標/ワークエリア座標）を指定できる
4. **When** キャラクターが画面外に移動した時, **the** Placement System **shall** 画面内に戻すオプションを提供する
5. **The** Placement System **shall** スナップグリッド（任意のピクセル単位に吸着）をサポートする

---

### Requirement 5: マルチモニター対応

**Objective:** ユーザーとして、複数モニター環境でキャラクターを自由に配置したい。それにより広いデスクトップ環境を活用できる。

#### Acceptance Criteria

1. **The** Placement System **shall** 複数モニターの構成を検出する
2. **The** Placement System **shall** 各モニターの位置、サイズ、DPIを取得する
3. **When** キャラクターがモニター間を移動した時, **the** Placement System **shall** 移動先モニターを認識する
4. **The** Placement System **shall** キャラクターを特定のモニターに固定するオプションを提供する
5. **When** モニター構成が変更された時, **the** Placement System **shall** キャラクターの位置を適切に調整する
6. **The** Placement System **shall** プライマリモニター/セカンダリモニターを区別する

---

### Requirement 6: 複数キャラクター配置

**Objective:** ユーザーとして、複数のキャラクターを自然な位置関係で配置したい。それによりキャラクター同士の掛け合いを視覚的に楽しめる。

#### Acceptance Criteria

1. **The** Placement System **shall** メインキャラクター（\0）と相方キャラクター（\1）の相対位置を定義できる
2. **The** Placement System **shall** 以下の相対配置パターンをサポートする：
   - `SideBySide`: 横並び
   - `FacingEach`: 向かい合い
   - `Stacked`: 縦並び
   - `Custom`: カスタム相対位置
3. **When** メインキャラクターが移動した時, **the** Placement System **shall** 相方キャラクターを追従させるオプションを提供する
4. **The** Placement System **shall** キャラクター間の間隔（ギャップ）を指定できる
5. **The** Placement System **shall** 相方キャラクターの独立移動を許可/禁止できる

---

### Requirement 7: 移動範囲制限

**Objective:** ユーザーとして、キャラクターの移動範囲を制限したい。それにより意図しない場所への移動を防げる。

#### Acceptance Criteria

1. **The** Placement System **shall** キャラクターの移動範囲を制限できる
2. **The** Placement System **shall** 以下の制限オプションを提供する：
   - 特定のモニター内のみ
   - 画面端からの距離制限
   - カスタム矩形領域内のみ
3. **When** キャラクターが制限領域外に移動しようとした時, **the** Placement System **shall** 移動を制限する
4. **The** Placement System **shall** 制限の有効/無効を切り替えられる
5. **The** Placement System **shall** 制限領域を視覚的にプレビューできる（開発者向け）

---

### Requirement 8: 配置の保存・復元

**Objective:** ユーザーとして、キャラクターの配置を記憶してほしい。それにより次回起動時に同じ場所からスタートできる。

#### Acceptance Criteria

1. **The** Placement System **shall** キャラクターの位置を自動保存する
2. **When** アプリケーションが起動した時, **the** Placement System **shall** 前回の位置を復元する
3. **The** Placement System **shall** 配置モードを保存・復元する
4. **When** モニター構成が変更された時, **the** Placement System **shall** 近似位置に復元するか、デフォルト位置に戻す
5. **The** Placement System **shall** 複数の配置プリセットを保存・切り替えできる

---

### Requirement 9: システムイベント対応

**Objective:** 開発者として、システムイベントに応じて配置を調整したい。それにより環境変化に自動対応できる。

#### Acceptance Criteria

1. **When** 画面解像度が変更された時, **the** Placement System **shall** 配置を再計算する
2. **When** モニターが追加/削除された時, **the** Placement System **shall** 配置を調整する
3. **When** タスクバー位置/サイズが変更された時, **the** Placement System **shall** 張り付き位置を更新する
4. **When** DPI設定が変更された時, **the** Placement System **shall** 座標をスケーリングする
5. **The** Placement System **shall** システムイベントをイベントシステムに通知する

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

- 配置計算: 1ms以内
- システムイベント応答: 100ms以内
- モニター情報取得: 10ms以内

### NFR-2: 精度

- 座標計算: ピクセル単位の精度
- DPIスケーリング: 正確なスケーリング
- タスクバー検出: リアルタイムの位置追従

### NFR-3: 永続性

- 位置情報の即座保存（遅延は1秒以内）
- クラッシュ時も最後の位置を保持

---

## Glossary

| 用語 | 説明 |
|------|------|
| タスクバー | Windowsの画面端にあるアプリケーションバー |
| ワークエリア | タスクバーを除いた利用可能なデスクトップ領域 |
| スクリーン座標 | 画面全体を基準とした座標系 |
| DPI | Dots Per Inch。高DPIでは座標のスケーリングが必要 |
| スナップ | 特定の位置やグリッドに吸着する動作 |
| プライマリモニター | Windowsで主として設定されたモニター |

---

## Appendix

### A. 関連ドキュメント

- 親仕様: `.kiro/specs/ukagaka-desktop-mascot/requirements.md`
- イベントシステム: `.kiro/specs/wintf-event-system/requirements.md`

### B. タスクバー位置検出

```rust
// Win32 API を使用したタスクバー位置検出の概要
fn get_taskbar_info() -> TaskbarInfo {
    let appbar_data = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        ..Default::default()
    };
    
    // SHAppBarMessage で取得
    unsafe {
        SHAppBarMessage(ABM_GETTASKBARPOS, &mut appbar_data);
    }
    
    TaskbarInfo {
        edge: match appbar_data.uEdge {
            ABE_BOTTOM => Edge::Bottom,
            ABE_TOP => Edge::Top,
            ABE_LEFT => Edge::Left,
            ABE_RIGHT => Edge::Right,
            _ => Edge::Bottom,
        },
        rect: appbar_data.rc.into(),
        auto_hide: is_auto_hide_enabled(),
    }
}
```

### C. 配置モード設定例

```toml
[placement]
mode = "TaskbarDock"  # Free, ScreenEdge, TaskbarDock

[placement.taskbar_dock]
horizontal_align = "right"  # left, center, right, custom
offset_x = -50  # タスクバー上での水平オフセット

[placement.screen_edge]
edge = "bottom-right"  # top, bottom, left, right, top-left, etc.
margin = 10

[placement.free]
x = 800
y = 600

[placement.multi_character]
pattern = "SideBySide"  # SideBySide, FacingEach, Stacked, Custom
gap = 20  # キャラクター間の間隔
follow_main = true  # 相方がメインに追従

[placement.constraints]
enabled = true
monitor = "primary"  # primary, secondary, any, monitor_index
```

### D. マルチモニター座標系

```
+-----------------+
|   Monitor 0     |
| (0,0)-(1920,1080)|
+-----------------++-----------------+
                  |   Monitor 1     |
                  | (1920,0)-(3840,1080)|
                  +-----------------+

スクリーン座標: モニター0の左上を(0,0)とした絶対座標
ワークエリア座標: タスクバーを除いた領域内の座標
```

---

_Document generated by AI-DLC System on 2025-11-29_
