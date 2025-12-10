# Requirements Document

## Project Description (Input)
pastaエンジンは初期化時にシリアライズディレクトリの絶対・相対パスを与える仕様になっているか。エンジンが把握する永続化データをシリアライズパスから読み込むか。テスト用の永続化ファイルディレクトリは存在するか。テスト用永続化ディレクトリの内容はテストコードが書き換えないように、テスト時に作成されるテンポラリディレクトリ（tempfileクレートを導入するとよい）に展開されるか。エンジンはdrop時にシリアライズパスに永続化データを保存するか。関連するテストは整備されているか。

## Introduction
Pastaエンジンの永続化パス管理とRuneスクリプト向け永続化API機能を実装します。エンジンは初期化時に指定された永続化ディレクトリパスを保持し、Runeスクリプトがそのパスを使用してファイルI/Oを行えるようにします。エンジンインスタンス自体の状態は永続化せず（`pasta-engine-independence`により各インスタンスは独立）、永続化が必要なデータはRuneスクリプト側で管理します。テスト環境では、元のテストデータを保護するために一時ディレクトリを使用します。

## Requirements

### Requirement 1: エンジン初期化時の永続化パス指定
**Objective:** 開発者として、PastaEngineの初期化時に永続化ディレクトリパスを指定できるようにしたい。Runeスクリプトがファイル永続化を行う際の基準パスとして使用するため。

#### Acceptance Criteria
1. When PastaEngineを初期化する際に永続化ディレクトリの絶対パスを引数として与える場合、the PastaEngine shall そのパスをRuneスクリプトに提供する永続化パスとして保持する
2. When PastaEngineを初期化する際に永続化ディレクトリの相対パスを引数として与える場合、the PastaEngine shall カレントディレクトリを基準としてパスを解決し、絶対パスに変換して保持する
3. When 永続化ディレクトリパスが指定されない場合（`None`または`Option::None`）、the PastaEngine shall 永続化パスなしで初期化し、Runeスクリプトからの永続化関数呼び出し時にエラーを返す
4. If 指定された永続化ディレクトリパスが無効または存在しない場合、then the PastaEngine shall 適切なエラーを返し、エンジンの初期化を失敗させる
5. The PastaEngine shall 設定された永続化ディレクトリパスをエンジンのライフタイム全体で保持し、Runeランタイムからアクセス可能にする

### Requirement 2: Runeスクリプト向け永続化API
**Objective:** Runeスクリプト開発者として、標準ライブラリ経由で永続化ディレクトリにデータを保存・読み込みできるようにしたい。セッション間でスクリプトレベルのデータを永続化できるようにするため。

#### Acceptance Criteria
1. The pasta stdlib shall `save_data(key: String, value: String)`関数を提供し、指定されたキーと値を永続化ディレクトリ内のファイルに保存する
2. The pasta stdlib shall `load_data(key: String) -> Option<String>`関数を提供し、指定されたキーの値を永続化ディレクトリから読み込む
3. When Runeスクリプトが`save_data`を呼び出し、永続化パスが設定されていない場合、the pasta stdlib shall エラーイベント（`ScriptEvent::Error`）を返す
4. When Runeスクリプトが`load_data`を呼び出し、キーに対応するファイルが存在しない場合、the pasta stdlib shall `None`を返す（エラーではない）
5. The pasta stdlib shall 永続化ファイル名をキー文字列から安全に生成し（サニタイズ、拡張子`.dat`または`.txt`追加）、パストラバーサル攻撃を防ぐ
6. When 永続化ファイルの読み書きに失敗した場合（権限エラー、I/Oエラー）、the pasta stdlib shall エラーイベント（`ScriptEvent::Error`）を返し、エラー詳細をログ出力する

### Requirement 3: テスト用永続化ディレクトリの管理
**Objective:** テストエンジニアとして、テスト実行時に元のテストデータを保護しながら永続化機能を検証したい。テストごとに独立した一時ディレクトリを使用できるようにするため。

#### Acceptance Criteria
1. When テストコードがテスト用永続化ディレクトリを準備する場合、the テストフレームワーク shall tempfileクレートを使用して一時ディレクトリを作成する
2. When テスト開始時にテスト用固定データが必要な場合、the テストコード shall リポジトリ内の固定テストデータディレクトリから一時ディレクトリへファイルをコピーする
3. While テスト実行中の場合、the PastaEngine shall 一時ディレクトリ内でのみ永続化データの読み書きを行い、元のテストデータディレクトリを変更しない
4. When テスト終了時、the テストフレームワーク shall 一時ディレクトリとその内容を自動的に削除する
5. The テストコード shall リポジトリ内のテスト用固定データディレクトリ（例: `tests/fixtures/persistence/`）を参照し、このディレクトリはバージョン管理に含める

