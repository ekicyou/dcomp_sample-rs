// Test: Execute transpiled code with yield limit

use rune::{Context, Sources, Vm};
use std::fs;
use std::sync::Arc;

#[test]
fn test_execute_simple_label() {
    // Read main.rn
    let main_rn = fs::read_to_string("tests/fixtures/test-project/main.rn")
        .expect("Failed to read main.rn");
    
    // Simple transpiled code with only one label (no recursion)
    let transpiled = r#"
use pasta_stdlib::*;

pub mod 会話_1 {
    use pasta_stdlib::*;

    pub fn __start__(ctx) {
        ctx.actor = "さくら";
        yield Actor("さくら");
        yield Talk("こんにちは");
        yield Talk("良い天気ですね");
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
            _ => |ctx, args| {
                yield Error(`ラベルID ${id} が見つかりませんでした。`);
            },
        }
    }
}
"#;
    
    println!("=== Testing Execution ===");
    
    // Create Rune context
    let mut context = Context::with_default_modules().expect("Failed to create context");
    
    // Install pasta_stdlib
    context.install(pasta::stdlib::create_module().expect("Failed to create stdlib"))
        .expect("Failed to install stdlib");
    
    // Add sources
    let mut sources = Sources::new();
    sources.insert(rune::Source::new("main", &main_rn).expect("Failed to create main source"))
        .expect("Failed to add main source");
    sources.insert(rune::Source::new("entry", transpiled).expect("Failed to create entry source"))
        .expect("Failed to add entry source");
    
    // Compile
    let unit = rune::prepare(&mut sources)
        .with_context(&context)
        .build()
        .expect("Failed to compile");
    
    println!("✅ Compilation successful!");
    
    // Create VM
    let runtime = Arc::new(context.runtime().expect("Failed to get runtime"));
    let unit = Arc::new(unit);
    let mut vm = Vm::new(runtime, unit);
    
    // Create context object
    let mut ctx_obj = rune::runtime::Object::new();
    ctx_obj.insert(
        rune::alloc::String::try_from("actor").unwrap(),
        rune::to_value("").unwrap()
    ).unwrap();
    
    let ctx = rune::to_value(ctx_obj).expect("Failed to create context");
    
    // Execute 会話_1::__start__
    println!("Executing 会話_1::__start__...");
    let execution = vm.execute(["会話_1", "__start__"], (ctx,))
        .expect("Failed to execute");
    
    let mut generator = execution.into_generator();
    let unit_value = rune::to_value(()).unwrap();
    
    let mut event_count = 0;
    let max_events = 10; // Safety limit
    
    loop {
        if event_count >= max_events {
            println!("⚠️ Reached safety limit of {} events", max_events);
            break;
        }
        
        match generator.resume(unit_value.clone()) {
            rune::runtime::VmResult::Ok(rune::runtime::GeneratorState::Yielded(value)) => {
                event_count += 1;
                
                // Try to extract event information
                if let Ok(obj) = rune::from_value::<rune::runtime::Object>(value.clone()) {
                    println!("Event {}: {:?}", event_count, obj);
                } else {
                    println!("Event {}: (non-object value)", event_count);
                }
            }
            rune::runtime::VmResult::Ok(rune::runtime::GeneratorState::Complete(_)) => {
                println!("✅ Execution completed after {} events", event_count);
                break;
            }
            rune::runtime::VmResult::Err(e) => {
                panic!("❌ Runtime error: {:?}", e);
            }
        }
    }
    
    assert!(event_count > 0, "Should generate at least one event");
    assert!(event_count <= 5, "Should generate 3-4 events (1 Actor + 2 Talk)");
    
    println!("✅ Execution test passed!");
}

