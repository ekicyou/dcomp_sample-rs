use rune::Context;
use std::fs;

fn main() {
    let code = fs::read_to_string("crates/pasta/debug_combined.rn").expect("Failed to read file");
    
    let mut context = Context::with_default_modules().expect("Failed to create context");
    
    // Install pasta_stdlib
    let pasta_module = pasta::stdlib::create_module().expect("Failed to create module");
    context.install(pasta_module).expect("Failed to install module");
    
    let mut sources = rune::Sources::new();
    sources.insert(rune::Source::new("test", &code).expect("Failed to create source"))
        .expect("Failed to insert source");
    
    match rune::prepare(&mut sources).with_context(&context).build() {
        Ok(_) => println!("✅ Compilation successful!"),
        Err(e) => {
            println!("❌ Compilation failed!");
            println!("Error: {}", e);
        }
    }
}
