//! Test parsing partial file

use pasta::parser::parse_file;
use std::path::Path;

#[test]
fn test_parse_partial() {
    let pasta_path = Path::new("tests/fixtures/test_partial.pasta");
    
    println!("Parsing {}...", pasta_path.display());
    match parse_file(pasta_path) {
        Ok(ast) => {
            println!("✅ Parse successful!");
            println!("Global labels: {}", ast.labels.len());
            println!("Global words: {}", ast.global_words.len());
            
            for label in &ast.labels {
                println!("\nLabel: {}", label.name);
                println!("  Statements: {}", label.statements.len());
                println!("  Local labels: {}", label.local_labels.len());
            }
        }
        Err(e) => {
            panic!("❌ Parse failed: {:?}", e);
        }
    }
}
