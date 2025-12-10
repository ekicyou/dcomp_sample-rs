# Requirements Document

## Project Description (Input)
pastaエンジンは初期化時にスクリプトディレクトリの絶対・相対パスを与える仕様になっているか。スクリプトパスからの相対パスでDSL/runeファイルの配置ルールに従い、DSL/runeファイルを読み込んで起動準備完了するか。DSL/スクリプト配置ルールは定まっているか。テスト用のスクリプトディレクトリは存在するか。関連するテストは整備されているか。

## はじめに

本仕様は、Pastaスクリプトエンジンにディレクトリベースのスクリプトローダー機能を追加するための要件を定義します。現在、PastaEngineは文字列ベースのスクリプト読み込み（`new(script: &str)`）のみをサポートしていますが、実用的なアプリケーション開発では、複数のPasta DSLファイルを体系的に管理・読み込むディレクトリ構造が必要です。

本機能により、開発者はスクリプトファイルを論理的に整理し、メンテナンス性と再利用性を向上させることができます。

## Requirements

### Requirement 1: スクリプトディレクトリ初期化

**目的:** PastaEngineの開発者として、絶対パスでスクリプトディレクトリを指定してエンジンを初期化したい。これにより、スクリプトファイルの配置場所を明確に管理できる。

#### Acceptance Criteria
1. The Pastaエンジン shall 絶対パスのみを受け付ける（相対パスは拒否する）
2. When 開発者が絶対パスを指定してPastaEngineを初期化する場合、the Pastaエンジン shall 指定されたディレクトリをスクリプトルートとして設定する
3. If 指定されたディレクトリが存在しない場合、then the Pastaエンジン shall 初期化時に即座に`PastaError::DirectoryNotFound`を返す
4. If 指定されたパスがディレクトリではなくファイルの場合、then the Pastaエンジン shall 初期化時に即座に`PastaError::NotADirectory`を返す
5. The Pastaエンジン shall 初期化時にディレクトリの読み取り権限を検証し、失敗時は即座に`PastaError::PermissionDenied`を返す
6. The Pastaエンジン shall 初期化処理を遅延せず、コンストラクタ実行時に全ての検証を完了する

### Requirement 2: DSL/Runeファイル配置ルール

**目的:** PastaEngineの開発者として、areka-P0-script-engineで定義されたファイル構成規約に従い、スクリプトを整理したい。これにより、宣言的な会話データ（pasta）と手続き的なロジック（rune）を明確に分離できる。

**参照仕様**: areka-P0-script-engine「ファイル構成とロード規則」

#### Acceptance Criteria
1. The Pastaエンジン shall スクリプトディレクトリ配下の`dic/`サブディレクトリ内の`.pasta`ファイルのみを検索する
2. The Pastaエンジン shall `dic/`ディレクトリを再帰的に探索し、全サブディレクトリ内の`.pasta`ファイルを読み込む（`./dic/**/*.pasta`パターン）
3. The Pastaエンジン shall スクリプトディレクトリ直下（ルート階層）の`.rn`ファイルをRune補助スクリプトとして認識する
4. If `dic/`内に`.rn`ファイルが存在する場合、then the Pastaエンジン shall 警告ログを出力する（技術的に可能だが作法違反）
5. The Pastaエンジン shall 拡張子を大文字小文字区別なく認識する（`.PASTA`, `.Rn`も認識）
6. The Pastaエンジン shall ファイル名が`_`（アンダースコア）で始まるファイルをスキップする
7. The Pastaエンジン shall 隠しファイル（`.`で始まる）を自動的に除外する
8. The Pastaエンジン shall ファイル探索順序を保証せず、ファイルシステム依存の順序で処理する
9. The Pastaエンジン shall Rust標準ライブラリ（`std::fs::read_to_string`）のUTF-8取り扱いルールに準じてファイルを読み込む
10. If `dic/`ディレクトリが存在しない場合、then the Pastaエンジン shall 初期化時に`PastaError::DicDirectoryNotFound`を返す
11. When `dic/`内に`.pasta`ファイルが存在しない場合、the Pastaエンジン shall 警告ログを出力する
12. When 空の`.pasta`ファイルが存在する場合、the Pastaエンジン shall 警告ログを出力しエラーとしては扱わない

### Requirement 3: スクリプトファイル読み込み

