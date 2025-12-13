// Test: Can the reference comprehensive_control_flow.rn compile?

use rune::{Context, Sources};
use std::fs;

#[test]
fn test_comprehensive_control_flow_rn_compiles() {
    // Read the reference .rn file
    let rune_code = fs::read_to_string("tests/fixtures/comprehensive_control_flow.rn")
        .expect("Failed to read comprehensive_control_flow.rn");

    println!("=== Reference Comprehensive Control Flow Rune Code ===");
    println!("{}", &rune_code[..std::cmp::min(500, rune_code.len())]);
    println!("... (truncated)");
    println!("===========================");

    // Create Rune context
    let mut context = Context::with_default_modules().expect("Failed to create context");

    // Install pasta_stdlib
    context
        .install(pasta::stdlib::create_module().expect("Failed to create stdlib"))
        .expect("Failed to install stdlib");

    // Add sources
    let mut sources = Sources::new();
    sources
        .insert(rune::Source::new("entry", &rune_code).expect("Failed to create source"))
        .expect("Failed to add source");

    // Compile
    let result = rune::prepare(&mut sources).with_context(&context).build();

    match result {
        Ok(_unit) => {
            println!("✅ Reference comprehensive_control_flow.rn compiles successfully!");
        }
        Err(e) => {
            println!("❌ Reference .rn file failed to compile!");
            println!("Error: {:?}", e);
            panic!("Reference .rn should compile but it doesn't");
        }
    }
}
