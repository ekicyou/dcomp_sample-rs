// Quick test to transpile comprehensive_control_flow.pasta

use pasta::parser::parse_file;
use pasta::transpiler::Transpiler;
use std::path::Path;

fn main() {
    let pasta_path = Path::new("crates/pasta/tests/fixtures/comprehensive_control_flow.pasta");
    
    println!("Parsing {}...", pasta_path.display());
    let ast = match parse_file(pasta_path) {
        Ok(ast) => {
            println!("✅ Parse successful!");
            println!("Global labels: {}", ast.labels.len());
            println!("Global words: {}", ast.global_words.len());
            ast
        }
        Err(e) => {
            eprintln!("❌ Parse failed: {:?}", e);
            std::process::exit(1);
        }
    };
    
    println!("\nTranspiling...");
    match Transpiler::transpile_to_string(&ast) {
        Ok(rune_code) => {
            println!("✅ Transpile successful!");
            println!("\n=== Generated Rune Code ===\n");
            println!("{}", rune_code);
        }
        Err(e) => {
            eprintln!("❌ Transpile failed: {:?}", e);
            std::process::exit(1);
        }
    }
}
