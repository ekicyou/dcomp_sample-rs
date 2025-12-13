// Test: Rune compilation of transpiled code

use pasta::parser::parse_str;
use pasta::transpiler::Transpiler;
use rune::{Context, Sources, Vm};

#[test]
fn test_rune_compile_simple() {
    // Simple pasta script
    let pasta_code = r#"
＊会話
　さくら：こんにちは
"#;

    // Parse
    let ast = parse_str(pasta_code, "test.pasta").expect("Failed to parse");
    
    // Transpile
    let rune_code = Transpiler::transpile_to_string(&ast).expect("Failed to transpile");
    
    println!("=== Generated Rune Code ===");
    println!("{}", rune_code);
    println!("===========================");
    
    // Create Rune context
    let mut context = Context::with_default_modules().expect("Failed to create context");
    
    // Install pasta_stdlib
    context.install(pasta::stdlib::create_module().expect("Failed to create stdlib"))
        .expect("Failed to install stdlib");
    
    // Add sources
    let mut sources = Sources::new();
    sources.insert(rune::Source::new("entry", &rune_code).expect("Failed to create source"))
        .expect("Failed to add source");
    
    // Compile
    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .build();
    
    match result {
        Ok(unit) => {
            println!("✅ Rune compilation succeeded!");
            
            // Try to get runtime and create VM
            let runtime = context.runtime().expect("Failed to get runtime");
            let mut vm = Vm::new(std::sync::Arc::new(runtime), std::sync::Arc::new(unit));
            
            // Create context object
            let mut ctx_obj = rune::runtime::Object::new();
            ctx_obj.insert(
                rune::alloc::String::try_from("actor").unwrap(),
                rune::to_value("").unwrap()
            ).unwrap();
            
            let ctx = rune::to_value(ctx_obj).expect("Failed to create context");
            
            // Try to call __start__
            println!("Attempting to call 会話_1::__start__...");
            let call_result = vm.call(["会話_1", "__start__"], (ctx,));
            
            match call_result {
                Ok(value) => {
                    println!("✅ Function call succeeded!");
                    println!("Return value: {:?}", value);
                }
                Err(e) => {
                    println!("❌ Function call failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Rune compilation failed!");
            println!("Error: {:?}", e);
            
            // Print error details
            eprintln!("Build error details: {:#?}", e);
            
            panic!("Rune compilation failed");
        }
    }
}
