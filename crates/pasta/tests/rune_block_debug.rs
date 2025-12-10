// Task 11: Rune Block support - rune_content rule not yet implemented
// These tests are disabled until the rune_content grammar rule is added

#[cfg(feature = "rune_block_support")]
mod tests {
    use pest::Parser;
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "parser/pasta.pest"]
    struct PastaParser;

    #[test]
    fn test_rune_content_simple() {
        let input = "fn test() {\n";
        let result = PastaParser::parse(Rule::rune_content, input);
        println!("Rune content parse: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rune_block_parts() {
        // Test rune_start
        let result = PastaParser::parse(Rule::rune_start, "```rune");
        println!("Start: {:?}", result);
        assert!(result.is_ok());
        
        // Test rune_end
        let result = PastaParser::parse(Rule::rune_end, "```");
        println!("End: {:?}", result);
        assert!(result.is_ok());
        
        // Test rune_content
        let result = PastaParser::parse(Rule::rune_content, "fn test() {}\n");
        println!("Content: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rune_block_step_by_step() {
        let full = r#"  ```rune
  fn test() {
    return 42;
  }
  ```
"#;
        
        println!("Full input:\n{}", full);
        println!("Trying to parse as rune_block...");
        
        let result = PastaParser::parse(Rule::rune_block, full);
        match &result {
            Ok(pairs) => {
                println!("SUCCESS!");
                for pair in pairs.clone() {
                    println!("Pair: {:?}", pair);
                }
            }
            Err(e) => {
                println!("ERROR: {:?}", e);
            }
        }
        
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }
}
