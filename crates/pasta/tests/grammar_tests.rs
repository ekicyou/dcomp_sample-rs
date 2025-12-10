//! Tests for Pasta DSL grammar parsing (pest)
//!
//! These tests validate that the pest grammar correctly parses various Pasta DSL constructs.

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/pasta.pest"]
struct PastaParser;

#[cfg(test)]
mod grammar_tests {
    use super::*;

    #[test]
    fn test_simple_global_label() {
        let input = "＊挨拶\n";
        let result = PastaParser::parse(Rule::global_label, input);
        assert!(result.is_ok(), "Failed to parse simple global label");
    }

    #[test]
    fn test_global_label_with_speech() {
        let input = "＊挨拶\n  さくら：こんにちは\n";
        let result = PastaParser::parse(Rule::global_label, input);
        assert!(result.is_ok(), "Failed to parse global label with speech");
    }

    #[test]
    fn test_halfwidth_global_label() {
        let input = "*greeting\n";
        let result = PastaParser::parse(Rule::global_label, input);
        assert!(result.is_ok(), "Failed to parse half-width global label");
    }

    #[test]
    fn test_speech_line() {
        let input = "  さくら：こんにちは\n";
        let result = PastaParser::parse(Rule::speech_line, input);
        assert!(
            result.is_ok(),
            "Failed to parse speech line: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_speech_with_var_ref() {
        let input = "  さくら：こんにちは＠ユーザー名さん\n";
        let result = PastaParser::parse(Rule::speech_line, input);
        assert!(
            result.is_ok(),
            "Failed to parse speech with variable reference"
        );
    }

    #[test]
    fn test_speech_with_func_call() {
        let input = "  さくら：こんにちは＠W（300）お元気ですか\n";
        let result = PastaParser::parse(Rule::speech_line, input);
        assert!(result.is_ok(), "Failed to parse speech with function call");
    }

    #[test]
    fn test_call_statement() {
        let input = "  ＞挨拶\n";
        let result = PastaParser::parse(Rule::call_stmt, input);
        assert!(result.is_ok(), "Failed to parse call statement");
    }

    #[test]
    fn test_call_global_label() {
        let input = "  ＞＊挨拶\n";
        let result = PastaParser::parse(Rule::call_stmt, input);
        assert!(result.is_ok(), "Failed to parse call to global label");
    }

    #[test]
    fn test_jump_statement() {
        let input = "  ？終了\n";
        let result = PastaParser::parse(Rule::jump_stmt, input);
        assert!(result.is_ok(), "Failed to parse jump statement");
    }

    #[test]
    fn test_attribute_line() {
        let input = "  ＠時間帯：朝\n";
        let result = PastaParser::parse(Rule::attribute_line, input);
        assert!(result.is_ok(), "Failed to parse attribute line");
    }

    #[test]
    fn test_var_assign_local() {
        let input = "  ＄カウンター＝1\n";
        let result = PastaParser::parse(Rule::var_assign, input);
        assert!(
            result.is_ok(),
            "Failed to parse local variable assignment: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_var_assign_global() {
        let input = "  ＄＊カウンター＝1\n";
        let result = PastaParser::parse(Rule::var_assign, input);
        assert!(
            result.is_ok(),
            "Failed to parse global variable assignment: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_local_label() {
        let input = "  ー朝の挨拶\n    さくら：おはよう\n";
        let result = PastaParser::parse(Rule::local_label, input);
        assert!(result.is_ok(), "Failed to parse local label");
    }

    #[test]
    fn test_string_literal_ja() {
        let input = "「こんにちは」";
        let result = PastaParser::parse(Rule::string_literal, input);
        assert!(result.is_ok(), "Failed to parse Japanese string literal");
    }

    #[test]
    fn test_string_literal_en() {
        let input = "\"hello world\"";
        let result = PastaParser::parse(Rule::string_literal, input);
        assert!(result.is_ok(), "Failed to parse English string literal");
    }

    #[test]
    fn test_number_literal() {
        let input = "123";
        let result = PastaParser::parse(Rule::number_literal, input);
        assert!(result.is_ok(), "Failed to parse integer literal");

        let input = "3.14";
        let result = PastaParser::parse(Rule::number_literal, input);
        assert!(result.is_ok(), "Failed to parse float literal");
    }

    #[test]
    fn test_func_call() {
        let input = "＠関数名（「引数１」　「引数２」）";
        let result = PastaParser::parse(Rule::func_call, input);
        assert!(
            result.is_ok(),
            "Failed to parse function call: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_sakura_script() {
        let input = "\\n";
        let result = PastaParser::parse(Rule::sakura_script, input);
        assert!(result.is_ok(), "Failed to parse sakura script escape");
    }

    #[test]
    fn test_complete_file() {
        let input = r#"＊挨拶
  ＠時間帯：朝
  さくら：おはよう！
  さくら：今日もいい天気だね

＊挨拶
  ＠時間帯：昼
  さくら：こんにちは

＊終了
  さくら：またね\nバイバイ
  ？＊挨拶
"#;
        let result = PastaParser::parse(Rule::file, input);
        assert!(
            result.is_ok(),
            "Failed to parse complete file: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_rune_block() {
        let input = r#"  ```rune
  fn test() {
    return 42;
  }
  ```
"#;
        let result = PastaParser::parse(Rule::rune_block, input);
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(
            result.is_ok(),
            "Failed to parse rune block: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_long_jump() {
        let input = "  ＞＊挨拶ー朝\n";
        let result = PastaParser::parse(Rule::call_stmt, input);
        assert!(result.is_ok(), "Failed to parse long jump");
    }

    #[test]
    fn test_dynamic_target() {
        let input = "  ？＠変数名\n";
        let result = PastaParser::parse(Rule::jump_stmt, input);
        assert!(result.is_ok(), "Failed to parse dynamic jump target");
    }

    #[test]
    fn test_named_arguments() {
        let input = "＠関数（name：「太郎」　age：20）";
        let result = PastaParser::parse(Rule::func_call, input);
        assert!(
            result.is_ok(),
            "Failed to parse function call with named arguments: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_expression() {
        let input = "1 + 2 * 3";
        let result = PastaParser::parse(Rule::expr, input);
        assert!(result.is_ok(), "Failed to parse expression");
    }

    #[test]
    fn test_continuation_line() {
        let input = r#"  さくら：こんにちは
    今日はいい天気ですね
    どこか行きましょう
"#;
        let result = PastaParser::parse(Rule::speech_line, input);
        assert!(result.is_ok(), "Failed to parse continuation lines");
    }
}
