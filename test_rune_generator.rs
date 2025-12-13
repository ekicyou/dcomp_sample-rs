// Runeのジェネレーター関数のパラメータテスト
use rune::{Context, Diagnostics, Source, Sources};

fn main() {
    let code = r#"
pub fn test_gen(ctx, label) {
    yield "test";
    for a in inner(ctx) { yield a; }
}

pub fn inner(ctx) {
    yield "inner";
}

pub fn main() {
    for x in test_gen(#{}, "label") {
        println(`{x}`);
    }
}
"#;

    let context = Context::new();
    let mut sources = Sources::new();
    sources.insert(Source::new("test", code).unwrap()).unwrap();
    
    let mut diagnostics = Diagnostics::new();
    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();
    
    if !diagnostics.is_empty() {
        eprintln!("Diagnostics:");
        for diag in diagnostics.diagnostics() {
            eprintln!("{:?}", diag);
        }
    }
    
    if let Ok(unit) = result {
        println!("Compilation successful!");
        
        let mut vm = rune::Vm::new(std::sync::Arc::new(context), std::sync::Arc::new(unit));
        let result = vm.call(rune::Hash::type_hash(["main"]), ());
        match result {
            Ok(value) => println!("Execution result: {:?}", value),
            Err(e) => eprintln!("Execution error: {:?}", e),
        }
    } else {
        eprintln!("Compilation failed");
    }
}
