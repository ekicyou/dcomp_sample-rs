// Task 1.3: 包括的なトランスパイルテスト
// このテストは comprehensive_control_flow_simple.pasta のトランスパイル結果を検証する

use pasta::parser::parse_str;
use pasta::transpiler::Transpiler;
use std::fs;

#[test]
fn test_comprehensive_control_flow_simple_transpile() {
    // Task 1.1で作成したPastaファイルを読み込み
    let pasta_content = fs::read_to_string(
        "tests/fixtures/comprehensive_control_flow_simple.pasta"
    ).expect("Failed to read comprehensive_control_flow_simple.pasta");

    // Task 1.2で作成した期待されるRune出力を読み込み
    let expected_rune = fs::read_to_string(
        "tests/fixtures/comprehensive_control_flow_simple.expected.rn"
    ).expect("Failed to read comprehensive_control_flow_simple.expected.rn");

    // パース
    let ast = parse_str(&pasta_content, "comprehensive_control_flow_simple.pasta")
        .expect("Failed to parse comprehensive_control_flow_simple.pasta");

    // トランスパイル
    let transpiled_rune = Transpiler::transpile(&ast)
        .expect("Failed to transpile comprehensive_control_flow_simple.pasta");

    // 空白を正規化して比較（実装初期は厳密比較、後で調整）
    let normalize = |s: &str| -> String {
        s.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    };

    let normalized_expected = normalize(&expected_rune);
    let normalized_actual = normalize(&transpiled_rune);

    // モジュール構造の検証
    assert!(
        normalized_actual.contains("pub mod 会話_1"),
        "グローバルラベル「会話」がモジュール「会話_1」として生成されていません"
    );

    // __start__関数の検証
    assert!(
        normalized_actual.contains("pub fn __start__(ctx)"),
        "__start__関数が生成されていません"
    );

    // Multiple global labels
    assert!(
        normalized_actual.contains("pub mod 別会話_1"),
        "グローバルラベル「別会話」がモジュール「別会話_1」として生成されていません"
    );

    // mod pasta の検証
    assert!(
        normalized_actual.contains("pub mod pasta"),
        "mod pasta が生成されていません"
    );

    assert!(
        normalized_actual.contains("pub fn label_selector(label, filters)"),
        "label_selector関数が生成されていません"
    );

    // label_selectorのmatch文検証
    assert!(
        normalized_actual.contains("match id {"),
        "label_selectorのmatch文が生成されていません"
    );

    assert!(
        normalized_actual.contains("1 => crate::会話_1::__start__"),
        "label_selectorのmatch文にID 1のマッピングがありません"
    );

    assert!(
        normalized_actual.contains("2 => crate::別会話_1::__start__"),
        "label_selectorのmatch文にID 2のマッピングがありません"
    );

    // 発言者切り替えの検証
    assert!(
        normalized_actual.contains("ctx.actor = さくら"),
        "発言者切り替えが正しく生成されていません"
    );

    assert!(
        normalized_actual.contains("yield Actor(\"さくら\")"),
        "発言者切り替えのyield Actor()が生成されていません"
    );

    // yield Talk()の検証
    assert!(
        normalized_actual.contains("yield Talk(\"おはよう！\")"),
        "yield Talk()が生成されていません"
    );

    // 厳密比較（デバッグ用）
    if normalized_expected != normalized_actual {
        eprintln!("=== Expected ===");
        eprintln!("{}", expected_rune);
        eprintln!("\n=== Actual ===");
        eprintln!("{}", transpiled_rune);
        eprintln!("\n=== Normalized Expected ===");
        eprintln!("{}", normalized_expected);
        eprintln!("\n=== Normalized Actual ===");
        eprintln!("{}", normalized_actual);
        
        // 差分を見やすくするために行ごとに比較
        let expected_lines: Vec<&str> = normalized_expected.lines().collect();
        let actual_lines: Vec<&str> = normalized_actual.lines().collect();
        
        eprintln!("\n=== Line-by-line Diff ===");
        let max_lines = expected_lines.len().max(actual_lines.len());
        for i in 0..max_lines {
            let exp = expected_lines.get(i).unwrap_or(&"<missing>");
            let act = actual_lines.get(i).unwrap_or(&"<missing>");
            if exp != act {
                eprintln!("Line {}: DIFF", i + 1);
                eprintln!("  Expected: {}", exp);
                eprintln!("  Actual:   {}", act);
            }
        }
    }

    assert_eq!(
        normalized_expected, normalized_actual,
        "トランスパイル結果が期待される出力と一致しません"
    );
}
