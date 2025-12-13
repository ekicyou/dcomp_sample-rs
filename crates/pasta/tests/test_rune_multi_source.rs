// Test: Rune multiple source imports

use rune::{Context, Sources};

#[test]
fn test_cross_source_import() {
    // Source 1: Define a constant
    let source1 = r#"
pub const MY_VALUE = 42;
"#;
    
    // Source 2: Import and use the constant
    let source2 = r#"
use source1::MY_VALUE;

pub fn test() {
    let x = MY_VALUE;
}
"#;
    
    let mut context = Context::with_default_modules().expect("Failed to create context");
    let mut sources = Sources::new();
    
    sources.insert(rune::Source::new("source1", source1).expect("Failed to create source1"))
        .expect("Failed to add source1");
    sources.insert(rune::Source::new("source2", source2).expect("Failed to create source2"))
        .expect("Failed to add source2");
    
    let mut diagnostics = rune::diagnostics::Diagnostics::new();
    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();
    
    if !diagnostics.is_empty() {
        let mut writer = rune::termcolor::Buffer::no_color();
        diagnostics.emit(&mut writer, &sources).expect("Failed to emit diagnostics");
        let output = String::from_utf8_lossy(writer.as_slice());
        println!("=== Diagnostics ===");
        println!("{}", output);
        println!("==================");
    }
    
    match result {
        Ok(_) => println!("✅ Cross-source import works!"),
        Err(e) => {
            println!("❌ Cross-source import failed: {:?}", e);
            panic!("Need to find another way");
        }
    }
}

#[test]
fn test_single_source_with_modules() {
    // All in one source
    let source = r#"
pub const MY_VALUE = 42;

pub mod sub {
    use super::MY_VALUE;
    
    pub fn test() {
        let x = MY_VALUE;
    }
}
"#;
    
    let mut context = Context::with_default_modules().expect("Failed to create context");
    let mut sources = Sources::new();
    
    sources.insert(rune::Source::new("main", source).expect("Failed to create source"))
        .expect("Failed to add source");
    
    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .build();
    
    match result {
        Ok(_) => println!("✅ Single source with super:: import works!"),
        Err(e) => {
            println!("❌ Failed: {:?}", e);
            panic!("Should work");
        }
    }
}

#[test]
fn test_combined_source() {
    // Combine main.rn and transpiled code into single source
    let main_part = r#"
pub const さくら = #{
    name: "さくら",
    id: "sakura",
};
"#;
    
    let transpiled_part = r#"
pub mod 会話_1 {
    use crate::さくら;
    
    pub fn __start__(ctx) {
        ctx.actor = さくら;
    }
}
"#;
    
    let combined = format!("{}\n{}", main_part, transpiled_part);
    
    println!("=== Combined Source ===");
    println!("{}", combined);
    println!("=======================");
    
    let mut context = Context::with_default_modules().expect("Failed to create context");
    let mut sources = Sources::new();
    
    sources.insert(rune::Source::new("main", &combined).expect("Failed to create source"))
        .expect("Failed to add source");
    
    let mut diagnostics = rune::diagnostics::Diagnostics::new();
    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();
    
    if !diagnostics.is_empty() {
        let mut writer = rune::termcolor::Buffer::no_color();
        diagnostics.emit(&mut writer, &sources).expect("Failed to emit diagnostics");
        let output = String::from_utf8_lossy(writer.as_slice());
        println!("=== Diagnostics ===");
        println!("{}", output);
        println!("==================");
    }
    
    match result {
        Ok(_) => println!("✅ Combined source with super:: import works!"),
        Err(e) => {
            println!("❌ Failed: {:?}", e);
            panic!("This is the solution we need");
        }
    }
}
