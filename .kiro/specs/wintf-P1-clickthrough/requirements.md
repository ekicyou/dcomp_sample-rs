# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | wintf-clickthrough 要件定義書 |
| **Version** | 0.1 (Draft) |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Author** | AI-DLC System |

---

## Introduction

本仕様書は wintf フレームワークにおけるクリックスルー機能の要件を定義する。親仕様「伺的デスクトップマスコットアプリ」のリリース必須機能として、透過領域のマウスイベント透過とキャラクター領域のドラッグ移動を両立させる。

### 背景

デスクトップマスコットアプリケーションでは、キャラクター画像の透明部分はデスクトップや背後のウィンドウが見えている必要がある。この透過領域をクリックした場合、マウスイベントは背後のウィンドウに「透過（クリックスルー）」されるべきである。一方、キャラクターの不透明領域をクリックした場合は、キャラクターを掴んでドラッグ移動できる必要がある。

この「透過領域はクリックスルー、不透明領域はドラッグ可能」という二重の要件を満たすには、Win32の `WM_NCHITTEST` メッセージを適切にハンドリングする必要がある。

### スコープ

**含まれるもの**:
- 透過領域のクリックスルー（マウスイベント透過）
- 不透明領域のヒット判定
- `WM_NCHITTEST` ハンドリング
- レイヤードウィンドウとの統合

**含まれないもの**:
- ヒット領域の形状定義（wintf-event-system で対応）
- ドラッグによるウィンドウ移動ロジック（wintf-event-system で対応）
- タッチイベントのクリックスルー

### 親仕様からの要件マッピング

本仕様は以下の親要件に対応する：
- **Requirement 1.6**: キャラクターが表示されている間、他のウィンドウの操作を妨げない（クリックスルー対応）

---

## Requirements

### Requirement 1: 透過領域クリックスルー

**Objective:** ユーザーとして、キャラクターの透明部分をクリックしたときに背後のウィンドウを操作したい。それによりデスクトップ操作を妨げずにキャラクターを表示できる。

#### Acceptance Criteria

1. **When** ユーザーが透過領域（アルファ=0）をクリックした時, **the** Window **shall** マウスイベントを背後のウィンドウに透過する
2. **When** ユーザーが不透明領域（アルファ>0）をクリックした時, **the** Window **shall** マウスイベントを受け取る
3. **The** Window **shall** `WM_NCHITTEST` メッセージに対して適切なヒットテスト結果を返す
4. **When** 透過領域でホバーした時, **the** Window **shall** マウスカーソルを背後のウィンドウに委譲する
5. **The** Window **shall** ピクセル単位のアルファ値に基づいてヒット判定を行う

---

### Requirement 2: アルファ閾値設定

**Objective:** 開発者として、クリックスルーのアルファ閾値を調整したい。それにより半透明領域の扱いを制御できる。

#### Acceptance Criteria

1. **The** Window **shall** クリックスルーのアルファ閾値を設定できる（デフォルト: 0）
2. **When** アルファ値が閾値以下の場合, **the** Window **shall** その領域をクリックスルー対象とする
3. **When** アルファ値が閾値より大きい場合, **the** Window **shall** その領域をヒット対象とする
4. **The** Window **shall** 閾値を0〜255の範囲で指定できる
5. **The** Window **shall** 閾値の変更を即座に反映する

---

### Requirement 3: WM_NCHITTEST ハンドリング

**Objective:** 開発者として、Win32 の `WM_NCHITTEST` を適切にハンドリングしたい。それによりOSレベルでクリックスルーが機能する。

#### Acceptance Criteria

1. **The** Window **shall** `WM_NCHITTEST` メッセージをインターセプトする
2. **When** ヒット位置が透過領域の場合, **the** Window **shall** `HTTRANSPARENT` を返す
3. **When** ヒット位置が不透明領域の場合, **the** Window **shall** `HTCLIENT` または適切なヒットコードを返す
4. **The** Window **shall** ヒットテストのパフォーマンスを最適化する（キャッシュ等）
5. **When** ウィンドウサイズが変更された時, **the** Window **shall** ヒットテスト領域を更新する

---

### Requirement 4: ドラッグ可能領域

**Objective:** ユーザーとして、キャラクターの不透明部分をドラッグしてウィンドウを移動したい。それによりキャラクターを好きな位置に配置できる。

#### Acceptance Criteria

1. **When** 不透明領域でマウスボタンを押した時, **the** Window **shall** ドラッグ操作を開始できる
2. **The** Window **shall** ドラッグ可能領域を明示的に指定できる（全不透明領域 or カスタム領域）
3. **When** ドラッグ可能領域以外の不透明領域をクリックした時, **the** Window **shall** ドラッグを開始しない（通常のクリックとして処理）
4. **The** Window **shall** `HTCAPTION` を返すことでOSネイティブのウィンドウドラッグをサポートする
5. **The** Window **shall** カスタムドラッグ処理とOSネイティブドラッグを切り替えられる

