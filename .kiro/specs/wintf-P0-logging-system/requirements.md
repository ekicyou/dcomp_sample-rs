# Requirements Document

## Project Description (Input)
現在、エラーログはeprintln!で平すら適当に出しているだけだが、bevyが提供するログシステムなどに移行した方がよいか？以降是非調査を含め仕様を立ち上げたい。要件定義では「何を使うか、あるいは諦めてこのままにする」までを決め、設計では「現状をどう変更するか」を決める感じになるかな。

## Introduction
wintfライブラリでは現在、デバッグ・エラー出力に`eprintln!`マクロを直接使用している。これは以下の課題を持つ：
- ログレベルによるフィルタリング不可
- 構造化されていない出力形式
- 本番環境での無効化が困難
- 非同期タスクのトレーシングが困難

本仕様では、適切なログシステムへの移行可否を調査・決定し、採用する場合の要件を定義する。

## Requirements

### Requirement 1: ログシステム選定
**Objective:** 開発者として、wintfに最適なログシステムを選定したい。そうすることで、一貫性のあるログ基盤を構築できる。

#### 選定結果
- **採用**: `tracing` クレート
- **理由**: 
  - Rustエコシステム標準の構造化ログ/トレーシングフレームワーク
  - bevy_logと完全互換（将来のbevy_appプラグイン導入時に移行容易）
  - ゼロコスト抽象化によりパフォーマンス影響最小
- **見送り**: bevy_log（プラグインシステム本格導入時に再検討）

#### Acceptance Criteria
1. The wintf library shall ログシステムとして`tracing`クレートを採用する
2. When ログ出力が呼び出された場合, the wintf library shall tracingのイベントマクロ（`info!`, `warn!`, `error!`等）を使用する
3. The wintf library shall Subscriberの初期化をライブラリ利用者に委ねる（ライブラリはログ発行のみ）
4. The wintf library shall 将来のbevy_log移行に備え、tracing互換のAPIのみを使用する

### Requirement 2: ログレベル対応
**Objective:** 開発者として、状況に応じてログ出力量を制御したい。そうすることで、デバッグ時は詳細に、本番時は最小限のログを得られる。

#### Acceptance Criteria
1. The wintf library shall 以下のログレベルを提供する: `error`, `warn`, `info`, `debug`, `trace`
2. When 環境変数`RUST_LOG`が設定された場合, the wintf library shall 指定されたレベル以上のログのみを出力する
3. Where リリースビルドの場合, the wintf library shall デフォルトで`warn`以上のログのみを出力する
4. Where デバッグビルドの場合, the wintf library shall デフォルトで`debug`以上のログを出力する

### Requirement 3: 構造化ログ出力
**Objective:** 開発者として、ログにコンテキスト情報を付与したい。そうすることで、問題発生時の原因特定が容易になる。

#### Acceptance Criteria
1. The wintf library shall ログにモジュール名を自動付与する
2. The wintf library shall ログにファイル名と行番号を含める（デバッグビルド時）
3. When Windowsメッセージ処理中にログが発行された場合, the wintf library shall HWNDまたはEntity IDをコンテキストとして含める

### Requirement 4: 責務分離と参考実装
**Objective:** 開発者として、ログ初期化の責任範囲を明確にしたい。そうすることで、wintfライブラリとアプリケーションの役割を正しく分離できる。

#### 設計方針
- **wintfライブラリ**: ログ発行のみ（Subscriber初期化はしない）
- **アプリケーション**: Subscriber初期化の責任を持つ
- **参考実装**: `taffy_flex_demo`に初期化コードを追加し、利用者への手本とする

#### Acceptance Criteria
1. The wintf library shall Subscriberの初期化を行わない（ログ発行のみ）
2. The wintf library shall 既存の`eprintln!`呼び出しを適切なtracingマクロに置換する
3. The taffy_flex_demo example shall main関数にtracing-subscriber初期化コードを含める
4. The taffy_flex_demo example shall RUST_LOG環境変数によるフィルタリングをサポートする

### Requirement 5: パフォーマンス影響
**Objective:** 開発者として、ログシステムがパフォーマンスに悪影響を与えないことを確認したい。そうすることで、GUIの滑らかさを維持できる。

#### Acceptance Criteria
1. When ログレベルが無効化されている場合, the wintf library shall ログ呼び出しのオーバーヘッドを最小化する（コンパイル時除去または早期リターン）
2. The wintf library shall メッセージループ内での同期ログ書き込みを避ける（非同期出力または遅延書き込み）
3. The wintf library shall ログシステム導入前後でフレームレートへの有意な影響がないことを確認する

