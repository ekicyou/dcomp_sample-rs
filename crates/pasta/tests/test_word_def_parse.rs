// Test: Parse word definitions

use pasta::parser::parse_str;

#[test]
fn test_parse_simple_word_def() {
    let pasta_code = "＠挨拶：こんにちは　やあ　ハロー\n";
    
    println!("=== Parsing simple word definition ===");
    println!("Input: {:?}", pasta_code);
    
    let result = parse_str(pasta_code, "test.pasta");
    
    match &result {
        Ok(ast) => {
            println!("✅ Parse successful!");
            println!("Global words: {}", ast.global_words.len());
            for word in &ast.global_words {
                println!("  - {} = {:?}", word.name, word.values);
            }
        }
        Err(e) => {
            println!("❌ Parse failed:");
            println!("{}", e);
        }
    }
    
    assert!(result.is_ok(), "Parse should succeed");
    let ast = result.unwrap();
    assert_eq!(ast.global_words.len(), 1);
    assert_eq!(ast.global_words[0].name, "挨拶");
    assert_eq!(ast.global_words[0].values, vec!["こんにちは", "やあ", "ハロー"]);
}

#[test]
fn test_parse_local_word_def() {
    // Test case 1: Label with local word def only (no comment)
    let pasta_code = "＊メイン\n　＠場所：東京　大阪　京都\n";
    
    println!("=== Parsing local word definition ===");
    
    let result = parse_str(pasta_code, "test.pasta");
    
    match &result {
        Ok(ast) => {
            println!("✅ Parse successful!");
            println!("Labels: {}", ast.labels.len());
            if !ast.labels.is_empty() {
                println!("Label: {}", ast.labels[0].name);
                println!("Local words: {}", ast.labels[0].local_words.len());
                for word in &ast.labels[0].local_words {
                    println!("  - {} = {:?}", word.name, word.values);
                }
            }
        }
        Err(e) => {
            println!("❌ Parse failed:");
            println!("{}", e);
        }
    }
    
    assert!(result.is_ok(), "Parse should succeed");
    let ast = result.unwrap();
    assert_eq!(ast.labels.len(), 1);
    println!("Local words count: {}", ast.labels[0].local_words.len());
    assert_eq!(ast.labels[0].local_words.len(), 1);
    assert_eq!(ast.labels[0].local_words[0].name, "場所");
    assert_eq!(ast.labels[0].local_words[0].values, vec!["東京", "大阪", "京都"]);
}
