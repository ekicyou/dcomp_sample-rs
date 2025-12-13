// comprehensive_control_flow.pastaのトランスパイルテスト
// P0完了条件の検証

use pasta::parser::parse_file;
use pasta::transpiler::Transpiler;
use std::path::Path;

#[test]
fn test_comprehensive_control_flow_transpile() {
    let input_path = Path::new("tests/fixtures/comprehensive_control_flow.pasta");

    // パースしてトランスパイル実行
    let file = parse_file(input_path).expect("Failed to parse file");
    let result = Transpiler::transpile_to_string(&file);
    assert!(result.is_ok(), "トランスパイルに失敗: {:?}", result.err());

    let rune_code = result.unwrap();

    // 基本的な構造チェック
    assert!(
        rune_code.contains("pub mod メイン_1"),
        "グローバルラベルのモジュールが生成されていない"
    );
    assert!(
        rune_code.contains("pub fn __start__(ctx, args)"),
        "__start__関数が生成されていない"
    );
    assert!(
        rune_code.contains("pub fn 自己紹介_1(ctx, args)"),
        "ローカルラベル関数が生成されていない"
    );
    assert!(
        rune_code.contains("mod pasta"),
        "mod pastaが生成されていない"
    );
    assert!(
        rune_code.contains("pub fn call("),
        "pasta::call関数が生成されていない"
    );
    assert!(
        rune_code.contains("pub fn jump("),
        "pasta::jump関数が生成されていない"
    );

    // Runeブロックがそのまま含まれているか確認
    assert!(
        rune_code.contains("pub fn ローカル関数(ctx, args)"),
        "Runeブロックが含まれていない"
    );

    eprintln!("=== Generated Rune Code ===");
    eprintln!("{}", rune_code);
    eprintln!("=== End of Generated Code ===");
}

#[test]
fn test_comprehensive_control_flow_rune_compile() {
    let input_path = Path::new("tests/fixtures/comprehensive_control_flow.pasta");

    // パースしてトランスパイル実行
    let file = parse_file(input_path).expect("Failed to parse file");
    let result = Transpiler::transpile_to_string(&file);
    assert!(result.is_ok(), "トランスパイルに失敗: {:?}", result.err());

    let rune_code = result.unwrap();

    // Runeコンパイルテスト（pasta_stdlibモジュール登録）
    use pasta::stdlib;
    use rune::{Context, Diagnostics, Source, Sources};

    let mut context = Context::new();
    context
        .install(stdlib::create_module().expect("Failed to create pasta_stdlib module"))
        .expect("Failed to install pasta_stdlib module");

    // アクター定義を含むmain.rn（最小限）
    let main_rn = r#"
pub const さくら = #{ name: "さくら" };
pub const うにゅう = #{ name: "うにゅう" };
pub const ななこ = #{ name: "ななこ" };
"#;

    let mut sources = Sources::new();
    sources
        .insert(Source::new("main", main_rn).expect("Failed to create main source"))
        .expect("Failed to insert main source");
    sources
        .insert(Source::new("entry", &rune_code).expect("Failed to create source"))
        .expect("Failed to insert source");

    let mut diagnostics = Diagnostics::new();
    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();

    if result.is_err() || diagnostics.has_error() {
        eprintln!("=== Rune Compilation Diagnostics ===");
        for diagnostic in diagnostics.diagnostics() {
            eprintln!("{:?}", diagnostic);
        }
        eprintln!("=== Generated Code ===");
        eprintln!("{}", rune_code);
    }

    assert!(
        !diagnostics.has_error(),
        "Runeコンパイルでエラーが発生: {:?}",
        diagnostics
    );
    assert!(result.is_ok(), "Runeコンパイルに失敗: {:?}", result.err());

    eprintln!("✅ P0合格: comprehensive_control_flow.pastaのトランスパイルとRuneコンパイルが成功しました！");
}
