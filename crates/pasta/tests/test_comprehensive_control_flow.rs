// Test: comprehensive_control_flow.pasta transpilation (P0 acceptance criteria)

use pasta::parser::parse_str;
use pasta::transpiler::Transpiler;
use std::fs;

#[test]
fn test_comprehensive_control_flow_transpile() {
    // Read comprehensive_control_flow.pasta
    let pasta_code = fs::read_to_string("tests/fixtures/comprehensive_control_flow.pasta")
        .expect("Failed to read comprehensive_control_flow.pasta");
    
    println!("=== Input: comprehensive_control_flow.pasta ===");
    println!("{}", &pasta_code[..std::cmp::min(500, pasta_code.len())]);
    println!("... (truncated)\n");
    
    // Parse
    let ast = parse_str(&pasta_code, "comprehensive_control_flow.pasta")
        .expect("Failed to parse comprehensive_control_flow.pasta");
    
    println!("✅ Parse successful!");
    
    // Transpile
    let transpiled = Transpiler::transpile_to_string(&ast)
        .expect("Failed to transpile comprehensive_control_flow.pasta");
    
    println!("\n=== Transpiled Output ===");
    println!("{}", transpiled);
    println!("========================\n");
    
    println!("✅ Transpilation successful!");
    
    // Basic validations
    assert!(transpiled.contains("pub mod メイン_1"), "Should have メイン_1 module");
    assert!(transpiled.contains("pub fn __start__(ctx)"), "Should have __start__ function");
    assert!(transpiled.contains("pub mod pasta"), "Should have pasta module");
    assert!(transpiled.contains("pub fn jump"), "Should have jump function");
    assert!(transpiled.contains("pub fn call"), "Should have call function");
    assert!(transpiled.contains("pub fn label_selector"), "Should have label_selector function");
    
    // Check for control flow statements
    assert!(transpiled.contains("pasta::call"), "Should contain call statements");
    assert!(transpiled.contains("pasta::jump"), "Should contain jump statements");
    
    println!("✅ All basic validations passed!");
}

#[test]
fn test_comprehensive_control_flow_has_local_labels() {
    let pasta_code = fs::read_to_string("tests/fixtures/comprehensive_control_flow.pasta")
        .expect("Failed to read comprehensive_control_flow.pasta");
    
    let ast = parse_str(&pasta_code, "comprehensive_control_flow.pasta")
        .expect("Failed to parse");
    
    let transpiled = Transpiler::transpile_to_string(&ast)
        .expect("Failed to transpile");
    
    // Check for local labels (they should be functions within the module)
    assert!(transpiled.contains("pub fn 自己紹介"), "Should have 自己紹介 local label");
    assert!(transpiled.contains("pub fn 趣味紹介"), "Should have 趣味紹介 local label");
    assert!(transpiled.contains("pub fn カウント表示"), "Should have カウント表示 local label");
    assert!(transpiled.contains("pub fn 会話分岐"), "Should have 会話分岐 local label");
    assert!(transpiled.contains("pub fn 別の話題"), "Should have 別の話題 local label");
    
    println!("✅ All local labels present!");
}

#[test]
fn test_comprehensive_control_flow_actor_extraction() {
    let pasta_code = fs::read_to_string("tests/fixtures/comprehensive_control_flow.pasta")
        .expect("Failed to read comprehensive_control_flow.pasta");
    
    let ast = parse_str(&pasta_code, "comprehensive_control_flow.pasta")
        .expect("Failed to parse");
    
    let transpiled = Transpiler::transpile_to_string(&ast)
        .expect("Failed to transpile");
    
    // Check for dynamic actor imports
    assert!(transpiled.contains("use crate::"), "Should have crate imports");
    assert!(transpiled.contains("さくら"), "Should import さくら");
    assert!(transpiled.contains("うにゅう"), "Should import うにゅう");
    
    // Check for actor assignments
    assert!(transpiled.contains("ctx.actor = さくら"), "Should have さくら actor assignment");
    assert!(transpiled.contains("ctx.actor = うにゅう"), "Should have うにゅう actor assignment");
    
    println!("✅ Actor extraction works!");
}