### Requirement 4: 永続化パスのRuneランタイムへの提供
**Objective:** pasta stdlib開発者として、Runeランタイムから永続化ディレクトリパスにアクセスできるようにしたい。stdlib関数が永続化ファイルの読み書きを実行できるようにするため。

#### Acceptance Criteria
1. The PastaEngine shall 永続化ディレクトリパスを`Option<PathBuf>`として内部フィールドに保持する
2. When Runeランタイムから永続化関連のstdlib関数が呼び出される場合、the PastaEngine shall 現在の永続化パスをstdlib関数に提供する仕組みを実装する（スレッドローカル、コンテキスト渡し、またはクロージャキャプチャ）
3. The PastaEngine shall 永続化パスの提供方法がスレッドセーフであり、複数エンジンインスタンスが独立して動作できることを保証する
4. When エンジンインスタンスが破棄される場合、the PastaEngine shall 永続化パスも含めすべてのデータが所有権により自動解放されることを保証する
5. The PastaEngine shall 永続化パスの設定がエンジン初期化後に変更されないことを保証する（イミュータブル）

### Requirement 5: 永続化ファイルの形式と安全性
**Objective:** Runeスクリプト開発者として、永続化データの形式とセキュリティ制約が明確に定義されていることを確認したい。データの互換性とセキュリティを保証するため。

#### Acceptance Criteria
1. The pasta stdlib shall 永続化データをプレーンテキスト（UTF-8）形式で保存する（キーごとに1ファイル、内容は文字列値）
2. The pasta stdlib shall 永続化ファイル名を`{sanitized_key}.dat`形式で生成し、キー文字列から不正な文字（`/`, `\`, `..`など）を除去またはエスケープする
3. If キー文字列にパストラバーサル攻撃を示すパターン（`../`, `..\`など）が含まれる場合、then the pasta stdlib shall エラーを返し、ファイル操作を拒否する
4. The pasta stdlib shall 永続化ディレクトリ外へのファイルアクセスを構造的に防止する（永続化パス + サニタイズ済みファイル名のみ許可）
5. The pasta stdlib shall 将来的にバイナリ形式やJSON形式への拡張が可能な設計とする（現時点ではプレーンテキストのみ実装）

### Requirement 6: テストカバレッジ
**Objective:** 品質保証担当者として、永続化パス管理とRuneスクリプト永続化APIが適切にテストされていることを確認したい。リグレッションを防ぎ、機能の信頼性を保証するため。

#### Acceptance Criteria
1. The プロジェクト shall 絶対パス・相対パスの両方を使用したエンジン初期化テストを含む
2. The プロジェクト shall 永続化パスなし（`None`）でエンジンを初期化し、永続化関数呼び出し時にエラーが返されることを検証するテストを含む
3. The プロジェクト shall Runeスクリプトから`save_data`と`load_data`を呼び出し、データが正しく保存・読み込みされることを検証するテストを含む
4. The プロジェクト shall 存在しないキーの`load_data`が`None`を返すことを検証するテストを含む
5. The プロジェクト shall パストラバーサル攻撃を試みるキー文字列（`../sensitive_file`など）が拒否されることを検証するテストを含む
6. The プロジェクト shall 一時ディレクトリ（tempfileクレート使用）を使用し、テスト終了後に自動削除されることを検証するテストを含む
7. The プロジェクト shall 複数のエンジンインスタンスが異なる永続化ディレクトリを使用し、互いに干渉しないことを検証するテストを含む

### Requirement 7: エラーハンドリングとロギング
**Objective:** 運用担当者として、永続化処理のエラーや警告が適切にログ出力されることを確認したい。問題の診断と対処を容易にするため。

#### Acceptance Criteria
1. When Runeスクリプトが`save_data`を呼び出し、ファイル書き込みに失敗した場合、the pasta stdlib shall `error!` レベルでエラー詳細（パス、キー、I/Oエラー）をログ出力し、`ScriptEvent::Error`を返す
2. When Runeスクリプトが`load_data`を呼び出し、ファイル読み込みに失敗した場合（権限エラーなど）、the pasta stdlib shall `error!` レベルでエラー詳細をログ出力し、`ScriptEvent::Error`を返す
3. When Runeスクリプトが永続化関数を呼び出し、永続化パスが設定されていない場合、the pasta stdlib shall `warn!` レベルで警告をログ出力し、`ScriptEvent::Error`を返す
4. When パストラバーサル攻撃が検出された場合、the pasta stdlib shall `warn!` レベルでセキュリティ警告をログ出力し、操作を拒否する
5. When 永続化処理が正常完了した場合、the pasta stdlib shall `debug!` レベルで操作詳細（キー、ファイルサイズなど）をログ出力する
6. The pasta stdlib shall すべての永続化関連ログに構造化フィールド（`key`, `path`, `error`, `operation`など）を含める
