//! PastaEngine - Main entry point for executing Pasta DSL scripts.
//!
//! This module provides the integrated engine that combines parsing, transpiling,
//! and runtime execution to provide a high-level API for running Pasta scripts.

use crate::{
    error::{PastaError, Result},
    ir::ScriptEvent,
    parser::parse_str,
    runtime::{DefaultRandomSelector, LabelInfo, LabelTable, RandomSelector},
    transpiler::Transpiler,
    LabelDef, LabelScope,
};
use rune::{Context, Vm};
use std::collections::HashMap;
use std::sync::Arc;

/// Main Pasta script engine.
///
/// This engine integrates all layers of the Pasta stack:
/// - Parser: Parses Pasta DSL to AST
/// - Transpiler: Converts AST to Rune source code
/// - Runtime: Executes Rune code with generators
///
/// # Example
///
/// ```no_run
/// use pasta::PastaEngine;
///
/// let script = r#"
/// ＊挨拶
///     さくら：こんにちは！
///     うにゅう：やあ！
/// "#;
///
/// let mut engine = PastaEngine::new(script)?;
/// let events = engine.execute_label("挨拶")?;
///
/// for event in events {
///     println!("{:?}", event);
/// }
/// # Ok::<(), pasta::PastaError>(())
/// ```
pub struct PastaEngine {
    /// The compiled Rune unit.
    unit: Arc<rune::Unit>,
    /// The Rune runtime context.
    runtime: Arc<rune::runtime::RuntimeContext>,
    /// Label table for label lookup and random selection.
    label_table: LabelTable,
}

impl PastaEngine {
    /// Create a new PastaEngine from a Pasta DSL script.
    ///
    /// This performs:
    /// 1. Parsing the DSL to AST
    /// 2. Transpiling AST to Rune source code
    /// 3. Compiling Rune code to bytecode
    /// 4. Building label table for runtime lookup
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The DSL has syntax errors
    /// - The transpilation fails
    /// - The Rune compilation fails
    pub fn new(script: &str) -> Result<Self> {
        Self::with_random_selector(script, Box::new(DefaultRandomSelector::new()))
    }

    /// Create a new PastaEngine with a custom random selector.
    ///
    /// This is primarily useful for testing with deterministic random selection.
    pub fn with_random_selector(
        script: &str,
        random_selector: Box<dyn RandomSelector>,
    ) -> Result<Self> {
        // Step 1: Parse DSL to AST
        let ast = parse_str(script, "<script>")?;

        // Step 2: Transpile AST to Rune source
        let rune_source = Transpiler::transpile(&ast)?;

        // Debug output for development
        #[cfg(debug_assertions)]
        {
            eprintln!("=== Generated Rune Source ===");
            eprintln!("{}", rune_source);
            eprintln!("=============================");
        }

        // Step 3: Build label table
        let mut label_table = LabelTable::new(random_selector);
        Self::register_labels(&mut label_table, &ast.labels, None)?;

        // Step 4: Compile Rune code
        let mut context = Context::with_default_modules().map_err(|e| {
            PastaError::RuneRuntimeError(format!("Failed to create Rune context: {}", e))
        })?;

        // Install standard library
        context.install(crate::stdlib::create_module().map_err(|e| {
            PastaError::RuneRuntimeError(format!("Failed to install stdlib: {}", e))
        })?)
            .map_err(|e| PastaError::RuneRuntimeError(format!("Failed to install context: {}", e)))?;

        let runtime = Arc::new(context.runtime().map_err(|e| {
            PastaError::RuneRuntimeError(format!("Failed to create runtime: {}", e))
        })?);

        // Compile the Rune source
        let mut sources = rune::Sources::new();
        sources.insert(rune::Source::new("entry", rune_source)
            .map_err(|e| PastaError::RuneRuntimeError(format!("Failed to create source: {}", e)))?)
            .map_err(|e| PastaError::RuneRuntimeError(format!("Failed to insert source: {}", e)))?;

        let unit = rune::prepare(&mut sources)
            .with_context(&context)
            .build()
            .map_err(|e| PastaError::RuneRuntimeError(format!("Failed to compile Rune: {}", e)))?;

        Ok(Self {
            unit: Arc::new(unit),
            runtime,
            label_table,
        })
    }

