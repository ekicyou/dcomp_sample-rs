//! PastaEngine - Main entry point for executing Pasta DSL scripts.
//!
//! This module provides the integrated engine that combines parsing, transpiling,
//! and runtime execution to provide a high-level API for running Pasta scripts.

use crate::{
    cache::ParseCache,
    error::{PastaError, Result},
    ir::ScriptEvent,
    loader::{DirectoryLoader, ErrorLogWriter},
    parser::{parse_file, parse_str},
    runtime::{DefaultRandomSelector, LabelInfo, LabelTable, RandomSelector},
    transpiler::Transpiler,
    LabelDef, LabelScope, PastaFile,
};
use rune::{Context, Vm};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Main Pasta script engine.
///
/// This engine integrates all layers of the Pasta stack:
/// - Parser: Parses Pasta DSL to AST
/// - Transpiler: Converts AST to Rune source code
/// - Runtime: Executes Rune code with generators
///
/// # Instance Independence
///
/// Each PastaEngine instance is completely independent and owns all its data,
/// including its own parse cache. Multiple engine instances can coexist safely
/// in the same process or across threads.
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
    /// Parse cache (instance-local).
    cache: ParseCache,
    /// Persistence directory path (optional).
    persistence_path: Option<PathBuf>,
}

impl PastaEngine {
    /// Create a new PastaEngine from script and persistence directories.
    ///
    /// This is the primary constructor for production use. It loads all `.pasta` files
    /// from the `dic/` directory and `main.rune` from the script root, following
    /// areka-P0-script-engine conventions.
    ///
    /// # Directory Structure
    ///
    /// ```text
    /// script_root/
    ///   ├── main.rune           # Rune entry point
    ///   └── dic/                # Pasta scripts
    ///       ├── *.pasta
    ///       └── ...
    ///
    /// persistence_root/
    ///   ├── variables.toml      # Persisted variables
    ///   └── ...                 # Other runtime data
    /// ```
    ///
    /// # Arguments
    ///
    /// * `script_root` - Script root directory (must be absolute path)
    /// * `persistence_root` - Persistence root directory (absolute or relative)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Script path is not absolute
    /// - Script directory does not exist or is not readable
    /// - Persistence directory does not exist
    /// - `dic/` directory not found
    /// - `main.rune` not found
    /// - Parse errors in `.pasta` files
    /// - Rune compilation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pasta::PastaEngine;
    /// use std::path::Path;
    ///
    /// let engine = PastaEngine::new(
    ///     Path::new("/path/to/script_root"),
    ///     Path::new("/path/to/persistence_root")
    /// )?;
    /// # Ok::<(), pasta::PastaError>(())
    /// ```
    pub fn new(
        script_root: impl AsRef<Path>,
        persistence_root: impl AsRef<Path>,
    ) -> Result<Self> {
        Self::with_random_selector(
            script_root,
            persistence_root,
            Box::new(DefaultRandomSelector::new()),
        )
    }

