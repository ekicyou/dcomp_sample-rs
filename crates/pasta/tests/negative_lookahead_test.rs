// Task 11: Rune Block support - rune_content rule not yet implemented
// This test is disabled until the rune_content grammar rule is added

#[cfg(feature = "rune_block_support")]
mod tests {
    use pest::Parser;
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "parser/pasta.pest"]
    struct PastaParser;

    #[test]
    fn test_negative_lookahead() {
        // This should match up to but not including "\n  ```"
        let input = "  fn test() {\n    return 42;\n  }\n";
        let result = PastaParser::parse(Rule::rune_content, input);
        println!("Result: {:?}", result);
        
        if let Ok(pairs) = result {
            for pair in pairs {
                println!("Matched: {:?}", pair.as_str());
                println!("Length: {}", pair.as_str().len());
                println!("Ends with newline: {}", pair.as_str().ends_with('\n'));
            }
        } else {
            println!("Failed to parse");
        }
    }
}
