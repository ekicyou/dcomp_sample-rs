// Test: Parse variable assignment

use pasta::parser::parse_str;

#[test]
fn test_parse_var_assign_in_label() {
    let pasta_code = "＊テスト\n　＄変数＝１０\n";
    
    println!("=== Parsing variable assignment in label ===");
    println!("Input: {:?}", pasta_code);
    
    let result = parse_str(pasta_code, "test.pasta");
    
    match &result {
        Ok(ast) => {
            println!("✅ Parse successful!");
            println!("Labels: {}", ast.labels.len());
            if !ast.labels.is_empty() {
                println!("Label: {}", ast.labels[0].name);
                println!("Statements: {}", ast.labels[0].statements.len());
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
    assert_eq!(ast.labels[0].statements.len(), 1);
}