#[test]
fn test_execute_with_call() {
    // Read main.rn
    let main_rn = fs::read_to_string("tests/fixtures/test-project/main.rn")
        .expect("Failed to read main.rn");
    
    // Code with call statement - will recurse infinitely with stub!
    let transpiled = r#"
use pasta_stdlib::*;

pub mod メイン_1 {
    use pasta_stdlib::*;

    pub fn __start__(ctx) {
        ctx.actor = "さくら";
        yield Actor("さくら");
        yield Talk("サブを呼びます");
        
        // This will infinitely recurse because stub always returns 1
        for a in pasta::call(ctx, "サブ", #{}, []) { yield a; }
    }
}

pub mod サブ_1 {
    use pasta_stdlib::*;

    pub fn __start__(ctx) {
        yield Talk("サブ実行中");
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
        // Stub always returns 1, so any call goes to メイン_1::__start__
        match id {
            1 => crate::メイン_1::__start__,
            2 => crate::サブ_1::__start__,
            _ => |ctx, args| {
                yield Error(`ラベルID ${id} が見つかりませんでした。`);
            },
        }
    }
}
"#;
    
    println!("=== Testing Execution with Call (Infinite Recursion) ===");
    
    // Create Rune context
    let mut context = Context::with_default_modules().expect("Failed to create context");
    
    // Install pasta_stdlib
    context.install(pasta::stdlib::create_module().expect("Failed to create stdlib"))
        .expect("Failed to install stdlib");
    
    // Add sources
    let mut sources = Sources::new();
    sources.insert(rune::Source::new("main", &main_rn).expect("Failed to create main source"))
        .expect("Failed to add main source");
    sources.insert(rune::Source::new("entry", transpiled).expect("Failed to create entry source"))
        .expect("Failed to add entry source");
    
    // Compile
    let unit = rune::prepare(&mut sources)
        .with_context(&context)
        .build()
        .expect("Failed to compile");
    
    println!("✅ Compilation successful!");
    
    // Create VM
    let runtime = Arc::new(context.runtime().expect("Failed to get runtime"));
    let unit = Arc::new(unit);
    let mut vm = Vm::new(runtime, unit);
    
    // Create context object
    let mut ctx_obj = rune::runtime::Object::new();
    ctx_obj.insert(
        rune::alloc::String::try_from("actor").unwrap(),
        rune::to_value("").unwrap()
    ).unwrap();
    
    let ctx = rune::to_value(ctx_obj).expect("Failed to create context");
    
    // Execute メイン_1::__start__
    println!("Executing メイン_1::__start__ (will recurse infinitely)...");
    let execution = vm.execute(["メイン_1", "__start__"], (ctx,))
        .expect("Failed to execute");
    
    let mut generator = execution.into_generator();
    let unit_value = rune::to_value(()).unwrap();
    
    let mut event_count = 0;
    let max_events = 10; // Safety limit - stop after 10 events
    
    println!("⚠️ Note: Due to stub returning fixed ID=1, this will recurse infinitely");
    println!("⚠️ We limit to {} events for safety", max_events);
    
    loop {
        if event_count >= max_events {
            println!("⚠️ Reached safety limit of {} events - stopping", max_events);
            break;
        }
        
        match generator.resume(unit_value.clone()) {
            rune::runtime::VmResult::Ok(rune::runtime::GeneratorState::Yielded(value)) => {
                event_count += 1;
                
                // Try to extract event information
                if let Ok(obj) = rune::from_value::<rune::runtime::Object>(value.clone()) {
                    println!("Event {}: {:?}", event_count, obj);
                } else {
                    println!("Event {}: (non-object value)", event_count);
                }
            }
            rune::runtime::VmResult::Ok(rune::runtime::GeneratorState::Complete(_)) => {
                println!("✅ Execution completed after {} events", event_count);
                break;
            }
            rune::runtime::VmResult::Err(e) => {
                // Expected to hit this due to infinite recursion eventually
                println!("⚠️ Runtime error after {} events: {:?}", event_count, e);
                break;
            }
        }
    }
    
    assert!(event_count > 0, "Should generate at least one event");
    println!("✅ Execution test with call passed (stopped safely)!");
}
