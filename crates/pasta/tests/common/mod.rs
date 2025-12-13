//! Common test utilities for Pasta integration tests.

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

/// Get a temporary directory for test script storage.
/// This directory persists for the duration of the test suite.
pub fn get_test_script_dir() -> PathBuf {
    static TEMP_DIR: OnceLock<Mutex<TempDir>> = OnceLock::new();
    
    let temp_dir = TEMP_DIR.get_or_init(|| {
        Mutex::new(TempDir::new().expect("Failed to create temp dir for scripts"))
    });
    
    temp_dir.lock().unwrap().path().to_path_buf()
}

/// Get a temporary directory for test persistence storage.
/// This directory persists for the duration of the test suite.
pub fn get_test_persistence_dir() -> PathBuf {
    static TEMP_DIR: OnceLock<Mutex<TempDir>> = OnceLock::new();
    
    let temp_dir = TEMP_DIR.get_or_init(|| {
        Mutex::new(TempDir::new().expect("Failed to create temp dir for persistence"))
    });
    
    temp_dir.lock().unwrap().path().to_path_buf()
}

/// Write a script to a temporary file and return the directory path.
/// 
/// This function creates a main.pasta file in a temporary directory
/// containing the provided script content.
pub fn create_test_script(script_content: &str) -> std::io::Result<PathBuf> {
    use std::fs;
    
    let script_dir = get_test_script_dir();
    let script_file = script_dir.join("main.pasta");
    
    // Create or overwrite the script file
    fs::write(&script_file, script_content)?;
    
    Ok(script_dir)
}
