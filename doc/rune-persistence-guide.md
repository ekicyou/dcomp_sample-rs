# Rune永続化実装ガイド

本ドキュメントは、Runeスクリプト開発者向けに、Pasta Engineの永続化機能を使用してゲーム進行状況やユーザー設定を保存・読み込みする方法を説明します。

## 概要

Pasta Engineは、永続化ディレクトリパスをRuneスクリプトに提供し、以下の機能を通じてファイル永続化を実現します：

- **永続化パスの取得**: コンテキスト引数から永続化ディレクトリパスを取得
- **TOMLシリアライズ**: Rune値をTOML形式の文字列に変換
- **TOMLデシリアライズ**: TOML文字列をRune値に変換
- **ファイルI/O**: テキストファイルの読み書き

## 永続化パスの取得

すべてのラベル関数は`ctx`引数を受け取り、永続化ディレクトリパスにアクセスできます。

### 基本的な使用方法

```rune
＊save_game
    ```rune
        let path = ctx["persistence_path"];
        if path == "" {
            yield emit_text("永続化は無効です");
        } else {
            // 永続化パスを使用してファイルを保存
            yield emit_text(`データを保存: ${path}`);
        }
    ```
```

### パスの結合

```rune
＊save_data
    ```rune
        let base_path = ctx["persistence_path"];
        let save_file = `${base_path}/save_data.toml`;
        // save_fileを使用してファイルを保存
    ```
```

## TOMLシリアライズ・デシリアライズ

### データの保存

`toml_to_string(data)`関数を使用してRune値をTOML文字列に変換します。

```rune
＊save_game
    ```rune
        let path = ctx["persistence_path"];
        let save_file = `${path}/save.toml`;
        
        // 保存するデータを作成
        let data = #{
            "level": 5,
            "gold": 100,
            "player_name": "TestPlayer"
        };
        
        // TOMLに変換
        let toml_str = toml_to_string(data)?;
        
        // ファイルに書き込み
        write_text_file(save_file, toml_str)?;
        
        yield emit_text("ゲームを保存しました");
    ```
```

### データの読み込み

`toml_from_string(toml_str)`関数を使用してTOML文字列をRune値に変換します。

```rune
＊load_game
    ```rune
        let path = ctx["persistence_path"];
        let save_file = `${path}/save.toml`;
        
        // ファイルを読み込み
        let toml_str = read_text_file(save_file)?;
        
        // TOMLをRune値に変換
        let data = toml_from_string(toml_str)?;
        
        // データを使用
        let level = data["level"];
        let gold = data["gold"];
        
        yield emit_text(`レベル: ${level}, ゴールド: ${gold}`);
    ```
```

## ファイルI/O関数

### read_text_file(path)

指定されたパスのテキストファイルを読み込みます。

- **引数**: `path: String` - ファイルパス
- **戻り値**: `Result<String, String>` - ファイル内容またはエラーメッセージ
- **エラー**: ファイルが存在しない、読み取り権限がない等

```rune
let content = read_text_file("/path/to/file.txt")?;
```

### write_text_file(path, content)

指定されたパスにテキストファイルを書き込みます。

- **引数**: 
  - `path: String` - ファイルパス
  - `content: String` - 書き込む内容
- **戻り値**: `Result<(), String>` - 成功またはエラーメッセージ
- **エラー**: ディスク容量不足、書き込み権限がない等

```rune
write_text_file("/path/to/file.txt", "Hello, World!")?;
```

## 完全な例

### ゲーム進行状況の保存・読み込み

```rune
＊save_game
    ```rune
        let path = ctx["persistence_path"];
        
        if path == "" {
            yield emit_text("永続化は無効です");
            return;
        }
        
        let save_file = `${path}/game_progress.toml`;
        
        let progress = #{
            "chapter": 3,
            "completed_quests": ["quest1", "quest2"],
            "inventory": #{
                "potion": 5,
                "sword": 1
            }
        };
        
        let toml_str = toml_to_string(progress)?;
        write_text_file(save_file, toml_str)?;
        
        yield emit_text("ゲームを保存しました");
    ```

＊load_game
    ```rune
        let path = ctx["persistence_path"];
        
        if path == "" {
            yield emit_text("永続化は無効です");
            return;
        }
        
        let save_file = `${path}/game_progress.toml`;
        
        let toml_str = read_text_file(save_file)?;
        let progress = toml_from_string(toml_str)?;
        
        let chapter = progress["chapter"];
        yield emit_text(`チャプター ${chapter} を読み込みました`);
    ```
```

