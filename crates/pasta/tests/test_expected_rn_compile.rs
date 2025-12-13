// Test: Can the expected .rn file compile?

use rune::{Context, Sources, Vm};
use std::fs;

#[test]
fn test_expected_rn_compiles() {
    // Read the expected .rn file
    let rune_code = fs::read_to_string("tests/fixtures/comprehensive_control_flow_simple.expected.rn")
        .expect("Failed to read expected.rn");
    
    println!("=== Expected Rune Code ===");
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
        Ok(_unit) => {
            println!("✅ Expected .rn file compiles successfully!");
        }
        Err(e) => {
            println!("❌ Expected .rn file failed to compile!");
            println!("Error: {:?}", e);
            panic!("Expected .rn should compile but it doesn't");
        }
    }
}
