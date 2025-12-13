// Test: Minimal Rune code compilation

use rune::{Context, Sources};

#[test]
fn test_minimal_rune_compiles() {
    let rune_code = r#"
pub fn test() {
    let x = "hello";
}
"#;

    println!("=== Minimal Rune Code ===");
    println!("{}", rune_code);
    println!("===========================");

    let mut context = Context::with_default_modules().expect("Failed to create context");
    let mut sources = Sources::new();
    sources
        .insert(rune::Source::new("test", rune_code).expect("Failed to create source"))
        .expect("Failed to add source");

    let result = rune::prepare(&mut sources).with_context(&context).build();

    match result {
        Ok(_) => println!("✅ Minimal Rune code compiles!"),
        Err(e) => {
            println!("❌ Failed: {:?}", e);
            panic!("Minimal Rune should compile");
        }
    }
}

#[test]
fn test_rune_with_pasta_stdlib() {
    let rune_code = r#"
use pasta_stdlib::*;

pub fn test() {
    yield Talk("hello");
}
"#;

    println!("=== Rune with pasta_stdlib ===");
    println!("{}", rune_code);
    println!("===========================");

    let mut context = Context::with_default_modules().expect("Failed to create context");
    context
        .install(pasta::stdlib::create_module().expect("Failed to create stdlib"))
        .expect("Failed to install stdlib");

    let mut sources = Sources::new();
    sources
        .insert(rune::Source::new("test", rune_code).expect("Failed to create source"))
        .expect("Failed to add source");

    let result = rune::prepare(&mut sources).with_context(&context).build();

    match result {
        Ok(_) => println!("✅ Rune with pasta_stdlib compiles!"),
        Err(e) => {
            println!("❌ Failed: {:?}", e);
            panic!("Rune with pasta_stdlib should compile");
        }
    }
}