**目的:** PastaEngineの開発者として、配置ルールに基づいてスクリプトファイルを自動的に読み込みたい。これにより、手動でファイルパスを指定する手間を省ける。

#### Acceptance Criteria
1. When Pastaエンジンが初期化される場合、the Pastaエンジン shall スクリプトディレクトリ内の全`.pasta`ファイルをパースする
2. When 複数の`.pasta`ファイルが存在する場合、the Pastaエンジン shall 全ファイルのラベルを単一のラベルテーブルへ統合する
3. If 複数のファイルで同名のグローバルラベルが定義されている場合、then the Pastaエンジン shall 全ての定義を保持し、実行時にランダム選択の対象とする
4. When `.pasta`ファイルのパース中にエラーが発生した場合、the Pastaエンジン shall 可能な限り全ファイルをパースし、全てのエラーを収集する
5. When 全`.pasta`ファイルのパースが完了した場合、the Pastaエンジン shall スクリプトルートディレクトリに`pasta_errors.log`を出力する
6. The `pasta_errors.log` shall 各エラーについてファイルパス・行番号・列番号・エラー詳細を含む
7. If 1つ以上のパースエラーが収集された場合、then the Pastaエンジン shall 初期化時に`PastaError::MultipleParseErrors`を返す
8. If ファイルの読み込み中にI/Oエラーが発生した場合、then the Pastaエンジン shall 即座にエラー発生ファイルパスを含む`PastaError::IoError`を返す
9. When `.rn`ファイルが存在する場合、the Pastaエンジン shall Rune `Sources`へファイル名をキーとして追加する
10. The Pastaエンジン shall `.rn`ファイルをRune標準のモジュールシステム（`use`文でのインポート）で利用可能にする
11. If `.rn`ファイル間で循環依存が存在する場合、then the Pastaエンジン shall Runeコンパイラが検出したエラーをそのまま返す

### Requirement 4: ラベル名前空間管理

**目的:** PastaEngineの開発者として、複数ファイル間でのラベル名の衝突を適切に管理したい。これにより、スクリプトの整合性を保ちながらモジュール化を進められる。

#### Acceptance Criteria
1. The Pastaエンジン shall 全ファイルのグローバルラベルを単一のグローバル名前空間へ登録する
2. When 同名のグローバルラベルが複数存在する場合（同一ファイル内または異なるファイル間）、the Pastaエンジン shall 各ラベルに内部的に連番を付与して区別する（例：`挨拶_0`, `挨拶_1`, `挨拶_2`）
3. When ランタイムが同名のグローバルラベルを呼び出す場合、the Pastaエンジン shall 全ての同名ラベル定義からランダムに1つを選択する
4. The Pastaエンジン shall ローカルラベル（`＊＊`）を親グローバルラベルに紐づけて登録する
5. When 親ラベルが同名で複数存在する場合、the Pastaエンジン shall 各親ラベルインスタンス（連番付き）ごとにローカルラベルを独立して管理する
6. The Pastaエンジン shall 親ラベルAのローカルラベルから、同名の親ラベルBのローカルラベルへのアクセスを禁止する
7. When ローカルラベルを実行する場合、the Pastaエンジン shall カレント親ラベルコンテキスト内のローカルラベルのみを検索対象とする

### Requirement 5: テスト用スクリプトディレクトリ

**目的:** PastaEngineの開発者として、既存のサンプルスクリプトをテスト用フィクスチャとしても活用したい。これにより、areka-P0-script-engineの規約に準拠したディレクトリ構造でローダー機能を検証できる。

**参照**: 既存の`crates/pasta/examples/scripts/`を活用

#### Acceptance Criteria
1. The プロジェクト shall `crates/pasta/examples/scripts/`を統合テストのフィクスチャとして使用する
2. The スクリプトディレクトリ shall areka-P0-script-engineの規約に従い`dic/`サブディレクトリ構造を持つ
3. The `dic/`ディレクトリ shall 既存サンプル（`01_basic_conversation.pasta`～`06_event_handlers.pasta`）を配置する
4. The `dic/`ディレクトリ shall サブディレクトリ構造のテスト用に`special/`などのネストを含む
5. The スクリプトディレクトリ shall 無効ファイル名パターンのテスト用に`_ignored.pasta`を`dic/`配下に含む
6. The スクリプトディレクトリルート shall `.rn`ファイルのテスト用にサンプルRuneモジュール（`helpers.rn`など）を配置する
7. The `dic/`ディレクトリ shall 作法違反テスト用に`.rn`ファイル（警告対象）を含む

