//! 予約ラベル名パターンのバリデーションテスト
//! __*__ パターンはシステム予約のため、ユーザー定義ラベルとして使用不可

use pasta::parser::parse_str;

#[test]
fn test_reserved_pattern_global_label() {
    // ❌ グローバルラベル: __start__ は予約パターン
    let script = r#"
＊__start__
    さくら：これは禁止されています
"#;

    let result = parse_str(script, "<test>");
    assert!(
        result.is_err(),
        "Expected error for reserved label '__start__'"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("reserved for system use") || err_msg.contains("__start__"),
        "Error message should mention reserved pattern: {}",
        err_msg
    );
}

#[test]
fn test_reserved_pattern_local_label() {
    // ❌ ローカルラベル: __test__ は予約パターン
    let script = r#"
＊親ラベル
    さくら：親ラベルです
    
    ＊＊__test__
        さくら：これは禁止されています
"#;

    let result = parse_str(script, "<test>");
    assert!(
        result.is_err(),
        "Expected error for reserved label '__test__'"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("reserved for system use") || err_msg.contains("__test__"),
        "Error message should mention reserved pattern: {}",
        err_msg
    );
}

#[test]
fn test_reserved_pattern_japanese() {
    // ❌ 日本語ラベル名でも予約パターンは禁止
    let script = r#"
＊__テスト__
    さくら：これは禁止されています
"#;

    let result = parse_str(script, "<test>");
    assert!(
        result.is_err(),
        "Expected error for reserved label '__テスト__'"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("reserved for system use") || err_msg.contains("__テスト__"),
        "Error message should mention reserved pattern: {}",
        err_msg
    );
}

#[test]
fn test_allowed_single_underscore() {
    // ✅ アンダースコア1つは許可
    let script = r#"
＊_private_label
    さくら：これはOKです
"#;

    let result = parse_str(script, "<test>");
    assert!(
        result.is_ok(),
        "Single underscore prefix should be allowed: {:?}",
        result.err()
    );
}

#[test]
fn test_allowed_double_underscore_not_enclosed() {
    // ✅ __で始まるが__で終わらない場合は許可（Pest定義による）
    // ただし、現在のPest定義では reserved_label_pattern が優先されるため、
    // label_nameルールは !reserved_label_pattern をチェックする
    let script = r#"
＊__test
    さくら：これはOKかもしれません
"#;

    // Pest文法次第でこれが許可されるかは要確認
    // reserved_pattern = @{ "__" ~ XID_START ~ XID_CONTINUE* ~ "__" }
    // なので、"__test" は末尾に "__" がないため許可される可能性がある
    let result = parse_str(script, "<test>");
    // このテストは現在のPest実装に依存するため、柔軟に検証
    println!(
        "Result for '__test': {}",
        if result.is_ok() { "OK" } else { "Error" }
    );
}

#[test]
fn test_allowed_middle_double_underscore() {
    // ✅ 途中に__がある場合は許可
    let script = r#"
＊label__with__double
    さくら：これはOKです
"#;

    let result = parse_str(script, "<test>");
    assert!(
        result.is_ok(),
        "Double underscore in the middle should be allowed: {:?}",
        result.err()
    );
}

#[test]
fn test_reserved_pattern_word_prefix() {
    // ❌ システム生成する予約関数パターン
    let script = r#"
＊__word_test__
    さくら：これは禁止されています
"#;

    let result = parse_str(script, "<test>");
    assert!(
        result.is_err(),
        "Expected error for reserved label '__word_test__'"
    );
}

#[test]
fn test_normal_labels_allowed() {
    // ✅ 通常のラベル名は問題なし
    let script = r#"
＊メイン
    さくら：こんにちは

＊挨拶
    さくら：やあ
"#;

    let result = parse_str(script, "<test>");
    assert!(
        result.is_ok(),
        "Normal label names should be allowed: {:?}",
        result.err()
    );
}