## セキュリティベストプラクティス

### パストラバーサル攻撃の防止

ユーザー入力を含むファイル名を使用する際は、パストラバーサル攻撃に注意が必要です。

#### 推奨: 固定ファイル名の使用

最も安全な方法は、ハードコードされたファイル名のみを使用することです。

```rune
// 安全: 固定ファイル名
let save_file = `${path}/save_data.toml`;
let config_file = `${path}/config.toml`;
```

#### ホワイトリスト検証

許可されたファイル名のみを受け入れます。

```rune
let allowed_files = ["save.toml", "config.toml", "progress.toml"];

fn is_allowed_file(filename) {
    for allowed in allowed_files {
        if filename == allowed {
            return true;
        }
    }
    false
}

if is_allowed_file(user_filename) {
    let file_path = `${path}/${user_filename}`;
    // ファイル操作を実行
} else {
    yield emit_error("無効なファイル名");
}
```

#### サニタイズ処理

ユーザー入力からパス区切り文字を除去します（最後の手段）。

```rune
fn sanitize_filename(filename) {
    filename
        .replace("/", "_")
        .replace("\\", "_")
        .replace("..", "_")
}

let safe_filename = sanitize_filename(user_input);
let file_path = `${path}/${safe_filename}.toml`;
```

### エラーハンドリング

ファイルI/O操作は失敗する可能性があるため、適切なエラーハンドリングが重要です。

```rune
＊load_with_fallback
    ```rune
        let path = ctx["persistence_path"];
        let save_file = `${path}/save.toml`;
        
        let data = match read_text_file(save_file) {
            Ok(content) => {
                match toml_from_string(content) {
                    Ok(parsed) => parsed,
                    Err(e) => {
                        yield emit_error(`TOML解析エラー: ${e}`);
                        // デフォルト値を使用
                        #{"level": 1, "gold": 0}
                    }
                }
            },
            Err(e) => {
                yield emit_text("セーブデータが見つかりません。新規ゲームを開始します");
                #{"level": 1, "gold": 0}
            }
        };
        
        yield emit_text(`レベル ${data["level"]} で開始します`);
    ```
```

または、`?`演算子を使用したシンプルなエラー伝播：

```rune
＊strict_load
    ```rune
        let path = ctx["persistence_path"];
        let save_file = `${path}/save.toml`;
        
        // エラー時は即座に伝播
        let toml_str = read_text_file(save_file)?;
        let data = toml_from_string(toml_str)?;
        
        yield emit_text("ロード成功");
    ```
```

## 永続化パスなしの処理

永続化パスが設定されていない場合の適切な処理：

```rune
＊handle_no_persistence
    ```rune
        let path = ctx["persistence_path"];
        
        if path == "" {
            yield emit_text("永続化機能は無効です");
            yield emit_text("ゲームは終了時に保存されません");
            return;
        }
        
        // 通常の永続化処理
        // ...
    ```
```

## トラブルシューティング

### ファイルが見つからない

```
Error: Failed to read file '/path/to/file.toml': No such file or directory
```

**解決策**: ファイルが存在することを確認するか、存在しない場合の処理を実装してください。

### TOML解析エラー

```
Error: TOML parsing failed: expected an equals, found a newline at line 2
```

**解決策**: TOML形式が正しいか確認してください。すべてのキーに値が設定されていることを確認してください。

### 書き込み権限エラー

```
Error: Failed to write file '/path/to/file.toml': Permission denied
```

**解決策**: アプリケーションが永続化ディレクトリへの書き込み権限を持っていることを確認してください。

## まとめ

- `ctx["persistence_path"]`で永続化ディレクトリパスを取得
- `toml_to_string()`でRune値をTOML文字列に変換
- `toml_from_string()`でTOML文字列をRune値に変換
- `read_text_file()`と`write_text_file()`でファイルI/O
- セキュリティのため固定ファイル名を使用
- 適切なエラーハンドリングを実装
- 永続化パスなしのケースに対応

本ガイドに従うことで、安全で堅牢な永続化機能をRuneスクリプトに実装できます。