---

### Requirement 5: レイヤードウィンドウ統合

**Objective:** 開発者として、レイヤードウィンドウとクリックスルーを統合したい。それによりDirectComposition ベースの透過ウィンドウでクリックスルーが機能する。

#### Acceptance Criteria

1. **The** Window **shall** `WS_EX_LAYERED` スタイルと連携する
2. **The** Window **shall** `WS_EX_TRANSPARENT` スタイルを適切に使用する
3. **The** Window **shall** DirectComposition Visual の透過情報を参照できる
4. **When** サーフェスの内容が更新された時, **the** Window **shall** ヒットテスト用の透過マップを更新する
5. **The** Window **shall** 透過マップの生成をバックグラウンドで行い、描画パフォーマンスに影響を与えない

---

### Requirement 6: パフォーマンス最適化

**Objective:** 開発者として、クリックスルーのパフォーマンスを最適化したい。それにより快適なユーザー体験を提供できる。

#### Acceptance Criteria

1. **The** Window **shall** ヒットテスト結果をキャッシュする
2. **When** サーフェスが更新された時のみ, **the** Window **shall** 透過マップを再生成する
3. **The** Window **shall** 透過マップの解像度を調整できる（精度 vs パフォーマンス）
4. **The** Window **shall** ヒットテストを1ms以内で完了する
5. **The** Window **shall** 透過マップのメモリ使用量を最小化する

---

### Requirement 7: ECS統合

**Objective:** 開発者として、クリックスルー機能をECSアーキテクチャに統合したい。それにより既存のwintfパターンと一貫性を保てる。

#### Acceptance Criteria

1. **The** Clickthrough **shall** ECSコンポーネントとして実装される
2. **The** Clickthrough **shall** ウィンドウエンティティに付与できる
3. **The** Clickthrough **shall** 有効/無効を動的に切り替えられる
4. **When** エンティティが削除された時, **the** Clickthrough **shall** 関連リソースを解放する
5. **The** Clickthrough **shall** 既存のウィンドウシステムと統合される

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

- ヒットテスト応答時間: 1ms以内
- 透過マップ生成: 100ms以内（初回）、10ms以内（差分更新）
- メモリ使用量: 透過マップで画像サイズの1/64程度（ダウンサンプリング）

### NFR-2: 互換性

- Windows 10 バージョン1809以降
- DirectComposition ベースのウィンドウと互換
- 高DPI環境での正確なヒット判定

### NFR-3: 正確性

- ピクセル単位での正確なヒット判定（閾値による誤差は許容）
- 高DPI環境でのスケーリング対応

---

## Glossary

| 用語 | 説明 |
|------|------|
| クリックスルー | マウスイベントが背後のウィンドウに透過すること |
| WM_NCHITTEST | Win32メッセージ。マウス位置がウィンドウのどの領域かを判定 |
| HTTRANSPARENT | WM_NCHITTESTの戻り値。透過領域を示す |
| HTCLIENT | WM_NCHITTESTの戻り値。クライアント領域を示す |
| HTCAPTION | WM_NCHITTESTの戻り値。タイトルバー領域（ドラッグ可能）を示す |
| レイヤードウィンドウ | WS_EX_LAYEREDスタイルを持つ透過可能なウィンドウ |
| 透過マップ | 各ピクセルの透過/不透過を示すビットマップ |

---

## Appendix

### A. 関連ドキュメント

- 親仕様: `.kiro/specs/ukagaka-desktop-mascot/requirements.md`
- イベントシステム: `.kiro/specs/wintf-event-system/requirements.md`

### B. Win32 実装パターン

```rust
// WM_NCHITTEST ハンドリングの概要
fn handle_nchittest(hwnd: HWND, x: i32, y: i32) -> LRESULT {
    // スクリーン座標をクライアント座標に変換
    let client_pos = screen_to_client(hwnd, x, y);
    
    // 透過マップを参照
    if is_transparent(client_pos) {
        return HTTRANSPARENT;  // クリックスルー
    }
    
    // ドラッグ可能領域かチェック
    if is_draggable_region(client_pos) {
        return HTCAPTION;  // OSネイティブドラッグ
    }
    
    HTCLIENT  // 通常のクライアント領域
}
```

### C. 透過マップ生成戦略

1. **フルスキャン**: サーフェス全体をスキャンし、各ピクセルのアルファ値をチェック
2. **ダウンサンプリング**: 4x4 または 8x8 ピクセル単位でサンプリングし、メモリ使用量を削減
3. **差分更新**: 変更された領域のみを更新
4. **遅延生成**: 初回ヒットテスト時に生成（lazy initialization）

---

_Document generated by AI-DLC System on 2025-11-29_
