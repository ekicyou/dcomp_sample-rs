//! PastaEngine - Main entry point for executing Pasta DSL scripts.
//!
//! This module provides the integrated engine that combines parsing, transpiling,
//! and runtime execution to provide a high-level API for running Pasta scripts.

use crate::{
    error::{PastaError, Result},
    ir::ScriptEvent,
    loader::{DirectoryLoader, ErrorLogWriter},
    parser::parse_file,
    runtime::{DefaultRandomSelector, LabelTable, RandomSelector},
    transpiler::Transpiler,
    PastaFile,
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
    /// Persistence directory path (optional).
    persistence_path: Option<PathBuf>,
}

impl PastaEngine {
    /// Create a new PastaEngine from script and persistence directories.
    ///
    /// This is the primary constructor for production use. It loads all `.pasta` files
    /// from the `dic/` directory and `main.rn` from the script root, following
    /// areka-P0-script-engine conventions.
    ///
    /// # Directory Structure
    ///
    /// ```text
    /// script_root/
    ///   ├── main.rn             # Rune entry point
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
    /// - `main.rn` not found
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
    pub fn new(script_root: impl AsRef<Path>, persistence_root: impl AsRef<Path>) -> Result<Self> {
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
            global_words: Vec::new(), // TODO: Merge global words from all files
            labels: all_labels,
            span: crate::parser::Span::new(1, 1, 1, 0),
        };

        // Step 5: Transpile merged AST to Rune source using two-pass transpiler
        let (rune_source, label_registry) = Transpiler::transpile_with_registry(&merged_ast)?;

        #[cfg(debug_assertions)]
        {
            eprintln!("=== Generated Rune Source (from directory) ===");
            eprintln!("{}", rune_source);
            eprintln!("===============================================");
        }

        // Step 6: Build Rune sources with main.rn
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

        // Add main.rn
        sources
            .insert(rune::Source::from_path(&loaded.main_rune).map_err(|e| {
                PastaError::RuneRuntimeError(format!("Failed to load main.rn: {}", e))
            })?)
            .map_err(|e| {
                PastaError::RuneRuntimeError(format!("Failed to insert main.rn: {}", e))
            })?;

        // Step 7: Compile Rune code
        let unit = rune::prepare(&mut sources)
            .with_context(&context)
            .build()
            .map_err(|e| PastaError::RuneCompileError(format!("Failed to compile Rune: {}", e)))?;

        // Step 8: Build label table from transpiler's label registry
        let label_table = LabelTable::from_label_registry(label_registry, random_selector);

        // Step 9: Validate persistence path
        let validated_persistence_path =
            Self::validate_persistence_path(persistence_root.as_ref())?;

        // Step 10: Construct engine
        Ok(Self {
            unit: Arc::new(unit),
            runtime,
            label_table,
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

        // Split fn_name into path components for Rune
        // fn_name format: "module_name::function_name"
        // Rune expects: ["module_name", "function_name"]
        let parts: Vec<&str> = fn_name.split("::").collect();
        let hash = rune::Hash::type_hash(&parts);

        // Build execution context
        let context = self.build_execution_context()?;

        // Execute and get a generator
        // Note: Generated functions expect (ctx, args) signature
        // args is currently an empty array for future argument support
        let args = rune::to_value(Vec::<rune::Value>::new()).map_err(|e| {
            PastaError::RuneRuntimeError(format!("Failed to create args array: {}", e))
        })?;

        let execution = vm
            .execute(hash, (context, args))
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
