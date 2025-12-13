// Test: Parse comprehensive_control_flow.pasta

use pasta::parser::parse_str;
use std::fs;

#[test]
fn test_parse_comprehensive_control_flow() {
    let pasta_code = fs::read_to_string("tests/fixtures/comprehensive_control_flow.pasta")
        .expect("Failed to read comprehensive_control_flow.pasta");
    
    println!("=== Attempting to parse comprehensive_control_flow.pasta ===");
    
    let result = parse_str(&pasta_code, "comprehensive_control_flow.pasta");
    
    match result {
        Ok(ast) => {
            println!("✅ Parse successful!");
            println!("Global labels: {}", ast.labels.len());
            for label in &ast.labels {
                println!("  - {} (local labels: {})", label.name, label.local_labels.len());
            }
        }
        Err(e) => {
            println!("❌ Parse failed:");
            println!("{}", e);
            panic!("Parse failed");
        }
    }
}