    /// Register labels from AST into label table.
    fn register_labels(
        label_table: &mut LabelTable,
        labels: &[LabelDef],
        parent_name: Option<&str>,
    ) -> Result<()> {
        for label in labels {
            let fn_name = Self::generate_fn_name(label, parent_name);
            
            let mut attributes = HashMap::new();
            for attr in &label.attributes {
                attributes.insert(attr.key.clone(), attr.value.to_string());
            }

            label_table.register(LabelInfo {
                name: label.name.clone(),
                scope: label.scope,
                attributes,
                fn_name,
                parent: parent_name.map(|s| s.to_string()),
            });

            // Register local labels recursively
            if !label.local_labels.is_empty() {
                Self::register_labels(label_table, &label.local_labels, Some(&label.name))?;
            }
        }

        Ok(())
    }

    /// Generate a Rune function name for a label.
    fn generate_fn_name(label: &LabelDef, parent_name: Option<&str>) -> String {
        let sanitize = |name: &str| name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
        
        match label.scope {
            LabelScope::Global => sanitize(&label.name),
            LabelScope::Local => {
                if let Some(parent) = parent_name {
                    format!("{}__{}", sanitize(parent), sanitize(&label.name))
                } else {
                    sanitize(&label.name)
                }
            }
        }
    }

    /// Execute a label and return all events synchronously.
    ///
    /// This looks up the label (with optional attribute filters), selects one
    /// if multiple labels match, and executes it to completion, returning all events.
    ///
    /// # Arguments
    ///
    /// * `label_name` - The name of the label to execute
    ///
    /// # Returns
    ///
    /// A vector of all `ScriptEvent`s generated by the label.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The label is not found
    /// - No labels match the filters
    /// - A runtime error occurs during execution
    pub fn execute_label(&mut self, label_name: &str) -> Result<Vec<ScriptEvent>> {
        self.execute_label_with_filters(label_name, &HashMap::new())
    }

    /// Execute a label with attribute filters and return all events.
    ///
    /// This is the full version of `execute_label` that accepts filters.
    pub fn execute_label_with_filters(
        &mut self,
        label_name: &str,
        filters: &HashMap<String, String>,
    ) -> Result<Vec<ScriptEvent>> {
        // Look up the label
        let fn_name = self.label_table.find_label(label_name, filters)?;

        // Create a new VM for this execution
        let mut vm = Vm::new(self.runtime.clone(), self.unit.clone());

        // Call the function
        let hash = rune::Hash::type_hash(&[fn_name.as_str()]);
        
        // Execute and get a generator
        let execution = vm.execute(hash, ())
            .map_err(|e| PastaError::VmError(e))?;
        
        let mut generator = execution.into_generator();
        
        // Collect all yielded events
        let mut events = Vec::new();
        let unit_value = rune::to_value(()).map_err(|e| {
            PastaError::RuneRuntimeError(format!("Failed to create unit value: {}", e))
        })?;
        
        loop {
            match generator.resume(unit_value.clone()) {
                rune::runtime::VmResult::Ok(rune::runtime::GeneratorState::Yielded(value)) => {
                    let event: ScriptEvent = rune::from_value(value)
                        .map_err(|e| PastaError::RuneRuntimeError(format!("Failed to convert yielded value: {}", e)))?;
                    events.push(event);
                }
                rune::runtime::VmResult::Ok(rune::runtime::GeneratorState::Complete(_)) => {
                    break;
                }
                rune::runtime::VmResult::Err(e) => {
                    return Err(PastaError::VmError(e));
                }
            }
        }
        
        Ok(events)
    }

    /// Check if a label exists in the label table.
    pub fn has_label(&self, label_name: &str) -> bool {
        self.label_table.has_label(label_name)
    }

    /// Get all registered label names.
    pub fn label_names(&self) -> Vec<String> {
        self.label_table.label_names()
    }

