use pasta::{parse_str, transpiler::{LabelRegistry, Transpiler}};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pasta_code = r#"
＊会話
　さくら：こんにちは

＊別会話
　うにゅう：やあ
"#;

    println!("Input pasta code:");
    println!("{}", pasta_code);
    println!("First char of line 2: {:?}", pasta_code.lines().nth(1));

    let ast = parse_str(pasta_code, "test.pasta")?;
    
    println!("AST has {} labels", ast.labels.len());
    for label in &ast.labels {
        println!("  Label: {}", label.name);
    }
    
    // Pass 1
    let mut registry = LabelRegistry::new();
    let mut buffer = Vec::new();
    Transpiler::transpile_pass1(&ast, &mut registry, &mut buffer)?;
    let mut output = String::from_utf8(buffer)?;
    
    println!("After Pass 1, registry has {} labels", registry.all_labels().len());
    for label in registry.all_labels() {
        println!("  Label {}: {} -> {}", label.id, label.name, label.fn_path);
    }
    
    // Pass 2
    let mut buffer = Vec::new();
    Transpiler::transpile_pass2(&registry, &mut buffer)?;
    let pass2_output = String::from_utf8(buffer)?;
    
    output.push_str(&pass2_output);
    
    // Write to file
    fs::write("C:\\home\\maz\\git\\dcomp_sample-rs-2\\test_output_debug.rn", &output)?;
    
    println!("Output written to test_output_debug.rn");
    println!("Checking for pattern: '1 => crate::会話_1::__start__'");
    println!("Contains pattern: {}", output.contains("1 => crate::会話_1::__start__"));
    
    // Also check what we actually have for ID 1
    for line in output.lines() {
        if line.trim().starts_with("1 =>") {
            println!("Found line starting with '1 =>': {}", line);
        }
    }
    
    Ok(())
}
