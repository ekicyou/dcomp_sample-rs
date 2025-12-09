//! Script generator for Pasta scripts.
//!
//! This module provides a wrapper around Rune generators for executing
//! Pasta scripts and yielding ScriptEvent IR.

use crate::error::PastaError;
use crate::ir::ScriptEvent;
use rune::runtime::{Generator, GeneratorState, Vm, VmResult};

/// State of the script generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptGeneratorState {
    /// Generator is running and can be resumed.
    Running,
    /// Generator is suspended (waiting for resume).
    Suspended,
    /// Generator has completed execution.
    Completed,
}

/// Script generator for executing Pasta scripts.
///
/// This wraps a Rune generator and provides a convenient interface for
/// stepping through script execution and yielding ScriptEvent IR.
pub struct ScriptGenerator {
    /// The underlying Rune generator.
    generator: Generator<Vm>,
    /// Current state of the generator.
    state: ScriptGeneratorState,
}

impl ScriptGenerator {
    /// Create a new script generator from a Rune generator.
    pub fn new(generator: Generator<Vm>) -> Self {
        Self {
            generator,
            state: ScriptGeneratorState::Running,
        }
    }

    /// Resume the generator and get the next ScriptEvent.
    ///
    /// Returns:
    /// - `Ok(Some(event))` if an event was yielded
    /// - `Ok(None)` if the generator completed
    /// - `Err(error)` if a runtime error occurred
    pub fn resume(&mut self) -> Result<Option<ScriptEvent>, PastaError> {
        if self.state == ScriptGeneratorState::Completed {
            return Ok(None);
        }

        // Create a unit value for resume
        let unit_value = rune::to_value(()).map_err(|e| {
            PastaError::RuneRuntimeError(format!("Failed to create unit value: {}", e))
        })?;

        match self.generator.resume(unit_value) {
            VmResult::Ok(GeneratorState::Yielded(value)) => {
                self.state = ScriptGeneratorState::Suspended;
                // Try to convert the Rune value to ScriptEvent
                let event: ScriptEvent = rune::from_value(value)
                    .map_err(|e| PastaError::RuneRuntimeError(format!("Failed to convert yielded value: {}", e)))?;
                Ok(Some(event))
            }
            VmResult::Ok(GeneratorState::Complete(_)) => {
                self.state = ScriptGeneratorState::Completed;
                Ok(None)
            }
            VmResult::Err(e) => {
                self.state = ScriptGeneratorState::Completed;
                Err(PastaError::VmError(e))
            }
        }
    }

    /// Resume the generator until completion and collect all events.
    ///
    /// This is a convenience method that calls `resume()` repeatedly
    /// until the generator completes or an error occurs.
    pub fn resume_all(&mut self) -> Result<Vec<ScriptEvent>, PastaError> {
        let mut events = Vec::new();

        while let Some(event) = self.resume()? {
            events.push(event);
        }

        Ok(events)
    }

    /// Get the current state of the generator.
    pub fn state(&self) -> ScriptGeneratorState {
        self.state
    }

    /// Check if the generator has completed.
    pub fn is_completed(&self) -> bool {
        self.state == ScriptGeneratorState::Completed
    }

    /// Skip the rest of the script (immediately complete).
    pub fn skip(&mut self) {
        self.state = ScriptGeneratorState::Completed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests are incomplete because we need a full Rune VM setup
    // to create actual generators. Integration tests will cover this.

    #[test]
    fn test_generator_state() {
        // This is a placeholder test - actual tests require Rune VM integration
        let state = ScriptGeneratorState::Running;
        assert_eq!(state, ScriptGeneratorState::Running);
        assert_ne!(state, ScriptGeneratorState::Completed);
    }

    #[test]
    fn test_generator_skip() {
        // This test demonstrates the skip functionality without requiring a real generator
        // In practice, we'd need to mock or create a real Rune generator
    }
}