### Requirement 6: 統合テスト

**目的:** PastaEngineの開発者として、スクリプトローダー機能が仕様通りに動作することを自動テストで検証したい。これにより、機能追加やリファクタリング時の回帰を防止できる。

#### Acceptance Criteria
1. When テストがスクリプトディレクトリからPastaEngineを初期化する場合、the テスト shall 初期化が成功することを確認する
2. When テストがディレクトリ内の全ラベルを列挙する場合、the テスト shall 期待されるラベル名が全て存在することを確認する
3. When テストが複数ファイル由来の同名ラベルを実行する場合、the テスト shall いずれかのラベル定義が選択されることを確認する
4. When テストがローカルラベルを実行する場合、the テスト shall ファイルスコープが正しく隔離されていることを確認する
5. When テストが存在しないディレクトリを指定する場合、the テスト shall 適切なエラーが返されることを確認する
6. When テストが読み取り権限のないディレクトリを指定する場合、the テスト shall 権限エラーが返されることを確認する
7. The 統合テスト shall `tests/script_loader_integration_test.rs`として実装される

### Requirement 7: エラーハンドリング

**目的:** PastaEngineの開発者として、スクリプトローダーが発生させる可能性のあるエラーを適切にハンドリングしたい。これにより、デバッグ効率とユーザー体験を向上させられる。

#### Acceptance Criteria
1. If ディレクトリが存在しない場合、then the Pastaエンジン shall `PastaError::DirectoryNotFound`エラーを返す
2. If ファイルのパースに失敗した場合、then the Pastaエンジン shall ファイルパス・行番号・エラー詳細を含む`PastaError::ParseError`を返す
3. If ファイルの読み込みに失敗した場合、then the Pastaエンジン shall ファイルパスとI/Oエラー詳細を含む`PastaError::IoError`を返す
4. If 循環依存が検出された場合、then the Pastaエンジン shall 依存チェーンを含む`PastaError::CircularDependency`を返す
5. The Pastaエンジン shall 全てのエラーに対して`std::error::Error`トレイトを実装する
6. The Pastaエンジン shall 開発者向けの詳細エラーメッセージと本番向けの簡潔エラーメッセージを提供する

### Requirement 8: パフォーマンス最適化

**目的:** PastaEngineの開発者として、大量のスクリプトファイルを効率的に読み込みたい。これにより、起動時間を短縮しユーザー体験を向上させられる。

#### Acceptance Criteria
1. The Pastaエンジン shall 既存のグローバルパースキャッシュ（`PARSE_CACHE`）をディレクトリローダーでも利用する
2. When 同一ファイル内容が再読み込みされる場合、the Pastaエンジン shall キャッシュされたAST/Runeソースを再利用する
3. The Pastaエンジン shall ファイル探索時にディレクトリエントリを遅延評価する
4. The Pastaエンジン shall 不要なファイル統計情報取得（`metadata()`呼び出し）を最小化する
5. When デバッグビルドの場合、the Pastaエンジン shall キャッシュヒット/ミス情報をログ出力する

### Requirement 9: APIデザイン

**目的:** PastaEngineの開発者として、直感的で一貫性のあるAPIを使用したい。これにより、学習コストを下げ、コードの可読性を向上させられる。

#### Acceptance Criteria
1. The Pastaエンジン shall `PastaEngine::from_directory(path: impl AsRef<Path>)`コンストラクタを提供する
2. The Pastaエンジン shall `PastaEngine::from_directory_with_selector(path: impl AsRef<Path>, selector: Box<dyn RandomSelector>)`コンストラクタを提供する
3. The Pastaエンジン shall 既存の`PastaEngine::new(script: &str)`コンストラクタとの互換性を維持する
4. The Pastaエンジン shall `list_labels(&self) -> Vec<String>`メソッドで全ラベル名を列挙可能とする
5. The Pastaエンジン shall `list_global_labels(&self) -> Vec<String>`メソッドでグローバルラベルのみを列挙可能とする
6. The Pastaエンジン shall `reload_directory(&mut self)`メソッドでディレクトリの再読み込みを可能とする
