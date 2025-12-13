// End-to-end test: Parse -> Transpile -> Compile -> Execute

use pasta::parser::parse_str;
use pasta::transpiler::Transpiler;
use rune::{Context, Sources, Vm};

#[test]
fn test_simple_end_to_end() {
    // Simple pasta script
    let pasta_code = r#"
＊会話
　さくら：こんにちは
"#;

    // Parse
    let ast = parse_str(pasta_code, "test.pasta").expect("Failed to parse");

    // Transpile (two-pass)
    let rune_code = Transpiler::transpile_to_string(&ast).expect("Failed to transpile");

    println!("=== Generated Rune Code ===");
    println!("{}", rune_code);
    println!("===========================");

    // Compile
    let mut context = Context::with_default_modules().expect("Failed to create context");
    context
        .install(pasta::stdlib::create_module().expect("Failed to create stdlib"))
        .expect("Failed to install stdlib");

    let mut sources = Sources::new();
    sources
        .insert(rune::Source::new("entry", &rune_code).expect("Failed to create source"))
        .expect("Failed to add source");

    let unit = match rune::prepare(&mut sources).with_context(&context).build() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("=== Rune Build Error ===");
            eprintln!("{:?}", e);
            panic!("Failed to compile Rune code: {:?}", e);
        }
    };

    println!("Rune compilation succeeded!");

    // Execute
    let mut vm = Vm::new(
        std::sync::Arc::new(context.runtime().expect("Failed to get runtime")),
        std::sync::Arc::new(unit),
    );

    // Create a simple context object
    let ctx = rune::to_value(rune::runtime::Object::new()).expect("Failed to create context");

    // Call __start__ function
    let result = vm.call(["会話_1", "__start__"], (ctx,));

    match result {
        Ok(value) => {
            println!("Execution succeeded: {:?}", value);
        }
        Err(e) => {
            panic!("Execution failed: {:?}", e);
        }
    }
}

#[test]
#[ignore] // Ignore for now, need to implement generator support
fn test_simple_generator_execution() {
    let pasta_code = r#"
＊会話
　さくら：こんにちは
"#;

    let ast = parse_str(pasta_code, "test.pasta").expect("Failed to parse");
    let rune_code = Transpiler::transpile_to_string(&ast).expect("Failed to transpile");

    let mut context = Context::with_default_modules().expect("Failed to create context");
    context
        .install(pasta::stdlib::create_module().expect("Failed to create stdlib"))
        .expect("Failed to install stdlib");

    let mut sources = Sources::new();
    sources
        .insert(rune::Source::new("entry", &rune_code).expect("Failed to create source"))
        .expect("Failed to add source");

    let unit = rune::prepare(&mut sources)
        .with_context(&context)
        .build()
        .expect("Failed to compile Rune code");

    let mut vm = Vm::new(
        std::sync::Arc::new(context.runtime().expect("Failed to get runtime")),
        std::sync::Arc::new(unit),
    );

    // Create context with required fields
    let mut ctx_obj = rune::runtime::Object::new();
    ctx_obj
        .insert(
            rune::alloc::String::try_from("actor").unwrap(),
            rune::to_value("").unwrap(),
        )
        .unwrap();
    let ctx = rune::to_value(ctx_obj).expect("Failed to create context");

    // Execute as generator - vm.call returns Value, need to convert
    let value = vm
        .call(["会話_1", "__start__"], (ctx,))
        .expect("Failed to call");
    let generator = rune::from_value::<rune::runtime::Generator<_>>(value)
        .expect("Failed to convert to generator");

    // Iterate through events
    let mut events = Vec::new();
    for event in generator {
        println!("Event: {:?}", event);
        events.push(event);
    }

    assert!(!events.is_empty(), "Generator should yield events");
}
