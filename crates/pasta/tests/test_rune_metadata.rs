//! RuneのUnitからメタデータ（関数一覧）を取得できるかのテスト

use rune::{Context, Source, Sources};

#[test]
fn test_inspect_unit_functions() -> Result<(), Box<dyn std::error::Error>> {
    let context = Context::with_default_modules()?;

    let rune_code = r#"
        pub mod test_mod {
            pub fn function_a() {
                yield "a";
            }
            
            pub fn function_b() {
                yield "b";
            }
            
            fn private_function() {
                yield "private";
            }
        }
    "#;

    let mut sources = Sources::new();
    sources.insert(Source::new("test", rune_code)?)?;

    let unit = rune::prepare(&mut sources).with_context(&context).build()?;

    // Unitから関数情報を取得
    println!("=== Unit Debug Info ===");

    // 1. debug_info() で詳細情報を取得
    if let Some(debug_info) = unit.debug_info() {
        println!("✓ Debug info available");

        // 関数一覧
        for (hash, signature) in debug_info.functions.iter() {
            println!("Function: {:?} -> Hash: {:?}", signature.path, hash);
        }
    } else {
        println!("✗ No debug info (release build?)");
    }

    // 3. 関数が存在するかテスト
    println!("\n=== Function Lookup ===");

    // VMで実行してみる
    let runtime = std::sync::Arc::new(context.runtime()?);
    let mut vm = rune::Vm::new(runtime, std::sync::Arc::new(unit));

    match vm.execute(["test_mod", "function_a"], ()) {
        Ok(_) => println!("✓ Function found and executable"),
        Err(e) => println!("✗ Function not found: {:?}", e),
    }

    Ok(())
}

#[test]
fn test_word_function_detection() -> Result<(), Box<dyn std::error::Error>> {
    // 単語定義とRune関数が混在するケース
    let context = Context::with_default_modules()?;

    let rune_code = r#"
        pub mod words {
            // これはRune関数として定義
            pub fn ローカル関数(ctx, args) {
                yield "from rune function";
            }
        }
    "#;

    let mut sources = Sources::new();
    sources.insert(Source::new("test", rune_code)?)?;

    let unit = rune::prepare(&mut sources).with_context(&context).build()?;

    // 2パス解決シミュレーション
    println!("=== 2-Pass Resolution ===");

    // Pass 1: Runeから利用可能な関数を収集
    let mut available_functions = std::collections::HashSet::new();

    if let Some(debug_info) = unit.debug_info() {
        for (_, signature) in debug_info.functions.iter() {
            let path_str = format!("{}", signature.path);
            if let Some(last_segment) = path_str.split("::").last() {
                available_functions.insert(last_segment.to_string());
                println!("Found Rune function: {}", last_segment);
            }
        }
    }

    // Pass 2: DSLの単語呼び出しを解決
    let word_calls = vec!["ローカル関数", "存在しない単語", "挨拶"];

    for word in word_calls {
        if available_functions.contains(word) {
            println!("'{}' -> Rune関数として呼び出し", word);
        } else {
            println!("'{}' -> 文字列辞書から検索", word);
        }
    }

    Ok(())
}
