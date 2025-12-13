// Test: Dynamic actor extraction

use pasta::parser::parse_str;
use pasta::transpiler::Transpiler;

#[test]
fn test_dynamic_actor_extraction_single() {
    let pasta_code = r#"
＊会話
　さくら：こんにちは
"#;

    let ast = parse_str(pasta_code, "test.pasta").expect("Failed to parse");
    let transpiled = Transpiler::transpile_to_string(&ast).expect("Failed to transpile");

    println!("=== Transpiled (single actor) ===");
    println!("{}", transpiled);

    // Should only import さくら
    assert!(
        transpiled.contains("use crate::{さくら};"),
        "Should import only さくら"
    );
    assert!(
        !transpiled.contains("うにゅう"),
        "Should not import うにゅう"
    );
}

#[test]
fn test_dynamic_actor_extraction_multiple() {
    let pasta_code = r#"
＊会話
　さくら：こんにちは
　うにゅう：はぅ〜
　みつこ：よろしくね
"#;

    let ast = parse_str(pasta_code, "test.pasta").expect("Failed to parse");
    let transpiled = Transpiler::transpile_to_string(&ast).expect("Failed to transpile");

    println!("=== Transpiled (multiple actors) ===");
    println!("{}", transpiled);

    // Should import all three actors
    assert!(
        transpiled.contains("use crate::{"),
        "Should have use crate import"
    );
    assert!(transpiled.contains("さくら"), "Should import さくら");
    assert!(transpiled.contains("うにゅう"), "Should import うにゅう");
    assert!(transpiled.contains("みつこ"), "Should import みつこ (NEW!)");
}

#[test]
fn test_dynamic_actor_extraction_across_labels() {
    let pasta_code = r#"
＊会話1
　さくら：こんにちは

＊会話2
　うにゅう：はぅ〜

＊会話3
　みつこ：よろしくね
　ななこ：どうも
"#;

    let ast = parse_str(pasta_code, "test.pasta").expect("Failed to parse");
    let transpiled = Transpiler::transpile_to_string(&ast).expect("Failed to transpile");

    println!("=== Transpiled (across labels) ===");
    println!("{}", transpiled);

    // Should import all actors from all labels
    assert!(
        transpiled.contains("さくら"),
        "Should import さくら from 会話1"
    );
    assert!(
        transpiled.contains("うにゅう"),
        "Should import うにゅう from 会話2"
    );
    assert!(
        transpiled.contains("みつこ"),
        "Should import みつこ from 会話3"
    );
    assert!(
        transpiled.contains("ななこ"),
        "Should import ななこ from 会話3"
    );
}

#[test]
fn test_dynamic_actor_no_duplicates() {
    let pasta_code = r#"
＊会話1
　さくら：こんにちは
　さくら：良い天気ですね
　さくら：元気ですか？

＊会話2
　さくら：また会いましたね
"#;

    let ast = parse_str(pasta_code, "test.pasta").expect("Failed to parse");
    let transpiled = Transpiler::transpile_to_string(&ast).expect("Failed to transpile");

    println!("=== Transpiled (no duplicates) ===");
    println!("{}", transpiled);

    // Should only have one さくら import
    let import_count = transpiled.matches("use crate::{さくら};").count();
    assert_eq!(
        import_count, 2,
        "Should have exactly 2 modules with さくら import (会話1_1 and 会話2_1)"
    );

    // Count さくら in the import lines only
    let lines: Vec<&str> = transpiled.lines().collect();
    let import_lines: Vec<&str> = lines
        .iter()
        .filter(|line| line.contains("use crate::"))
        .copied()
        .collect();

    for line in import_lines {
        // Should not have さくら multiple times in the same import
        assert_eq!(
            line.matches("さくら").count(),
            1,
            "Should have さくら only once per import line"
        );
    }
}
