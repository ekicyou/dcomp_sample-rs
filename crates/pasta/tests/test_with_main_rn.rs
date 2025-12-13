// Test: Compile transpiled code with main.rn

use rune::{Context, Sources};
use std::fs;

#[test]
fn test_transpiled_with_main_rn() {
    // Read main.rn
    let main_rn = fs::read_to_string("tests/fixtures/test-project/main.rn")
        .expect("Failed to read main.rn");
    
    // Read transpiled code
    let transpiled = r#"
use pasta_stdlib::*;

pub mod 会話_1 {
    use pasta_stdlib::*;

    pub fn __start__(ctx) {
        ctx.actor = "さくら";
        yield Actor("さくら");
        yield Talk("おはよう！");
    }
}

pub mod 別会話_1 {
    use pasta_stdlib::*;

    pub fn __start__(ctx) {
        ctx.actor = "さくら";
        yield Actor("さくら");
        yield Talk("別の会話です。");
    }
}

pub mod pasta {
    use pasta_stdlib::*;

    pub fn jump(ctx, label, filters, args) {
        let label_fn = label_selector(label, filters);
        for a in label_fn(ctx, args) { yield a; }
    }

    pub fn call(ctx, label, filters, args) {
        let label_fn = label_selector(label, filters);
        for a in label_fn(ctx, args) { yield a; }
    }

    pub fn label_selector(label, filters) {
        let id = pasta_stdlib::select_label_to_id(label, filters);
        match id {
            1 => crate::会話_1::__start__,
            2 => crate::別会話_1::__start__,
            _ => |ctx, args| {
                yield Error(`ラベルID ${id} が見つかりませんでした。`);
            },
        }
    }
}
"#;
    
    println!("=== main.rn ===");
    println!("{}", main_rn);
    println!("=== Transpiled Code ===");
    println!("{}", transpiled);
    println!("===========================");
    
    // Create Rune context
    let mut context = Context::with_default_modules().expect("Failed to create context");
    
    // Install pasta_stdlib
    context.install(pasta::stdlib::create_module().expect("Failed to create stdlib"))
        .expect("Failed to install stdlib");
    
    // Add sources
    let mut sources = Sources::new();
    
    // Add main.rn first
    sources.insert(rune::Source::new("main", &main_rn).expect("Failed to create main source"))
        .expect("Failed to add main source");
    
    // Add transpiled code
    sources.insert(rune::Source::new("entry", transpiled).expect("Failed to create entry source"))
        .expect("Failed to add entry source");
    
    // Compile with diagnostics
    let mut diagnostics = rune::diagnostics::Diagnostics::new();
    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();
    
    // Print diagnostics
    if !diagnostics.is_empty() {
        let mut writer = rune::termcolor::Buffer::no_color();
        diagnostics.emit(&mut writer, &sources).expect("Failed to emit diagnostics");
        let output = String::from_utf8_lossy(writer.as_slice());
        println!("=== Diagnostics ===");
        println!("{}", output);
        println!("==================");
    }
    
    match result {
        Ok(_unit) => {
            println!("✅ Transpiled code with main.rn compiles successfully!");
        }
        Err(e) => {
            println!("❌ Compilation failed!");
            println!("Error: {:?}", e);
            panic!("Should compile with main.rn");
        }
    }
}