    /// Execute a chain of labels automatically.
    ///
    /// This method supports "chain talk" by repeatedly executing labels
    /// until no chain is detected. A label can specify a chain target
    /// using the @chain attribute.
    ///
    /// # Arguments
    ///
    /// * `initial_label` - The starting label name
    /// * `max_chain_depth` - Maximum number of labels to chain (prevents infinite loops)
    ///
    /// # Returns
    ///
    /// A vector of all `ScriptEvent`s generated by the entire chain.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pasta::PastaEngine;
    /// let script = r#"
    /// ＊挨拶
    ///     さくら：おはよう！
    /// 
    /// ＊挨拶_続き
    ///     さくら：今日も元気だね！
    /// "#;
    /// let mut engine = PastaEngine::new(script)?;
    /// 
    /// // Execute chain manually
    /// let mut all_events = engine.execute_label("挨拶")?;
    /// all_events.extend(engine.execute_label("挨拶_続き")?);
    /// # Ok::<(), pasta::PastaError>(())
    /// ```
    pub fn execute_label_chain(
        &mut self,
        initial_label: &str,
        max_chain_depth: usize,
    ) -> Result<Vec<ScriptEvent>> {
        let mut all_events = Vec::new();
        let mut current_label = initial_label.to_string();
        let mut depth = 0;

        while depth < max_chain_depth {
            // Execute current label
            let events = self.execute_label(&current_label)?;
            
            // Check if any event indicates a chain
            let mut next_label = None;
            for event in &events {
                if let ScriptEvent::FireEvent { event_name, .. } = event {
                    // Check if this is a chain event
                    if event_name.starts_with("chain:") {
                        next_label = Some(event_name.trim_start_matches("chain:").to_string());
                        break;
                    }
                }
            }
            
            all_events.extend(events);
            
            // If no chain detected, stop
            if let Some(label) = next_label {
                current_label = label;
                depth += 1;
            } else {
                break;
            }
        }

        Ok(all_events)
    }
}

impl Drop for PastaEngine {
    /// Persist engine state on destruction.
    ///
    /// This implementation saves:
    /// - Global variables (if variable manager is added in future)
    /// - Label execution history and caches
    ///
    /// Currently, this is a placeholder for Task 5.5 implementation.
    /// Full persistence will be added when VariableManager is integrated
    /// into PastaEngine.
    fn drop(&mut self) {
        // TODO: Persist global variables when VariableManager is integrated
        // self.variables.save_to_disk().ok();
        
        // TODO: Persist label execution history/cache
        // self.label_table.save_cache().ok();
        
        // For now, we just log that the engine is being dropped
        #[cfg(debug_assertions)]
        {
            eprintln!("[PastaEngine] Dropping engine (persistence not yet implemented)");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_engine_new() {
        let script = r#"
＊挨拶
    さくら：こんにちは
"#;
        let engine = PastaEngine::new(script);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_engine_invalid_script() {
        let script = "invalid syntax @@@";
        let engine = PastaEngine::new(script);
        assert!(engine.is_err());
    }

    #[test]
    fn test_engine_has_label() {
        let script = r#"
＊挨拶
    さくら：こんにちは
"#;
        let engine = PastaEngine::new(script).unwrap();
        assert!(engine.has_label("挨拶"));
        assert!(!engine.has_label("存在しない"));
    }

    #[test]
    fn test_engine_label_names() {
        let script = r#"
＊挨拶
    さくら：こんにちは

＊別れ
    さくら：さようなら
"#;
        let engine = PastaEngine::new(script).unwrap();
        let names = engine.label_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"挨拶".to_string()));
        assert!(names.contains(&"別れ".to_string()));
    }

    #[test]
    fn test_execute_label_not_found() {
        let script = r#"
＊挨拶
    さくら：こんにちは
"#;
        let mut engine = PastaEngine::new(script).unwrap();
        let result = engine.execute_label("存在しない");
        assert!(result.is_err());
        match result {
            Err(PastaError::LabelNotFound { label }) => {
                assert_eq!(label, "存在しない");
            }
            _ => panic!("Expected LabelNotFound error"),
        }
    }

    #[test]
    fn test_execute_label_returns_events() {
        let script = r#"
＊test
    さくら：こんにちは
"#;
        let mut engine = PastaEngine::new(script).unwrap();
        let events = engine.execute_label("test").unwrap();
        // Should have at least ChangeSpeaker + Talk
        assert!(events.len() >= 2);
    }
}
