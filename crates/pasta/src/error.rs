//! Error types for the Pasta script engine.

use thiserror::Error;

/// Result type alias for Pasta operations.
pub type Result<T> = std::result::Result<T, PastaError>;

/// Structured error type for Pasta script engine.
#[derive(Error, Debug)]
pub enum PastaError {
    /// Parse error with source location information.
    #[error("Parse error at {file}:{line}:{column}: {message}")]
    ParseError {
        file: String,
        line: usize,
        column: usize,
        message: String,
    },

    /// Label not found error.
    #[error("Label not found: {label}")]
    LabelNotFound { label: String },

    /// Function not found error.
    #[error("Function not found: {name}")]
    FunctionNotFound { name: String },

    /// Name conflict error (duplicate label or variable).
    #[error("Name conflict: '{name}' is already defined as {existing_kind}")]
    NameConflict { name: String, existing_kind: String },

    /// Rune compilation error.
    #[error("Rune compilation error: {0}")]
    RuneCompileError(String),

    /// Rune runtime error.
    #[error("Rune runtime error: {0}")]
    RuneRuntimeError(String),

    /// Rune VM error.
    #[error("Rune VM error: {0}")]
    VmError(#[from] rune::runtime::VmError),

    /// IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// pest parse error.
    #[error("Pest parse error: {0}")]
    PestError(String),

    /// Persistence directory not found.
    #[error("Persistence directory not found: {path}")]
    PersistenceDirectoryNotFound { path: String },

    /// Invalid persistence path.
    #[error("Invalid persistence path: {path}")]
    InvalidPersistencePath { path: String },

    /// Path must be absolute.
    #[error("Path must be absolute: {path}")]
    NotAbsolutePath { path: String },

    /// Directory not found.
    #[error("Directory not found: {path}")]
    DirectoryNotFound { path: String },

    /// Path is not a directory.
    #[error("Path is not a directory: {path}")]
    NotADirectory { path: String },

    /// Permission denied.
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    /// dic/ directory not found.
    #[error("dic/ directory not found in: {script_root}")]
    DicDirectoryNotFound { script_root: String },

    /// main.rune not found.
    #[error("main.rune not found in: {script_root}")]
    MainRuneNotFound { script_root: String },

    /// Multiple parse errors.
    #[error("Multiple parse errors ({} errors). See logs for details.", .errors.len())]
    MultipleParseErrors { errors: Vec<ParseErrorInfo> },
}

/// Individual parse error information for MultipleParseErrors.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseErrorInfo {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl PastaError {
    /// Create a new parse error with source location.
    pub fn parse_error(
        file: impl Into<String>,
        line: usize,
        column: usize,
        message: impl Into<String>,
    ) -> Self {
        PastaError::ParseError {
            file: file.into(),
            line,
            column,
            message: message.into(),
        }
    }

    /// Create a new label not found error.
    pub fn label_not_found(label: impl Into<String>) -> Self {
        PastaError::LabelNotFound {
            label: label.into(),
        }
    }

    /// Create a new function not found error.
    pub fn function_not_found(name: impl Into<String>) -> Self {
        PastaError::FunctionNotFound { name: name.into() }
    }

    /// Create a new name conflict error.
    pub fn name_conflict(name: impl Into<String>, existing_kind: impl Into<String>) -> Self {
        PastaError::NameConflict {
            name: name.into(),
            existing_kind: existing_kind.into(),
        }
    }

    /// Create a new pest parse error.
    pub fn pest_error(message: impl Into<String>) -> Self {
        PastaError::PestError(message.into())
    }
}

impl From<&PastaError> for Option<ParseErrorInfo> {
    fn from(e: &PastaError) -> Self {
        match e {
            PastaError::ParseError {
                file,
                line,
                column,
                message,
            } => Some(ParseErrorInfo {
                file: file.clone(),
                line: *line,
                column: *column,
                message: message.clone(),
            }),
            _ => None,
        }
    }
}
