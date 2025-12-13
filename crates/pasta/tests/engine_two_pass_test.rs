// Test PastaEngine with two-pass transpiler

use pasta::PastaEngine;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_engine_with_simple_project() {
    let script_root = std::env::current_dir()
        .unwrap()
        .join("tests/fixtures/simple-test");
    let temp_dir = TempDir::new().unwrap();
    
    // Create engine with simple test project
    let result = PastaEngine::new(&script_root, temp_dir.path());
    
    match result {
        Ok(engine) => {
            println!("Engine created successfully!");
            
            // Test label existence
            assert!(engine.has_label("会話"), "Label '会話' should exist");
        }
        Err(e) => {
            panic!("Failed to create engine: {:?}", e);
        }
    }
}

#[test]
#[ignore] // test-project has encoding issues
fn test_engine_with_test_project() {
    let script_root = std::env::current_dir()
        .unwrap()
        .join("tests/fixtures/test-project");
    let temp_dir = TempDir::new().unwrap();
    
    // Create engine with test project
    let result = PastaEngine::new(&script_root, temp_dir.path());
    
    match result {
        Ok(engine) => {
            println!("Engine created successfully!");
            
            // Test label existence
            assert!(engine.has_label("greetings"), "Label 'greetings' should exist");
        }
        Err(e) => {
            panic!("Failed to create engine: {:?}", e);
        }
    }
}

#[test]
#[ignore] // Ignore until we can test execution
fn test_engine_execute_label() {
    let script_root = std::env::current_dir()
        .unwrap()
        .join("tests/fixtures/test-project");
    let temp_dir = TempDir::new().unwrap();
    
    let mut engine = PastaEngine::new(&script_root, temp_dir.path()).unwrap();
    
    // Execute a label
    let events = engine.execute_label("greetings").unwrap();
    
    // Verify events
    assert!(!events.is_empty(), "Should generate events");
}