    /// Create a new PastaEngine with a custom random selector.
    ///
    /// This is primarily useful for testing with deterministic random selection.
    ///
    /// # Arguments
    ///
    /// * `script_root` - Script root directory (must be absolute path)
    /// * `persistence_root` - Persistence root directory (absolute or relative)
    /// * `random_selector` - Custom random selector implementation
    ///
    /// # Errors
    ///
    /// Same as `new()`
    pub fn with_random_selector(
        script_root: impl AsRef<Path>,
        persistence_root: impl AsRef<Path>,
        random_selector: Box<dyn RandomSelector>,
    ) -> Result<Self> {
        let path = script_root.as_ref();

        // Step 1: Load files from directory
        let loaded = DirectoryLoader::load(path)?;

        // Step 2: Parse all .pasta files (collect errors)
        let mut all_labels = Vec::new();
        let mut parse_errors = Vec::new();

        for pasta_file in &loaded.pasta_files {
            match parse_file(pasta_file) {
                Ok(ast) => {
                    all_labels.extend(ast.labels);
                }
                Err(e) => {
                    // Collect parse errors, fail-fast on other errors
                    if let Some(parse_err) = Option::from(&e) {
                        parse_errors.push(parse_err);
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        // Step 3: If parse errors exist, log and return error
        if !parse_errors.is_empty() {
            ErrorLogWriter::log(&loaded.script_root, &parse_errors);
            return Err(PastaError::MultipleParseErrors {
                errors: parse_errors,
            });
        }

        // Step 4: Merge all ASTs into a single AST
        let merged_ast = PastaFile {
            path: loaded.script_root.clone(),
            labels: all_labels,
            span: crate::parser::Span::new(1, 1, 1, 0),
        };

        // Step 5: Transpile merged AST to Rune source
        let rune_source = Transpiler::transpile(&merged_ast)?;

        #[cfg(debug_assertions)]
        {
            eprintln!("=== Generated Rune Source (from directory) ===");
            eprintln!("{}", rune_source);
            eprintln!("===============================================");
        }

        // Step 6: Build Rune sources with main.rune
        let mut context = Context::with_default_modules().map_err(|e| {
            PastaError::RuneRuntimeError(format!("Failed to create Rune context: {}", e))
        })?;

        // Install standard library
        context
            .install(crate::stdlib::create_module().map_err(|e| {
                PastaError::RuneRuntimeError(format!("Failed to install stdlib: {}", e))
            })?)
            .map_err(|e| {
                PastaError::RuneRuntimeError(format!("Failed to install context: {}", e))
            })?;

        let runtime = Arc::new(context.runtime().map_err(|e| {
            PastaError::RuneRuntimeError(format!("Failed to create runtime: {}", e))
        })?);

        let mut sources = rune::Sources::new();

        // Add transpiled pasta source
        sources
            .insert(rune::Source::new("entry", rune_source).map_err(|e| {
                PastaError::RuneRuntimeError(format!("Failed to create source: {}", e))
            })?)
            .map_err(|e| PastaError::RuneRuntimeError(format!("Failed to insert source: {}", e)))?;

        // Add main.rune
        sources
            .insert(rune::Source::from_path(&loaded.main_rune).map_err(|e| {
                PastaError::RuneRuntimeError(format!("Failed to load main.rune: {}", e))
            })?)
            .map_err(|e| {
                PastaError::RuneRuntimeError(format!("Failed to insert main.rune: {}", e))
            })?;

        // Step 7: Compile Rune code
        let unit = rune::prepare(&mut sources)
            .with_context(&context)
            .build()
            .map_err(|e| PastaError::RuneCompileError(format!("Failed to compile Rune: {}", e)))?;

        // Step 8: Build label table
        let mut label_table = LabelTable::new(random_selector);
        Self::register_labels(&mut label_table, &merged_ast.labels, None)?;

        // Step 9: Validate persistence path
        let validated_persistence_path =
            Self::validate_persistence_path(persistence_root.as_ref())?;

        // Step 10: Create instance-local cache and construct engine
        let cache = ParseCache::new();

        Ok(Self {
            unit: Arc::new(unit),
            runtime,
            label_table,
            cache,
            persistence_path: Some(validated_persistence_path),
        })
    }

    /// Validate and canonicalize the persistence path.
    fn validate_persistence_path(path: &Path) -> Result<PathBuf> {
        if !path.exists() {
            tracing::error!(
                path = %path.display(),
                error = "Directory not found",
                "[PastaEngine::validate_persistence_path] Persistence directory does not exist"
            );
            return Err(PastaError::PersistenceDirectoryNotFound {
                path: path.display().to_string(),
            });
        }

        if !path.is_dir() {
            tracing::error!(
                path = %path.display(),
                error = "Not a directory",
                "[PastaEngine::validate_persistence_path] Path is not a directory"
            );
            return Err(PastaError::InvalidPersistencePath {
                path: path.display().to_string(),
            });
        }

        let canonical = path.canonicalize().map_err(|e| {
            tracing::error!(
                path = %path.display(),
                error = %e,
                "[PastaEngine::validate_persistence_path] Failed to canonicalize path"
            );
            PastaError::InvalidPersistencePath {
                path: path.display().to_string(),
            }
        })?;

        tracing::info!(
            path = %canonical.display(),
            "[PastaEngine::validate_persistence_path] Persistence path configured"
        );

        Ok(canonical)
    }

    /// Build the PastaEngine with optional persistence path.
    fn build_engine(
        script: &str,
        persistence_path: Option<PathBuf>,
        random_selector: Box<dyn RandomSelector>,
    ) -> Result<Self> {
        // Step 1: Create empty instance-local cache
        let mut cache = ParseCache::new();

        // Step 2: Try to get from cache first
        let (ast, rune_source) = if let Some(cached) = cache.get(script) {
            // Cache hit - reuse parsed AST and Rune source
            #[cfg(debug_assertions)]
            {
                eprintln!("[PastaEngine] Cache hit for script");
            }
            cached
        } else {
            // Cache miss - parse and transpile
            #[cfg(debug_assertions)]
            {
                eprintln!("[PastaEngine] Cache miss - parsing script");
            }

            // Parse DSL to AST
            let ast = parse_str(script, "<script>")?;

            // Transpile AST to Rune source
            let rune_source = Transpiler::transpile(&ast)?;

            // Store in cache for future use
            cache.insert(script, ast.clone(), rune_source.clone());

            (ast, rune_source)
        };

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

        // Install standard library (includes persistence functions)
        context
            .install(crate::stdlib::create_module().map_err(|e| {
                PastaError::RuneRuntimeError(format!("Failed to install stdlib: {}", e))
            })?)
            .map_err(|e| {
                PastaError::RuneRuntimeError(format!("Failed to install context: {}", e))
            })?;

        let runtime = Arc::new(context.runtime().map_err(|e| {
            PastaError::RuneRuntimeError(format!("Failed to create runtime: {}", e))
        })?);

        // Compile the Rune source
        let mut sources = rune::Sources::new();
        sources
            .insert(rune::Source::new("entry", rune_source).map_err(|e| {
                PastaError::RuneRuntimeError(format!("Failed to create source: {}", e))
            })?)
            .map_err(|e| PastaError::RuneRuntimeError(format!("Failed to insert source: {}", e)))?;

        let unit = rune::prepare(&mut sources)
            .with_context(&context)
            .build()
            .map_err(|e| PastaError::RuneRuntimeError(format!("Failed to compile Rune: {}", e)))?;

        // Step 5: Construct PastaEngine with all fields
        Ok(Self {
            unit: Arc::new(unit),
            runtime,
            label_table,
            cache,
            persistence_path,
        })
    }

    /// Build execution context with persistence path.
    fn build_execution_context(&self) -> Result<rune::Value> {
        let mut ctx = HashMap::new();

        let path_str = if let Some(ref path) = self.persistence_path {
            path.to_string_lossy().to_string()
        } else {
            String::new()
        };

        ctx.insert("persistence_path".to_string(), path_str.clone());

        tracing::debug!(
            persistence_path = %path_str,
            "[PastaEngine::build_execution_context] Building execution context"
        );

        rune::to_value(ctx)
            .map_err(|e| PastaError::RuneRuntimeError(format!("Failed to build context: {}", e)))
    }

    /// Register labels from AST into label table.
    fn register_labels(
        label_table: &mut LabelTable,
        labels: &[LabelDef],
        parent_name: Option<&str>,
    ) -> Result<()> {
        // Track label counters for generating unique function names for duplicates
        let mut label_counters: HashMap<String, usize> = HashMap::new();

        for label in labels {
            // Get the counter for this label name
            let counter = label_counters.entry(label.name.clone()).or_insert(0);
            let fn_name = Self::generate_fn_name_with_counter(label, parent_name, *counter);
            *counter += 1;

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

    /// Generate a Rune function name for a label with a counter for duplicates.
    fn generate_fn_name_with_counter(
        label: &LabelDef,
        parent_name: Option<&str>,
        counter: usize,
    ) -> String {
        let sanitize = |name: &str| name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");

        let base_name = match label.scope {
            LabelScope::Global => sanitize(&label.name),
            LabelScope::Local => {
                if let Some(parent) = parent_name {
                    format!("{}__{}", sanitize(parent), sanitize(&label.name))
                } else {
                    sanitize(&label.name)
                }
            }
        };

        // Append counter if this is a duplicate (counter > 0)
        if counter > 0 {
            format!("{}_{}", base_name, counter)
        } else {
            base_name
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

        // Build execution context
        let context = self.build_execution_context()?;

        // Execute and get a generator
        let execution = vm
            .execute(hash, (context,))
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
                    let event: ScriptEvent = rune::from_value(value).map_err(|e| {
                        PastaError::RuneRuntimeError(format!(
                            "Failed to convert yielded value: {}",
                            e
                        ))
                    })?;
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

    /// List all labels (global + local).
    pub fn list_labels(&self) -> Vec<String> {
        self.label_table.list_all_labels()
    }

    /// List only global labels.
    pub fn list_global_labels(&self) -> Vec<String> {
        self.label_table.list_global_labels()
    }

    /// Find labels matching an event naming convention.
    ///
    /// Event labels follow the pattern:
    /// - `On<EventName>` (e.g., `OnClick`, `OnDoubleClick`, `OnStartup`)
    /// - Case-insensitive matching
    ///
    /// # Arguments
    ///
    /// * `event_name` - The event name to search for (without "On" prefix)
    ///
    /// # Returns
    ///
    /// A vector of label names that match the event pattern.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pasta::PastaEngine;
    /// let script = r#"
    /// ＊OnClick
    ///     さくら：クリックされました！
    ///
    /// ＊OnDoubleClick
    ///     さくら：ダブルクリック！
    /// "#;
    /// let engine = PastaEngine::new(script)?;
    ///
    /// let handlers = engine.find_event_handlers("Click");
    /// assert!(handlers.contains(&"OnClick".to_string()));
    /// # Ok::<(), pasta::PastaError>(())
    /// ```
    pub fn find_event_handlers(&self, event_name: &str) -> Vec<String> {
        let target_pattern = format!("On{}", event_name);
        let target_lower = target_pattern.to_lowercase();

        self.label_table
            .label_names()
            .into_iter()
            .filter(|name| name.to_lowercase() == target_lower)
            .collect()
    }

    /// Execute an event by name, finding and calling appropriate event handlers.
    ///
    /// This method looks for labels with the naming convention `On<EventName>` and
    /// executes them. If multiple handlers exist, one is randomly selected.
    ///
    /// # Arguments
    ///
    /// * `event_name` - The event name (without "On" prefix, e.g., "Click", "Startup")
    /// * `params` - Optional event parameters to pass to the handler
    ///
    /// # Returns
    ///
    /// A vector of `ScriptEvent`s generated by the event handler, or an empty
    /// vector if no handler is found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pasta::PastaEngine;
    /// # use std::collections::HashMap;
    /// let script = r#"
    /// ＊OnClick
    ///     さくら：クリックされました！
    /// "#;
    /// let mut engine = PastaEngine::new(script)?;
    ///
    /// let events = engine.on_event("Click", HashMap::new())?;
    /// # Ok::<(), pasta::PastaError>(())
    /// ```
    pub fn on_event(
        &mut self,
        event_name: &str,
        params: HashMap<String, String>,
    ) -> Result<Vec<ScriptEvent>> {
        // Find matching event handlers
        let handlers = self.find_event_handlers(event_name);

        if handlers.is_empty() {
            // No handler found - not an error, just return empty events
            return Ok(Vec::new());
        }

        // If multiple handlers exist, randomly select one
        // The label_table will handle the selection logic
        let handler_name = format!("On{}", event_name);

        // Execute the handler with filters (if provided as params)
        self.execute_label_with_filters(&handler_name, &params)
    }

    /// Fire a custom event and return the FireEvent script event.
    ///
    /// This is a convenience method that creates a `ScriptEvent::FireEvent`
    /// to be yielded by scripts.
    ///
    /// # Arguments
    ///
    /// * `event_name` - The name of the event to fire
    /// * `params` - Key-value parameters for the event
    ///
    /// # Returns
    ///
    /// A `ScriptEvent::FireEvent` that can be yielded or processed.
    ///
    /// # Example
    ///
    /// This would typically be called from within Rune scripts via the
    /// standard library `fire_event` function.
    pub fn create_fire_event(event_name: String, params: Vec<(String, String)>) -> ScriptEvent {
        ScriptEvent::FireEvent { event_name, params }
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

    // ============================================================================
    // Task 7: Event Handling Tests
    // ============================================================================

    #[test]
    fn test_find_event_handlers_basic() {
        let script = r#"
＊OnClick
    さくら：クリックされました！

＊OnDoubleClick
    さくら：ダブルクリック！

＊通常のラベル
    さくら：これはイベントハンドラではありません
"#;
        let engine = PastaEngine::new(script).unwrap();

        // Test finding Click event handler
        let click_handlers = engine.find_event_handlers("Click");
        assert_eq!(click_handlers.len(), 1);
        assert!(click_handlers.contains(&"OnClick".to_string()));

        // Test finding DoubleClick event handler
        let dblclick_handlers = engine.find_event_handlers("DoubleClick");
        assert_eq!(dblclick_handlers.len(), 1);
        assert!(dblclick_handlers.contains(&"OnDoubleClick".to_string()));

        // Test finding non-existent event handler
        let nonexistent = engine.find_event_handlers("NonExistent");
        assert_eq!(nonexistent.len(), 0);
    }

    #[test]
    fn test_find_event_handlers_case_insensitive() {
        let script = r#"
＊OnStartup
    さくら：起動しました！
"#;
        let engine = PastaEngine::new(script).unwrap();

        // Test case-insensitive matching
        let handlers1 = engine.find_event_handlers("Startup");
        let handlers2 = engine.find_event_handlers("startup");
        let handlers3 = engine.find_event_handlers("STARTUP");

        assert_eq!(handlers1.len(), 1);
        assert_eq!(handlers2.len(), 1);
        assert_eq!(handlers3.len(), 1);
        assert!(handlers1.contains(&"OnStartup".to_string()));
    }

    #[test]
    fn test_on_event_executes_handler() {
        let script = r#"
＊OnClick
    さくら：クリックありがとう！
"#;
        let mut engine = PastaEngine::new(script).unwrap();

        // Execute Click event
        let events = engine.on_event("Click", HashMap::new()).unwrap();

        // Should have events (ChangeSpeaker + Talk)
        assert!(events.len() >= 2);

        // Check that we have a ChangeSpeaker event
        let has_change_speaker = events
            .iter()
            .any(|e| matches!(e, ScriptEvent::ChangeSpeaker { name } if name == "さくら"));
        assert!(
            has_change_speaker,
            "Expected ChangeSpeaker event for さくら"
        );

        // Check that we have a Talk event with the content
        let has_talk = events.iter().any(|e| {
            if let ScriptEvent::Talk {
                speaker: _,
                content,
            } = e
            {
                content
                    .iter()
                    .any(|c| matches!(c, crate::ir::ContentPart::Text(s) if s.contains("クリック")))
            } else {
                false
            }
        });
        assert!(has_talk, "Expected Talk event with 'クリック' content");
    }

    #[test]
    fn test_on_event_no_handler_returns_empty() {
        let script = r#"
＊通常のラベル
    さくら：こんにちは
"#;
        let mut engine = PastaEngine::new(script).unwrap();

        // Try to execute non-existent event
        let events = engine.on_event("NonExistent", HashMap::new()).unwrap();

        // Should return empty vector, not error
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_on_event_with_multiple_handlers() {
        let script = r#"
＊OnClick
    さくら：クリック1
    
＊OnClick
    さくら：クリック2
"#;
        let mut engine = PastaEngine::new(script).unwrap();

        // Execute event - should select one of the handlers randomly
        let events = engine.on_event("Click", HashMap::new()).unwrap();

        // Should have events from one of the handlers
        assert!(events.len() >= 2);
    }

    #[test]
    fn test_on_event_with_attributes() {
        let script = r#"
＊OnClick
    ＠time：morning
    さくら：おはようございます！

＊OnClick
    ＠time：evening
    さくら：こんばんは！
"#;
        let mut engine = PastaEngine::new(script).unwrap();

        // Execute with morning filter
        let mut filters = HashMap::new();
        filters.insert("time".to_string(), "morning".to_string());
        let events = engine.on_event("Click", filters).unwrap();

        // Should execute the morning handler
        assert!(events.len() >= 2);
        let has_morning = events.iter().any(|e| {
            if let ScriptEvent::Talk { content, .. } = e {
                content
                    .iter()
                    .any(|c| matches!(c, crate::ir::ContentPart::Text(s) if s.contains("おはよう")))
            } else {
                false
            }
        });
        assert!(has_morning, "Expected morning greeting");
    }

    #[test]
    fn test_create_fire_event() {
        let event_name = "CustomEvent".to_string();
        let params = vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ];

        let event = PastaEngine::create_fire_event(event_name.clone(), params.clone());

        match event {
            ScriptEvent::FireEvent {
                event_name: name,
                params: p,
            } => {
                assert_eq!(name, event_name);
                assert_eq!(p, params);
            }
            _ => panic!("Expected FireEvent"),
        }
    }

    #[test]
    fn test_event_naming_convention() {
        // Test that labels with "On" prefix are recognized as event handlers
        let script = r#"
＊OnStartup
    さくら：起動イベント

＊OnShutdown
    さくら：終了イベント

＊OnUserInput
    さくら：ユーザー入力イベント

＊NotAnEvent
    さくら：通常ラベル
"#;
        let engine = PastaEngine::new(script).unwrap();

        // Should find all "On*" labels
        assert!(!engine.find_event_handlers("Startup").is_empty());
        assert!(!engine.find_event_handlers("Shutdown").is_empty());
        assert!(!engine.find_event_handlers("UserInput").is_empty());

        // Should not find "NotAnEvent"
        assert!(engine.find_event_handlers("NotAnEvent").is_empty());
    }

    #[test]
    fn test_event_integration_with_label_execution() {
        // Test that events can be fired from within scripts
        let script = r#"
＊test
    さくら：イベントを発火します
"#;
        let mut engine = PastaEngine::new(script).unwrap();

        // Execute label
        let events = engine.execute_label("test").unwrap();

        // Verify basic execution works
        assert!(events.len() >= 2);
    }

    #[test]
    fn test_multiple_event_types() {
        // Test engine with multiple different event types
        let script = r#"
＊OnClick
    さくら：クリックイベント

＊OnDoubleClick
    さくら：ダブルクリックイベント

＊OnStartup
    さくら：起動イベント
    
＊OnShutdown
    さくら：終了イベント
"#;
        let mut engine = PastaEngine::new(script).unwrap();

        // Test each event type
        let click_events = engine.on_event("Click", HashMap::new()).unwrap();
        assert!(!click_events.is_empty());

        let dblclick_events = engine.on_event("DoubleClick", HashMap::new()).unwrap();
        assert!(!dblclick_events.is_empty());

        let startup_events = engine.on_event("Startup", HashMap::new()).unwrap();
        assert!(!startup_events.is_empty());

        let shutdown_events = engine.on_event("Shutdown", HashMap::new()).unwrap();
        assert!(!shutdown_events.is_empty());
    }

    #[test]
    fn test_label_lookup_performance_many_labels() {
        // Test O(1) lookup performance with many labels
        let mut script = String::new();
        for i in 0..100 {
            script.push_str(&format!("＊label{}\n", i));
            script.push_str("    さくら：テスト\n\n");
        }

        let engine = PastaEngine::new(&script).unwrap();

        // All labels should be found instantly
        for i in 0..100 {
            let label_name = format!("label{}", i);
            assert!(engine.has_label(&label_name));
        }
    }

    #[test]
    fn test_duplicate_labels_grouping() {
        // Test that duplicate labels are properly grouped
        let script = r#"
＊greeting
    さくら：こんにちは

＊greeting
    さくら：やあ

＊greeting
    さくら：おはよう
"#;

        let mut engine = PastaEngine::new(script).unwrap();

        // Execute the label multiple times - should work with random selection
        for _ in 0..10 {
            let events = engine.execute_label("greeting").unwrap();
            assert!(!events.is_empty());
        }
    }

    #[test]
    fn test_label_lookup_nonexistent() {
        let script = r#"
＊test
    さくら：テスト
"#;
        let engine = PastaEngine::new(script).unwrap();

        // Nonexistent label should be found quickly (O(1))
        assert!(!engine.has_label("nonexistent"));
    }

    #[test]
    fn test_build_execution_context_with_path() {
        use std::path::PathBuf;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let script = r#"
＊test
    さくら：Hello
"#;

        let engine = PastaEngine::new_with_persistence(script, temp_dir.path()).unwrap();
        let context = engine.build_execution_context().unwrap();

        // Context should be a HashMap-like structure
        let map: std::collections::HashMap<String, String> = rune::from_value(context).unwrap();
        assert!(map.contains_key("persistence_path"));
        assert!(!map["persistence_path"].is_empty());
    }

    #[test]
    fn test_build_execution_context_without_path() {
        let script = r#"
＊test
    さくら：Hello
"#;

        let engine = PastaEngine::new(script).unwrap();
        let context = engine.build_execution_context().unwrap();

        let map: std::collections::HashMap<String, String> = rune::from_value(context).unwrap();
        assert!(map.contains_key("persistence_path"));
        assert_eq!(map["persistence_path"], "");
    }

    #[test]
    fn test_validate_persistence_path_nonexistent() {
        let result =
            PastaEngine::validate_persistence_path(std::path::Path::new("/nonexistent/directory"));
        assert!(result.is_err());
        if let Err(PastaError::PersistenceDirectoryNotFound { .. }) = result {
            // Expected error
        } else {
            panic!("Expected PersistenceDirectoryNotFound error");
        }
    }

    #[test]
    fn test_validate_persistence_path_file() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let result = PastaEngine::validate_persistence_path(temp_file.path());
        assert!(result.is_err());
        if let Err(PastaError::InvalidPersistencePath { .. }) = result {
            // Expected error
        } else {
            panic!("Expected InvalidPersistencePath error");
        }
    }
}
