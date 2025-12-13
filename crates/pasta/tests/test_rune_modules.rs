// Test: Rune module structure

use rune::{Context, Sources};

#[test]
fn test_rune_module_simple() {
    let rune_code = r#"
pub mod test_1 {
    pub fn __start__(ctx) {
        let x = "hello";
    }
}
"#;

    println!("=== Simple Rune Module ===");
    println!("{}", rune_code);
    println!("===========================");

    let mut context = Context::with_default_modules().expect("Failed to create context");

    let mut sources = Sources::new();
    sources
        .insert(rune::Source::new("test", rune_code).expect("Failed to create source"))
        .expect("Failed to add source");

    let result = rune::prepare(&mut sources).with_context(&context).build();

    match result {
        Ok(_) => println!("✅ Simple module compiles!"),
        Err(e) => {
            println!("❌ Failed: {:?}", e);
            panic!("Simple module should compile");
        }
    }
}

#[test]
fn test_rune_module_with_use() {
    let rune_code = r#"
use pasta_stdlib::*;

pub mod test_1 {
    pub fn __start__(ctx) {
        let x = "hello";
    }
}
"#;

    println!("=== Module with use ===");
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
        Ok(_) => println!("✅ Module with use compiles!"),
        Err(e) => {
            println!("❌ Failed: {:?}", e);
            panic!("Module with use should compile");
        }
    }
}

#[test]
fn test_rune_yield_outside_module() {
    let rune_code = r#"
use pasta_stdlib::*;

pub fn test() {
    yield Talk("hello");
}
"#;

    println!("=== Yield outside module ===");
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
        Ok(_) => println!("✅ Yield outside module compiles!"),
        Err(e) => {
            println!("❌ Failed: {:?}", e);
            panic!("Yield outside module should compile");
        }
    }
}

#[test]
fn test_rune_module_with_yield() {
    let rune_code = r#"
use pasta_stdlib::*;

pub mod test_1 {
    pub fn __start__(ctx) {
        yield Talk("hello");
    }
}
"#;

    println!("=== Module with yield ===");
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
        Ok(_) => println!("✅ Module with yield compiles!"),
        Err(e) => {
            println!("❌ Failed: {:?}", e);
            panic!("Module with yield should compile");
        }
    }
}

#[test]
fn test_rune_module_with_ctx_access() {
    let rune_code = r#"
use pasta_stdlib::*;

pub mod test_1 {
    pub fn __start__(ctx) {
        ctx.actor = "さくら";
        yield Actor("さくら");
        yield Talk("こんにちは");
    }
}
"#;

    println!("=== Rune Module Code ===");
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
        Ok(_) => println!("✅ Rune module compiles!"),
        Err(e) => {
            println!("❌ Failed: {:?}", e);
            panic!("Rune module should compile");
        }
    }
}
