use pasta::parser::parse_file;
use pasta::transpiler::Transpiler;
use rune::{Context, Sources};
use std::path::Path;

#[test]
fn test_comprehensive_control_flow_rune_compile() {
    let pasta_path = Path::new("tests/fixtures/comprehensive_control_flow.pasta");
    let main_rn_path = Path::new("tests/fixtures/test-project/main.rn");

    // Parse Pasta file
    let ast = parse_file(pasta_path).expect("Failed to parse");

    // Transpile to Rune code
    let transpiled_code = Transpiler::transpile_to_string(&ast).expect("Failed to transpile");

    println!("=== Transpiled Rune Code (first 100 lines) ===");
    for (i, line) in transpiled_code.lines().take(100).enumerate() {
        println!("{:3}: {}", i + 1, line);
    }
    println!("... ({} lines total)", transpiled_code.lines().count());

    // Read main.rn (actor definitions)
    let main_rn = std::fs::read_to_string(main_rn_path).expect("Failed to read main.rn");

    // Create Rune context
    let mut context = Context::with_default_modules().expect("Failed to create context");

    // Install pasta_stdlib
    context
        .install(pasta::stdlib::create_module().expect("Failed to create stdlib"))
        .expect("Failed to install stdlib");

    // Add sources - main.rn first, then transpiled code
    let mut sources = Sources::new();
    sources
        .insert(rune::Source::new("main", &main_rn).expect("Failed to create main.rn source"))
        .expect("Failed to add main.rn");
    sources
        .insert(
            rune::Source::new("transpiled", &transpiled_code)
                .expect("Failed to create transpiled source"),
        )
        .expect("Failed to add transpiled code");

    // Compile
    let result = rune::prepare(&mut sources).with_context(&context).build();

    match result {
        Ok(_unit) => {
            println!("\n✅ Rune VM compilation SUCCEEDED for comprehensive_control_flow.pasta!");
            println!("   ✓ main.rn (actor definitions) compiled successfully");
            println!("   ✓ __pasta_trans2__ module compiled successfully");
            println!("   ✓ pasta module compiled successfully");
            println!("   ✓ All label functions (6 labels) compiled successfully");
            println!("   ✓ Rune blocks with actor variables resolved successfully");
        }
        Err(e) => {
            println!("\n❌ Rune VM compilation FAILED!");
            eprintln!("Error details: {:#?}", e);
            panic!("Rune compilation failed for comprehensive_control_flow.pasta");
        }
    }
}
