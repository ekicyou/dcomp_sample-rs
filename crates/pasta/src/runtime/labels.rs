//! Label management for Pasta scripts.
//!
//! This module provides label registration, lookup, and random selection
//! for labels with the same name.

use crate::runtime::random::RandomSelector;
use crate::{LabelScope, PastaError};
use std::collections::HashMap;

/// Information about a single label.
#[derive(Debug, Clone)]
pub struct LabelInfo {
    /// Label name.
    pub name: String,
    /// Label scope.
    pub scope: LabelScope,
    /// Attributes for filtering.
    pub attributes: HashMap<String, String>,
    /// Generated function name in Rune code.
    pub fn_name: String,
    /// Parent label name (for local labels).
    pub parent: Option<String>,
}

/// Label table for managing script labels.
pub struct LabelTable {
    /// Map from label name to list of label infos (multiple labels can share a name).
    labels: HashMap<String, Vec<LabelInfo>>,
    /// Map from label name to execution history (for sequential selection).
    history: HashMap<String, Vec<usize>>,
    /// Random selector for label selection.
    random_selector: Box<dyn RandomSelector>,
}

impl LabelTable {
    /// Create a new label table with default random selector.
    pub fn new(random_selector: Box<dyn RandomSelector>) -> Self {
        Self {
            labels: HashMap::new(),
            history: HashMap::new(),
            random_selector,
        }
    }

    /// Create a label table from a transpiler's LabelRegistry.
    ///
    /// This converts the LabelRegistry (used during transpilation) into
    /// a LabelTable (used during runtime).
    pub fn from_label_registry(
        registry: crate::transpiler::LabelRegistry,
        random_selector: Box<dyn RandomSelector>,
    ) -> Self {
        let mut labels: HashMap<String, Vec<LabelInfo>> = HashMap::new();

        // Convert each label from the registry
        for (_, registry_info) in registry.iter() {
            let runtime_info = LabelInfo {
                name: registry_info.name.clone(),
                scope: if registry_info.parent.is_some() {
                    LabelScope::Local
                } else {
                    LabelScope::Global
                },
                attributes: registry_info.attributes.clone(),
                fn_name: registry_info.fn_name.clone(),
                parent: registry_info.parent.clone(),
            };

            labels
                .entry(runtime_info.name.clone())
                .or_insert_with(Vec::new)
                .push(runtime_info);
        }

        Self {
            labels,
            history: HashMap::new(),
            random_selector,
        }
    }

    /// Register a label.
    pub fn register(&mut self, info: LabelInfo) {
        self.labels
            .entry(info.name.clone())
            .or_insert_with(Vec::new)
            .push(info);
    }

    /// Find a label by name, with optional attribute filters.
    ///
    /// If multiple labels match, selects one randomly.
    /// Returns the function name to call in Rune.
    pub fn find_label(
        &mut self,
        name: &str,
        filters: &HashMap<String, String>,
    ) -> Result<String, PastaError> {
        let candidates = self
            .labels
            .get(name)
            .ok_or_else(|| PastaError::LabelNotFound {
                label: name.to_string(),
            })?;

        // Filter by attributes
        let matching: Vec<&LabelInfo> = candidates
            .iter()
            .filter(|label| {
                filters
                    .iter()
                    .all(|(key, value)| label.attributes.get(key) == Some(value))
            })
            .collect();

        if matching.is_empty() {
            return Err(PastaError::LabelNotFound {
                label: format!("{} (with filters {:?})", name, filters),
            });
        }

        // If only one match, return it
        if matching.len() == 1 {
            return Ok(matching[0].fn_name.clone());
        }

        // Multiple matches: select randomly
        let selected_idx = self
            .random_selector
            .select_index(matching.len())
            .ok_or_else(|| PastaError::LabelNotFound {
                label: name.to_string(),
            })?;

        let selected = matching[selected_idx];

        // Record selection in history
        self.history
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(
                candidates
                    .iter()
                    .position(|l| l.fn_name == selected.fn_name)
                    .unwrap(),
            );

        Ok(selected.fn_name.clone())
    }

    /// Get execution history for a label.
    pub fn get_history(&self, name: &str) -> Option<&Vec<usize>> {
        self.history.get(name)
    }

    /// Clear execution history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Get all labels with a given name.
    pub fn get_labels(&self, name: &str) -> Option<&Vec<LabelInfo>> {
        self.labels.get(name)
    }

    /// Get all label names.
    pub fn label_names(&self) -> Vec<String> {
        self.labels.keys().cloned().collect()
    }

    /// Check if a label exists.
    pub fn has_label(&self, name: &str) -> bool {
        self.labels.contains_key(name)
    }

    /// List all labels (global + local).
    pub fn list_all_labels(&self) -> Vec<String> {
        let mut all_labels = Vec::new();
        for (name, infos) in &self.labels {
            for (idx, _) in infos.iter().enumerate() {
                if infos.len() > 1 {
                    all_labels.push(format!("{}_{}", name, idx));
                } else {
                    all_labels.push(name.clone());
                }
            }
        }
        all_labels.sort();
        all_labels
    }

    /// List only global labels.
    pub fn list_global_labels(&self) -> Vec<String> {
        let mut global_labels = Vec::new();
        for (name, infos) in &self.labels {
            for (idx, info) in infos.iter().enumerate() {
                if info.scope == LabelScope::Global {
                    if infos.len() > 1 {
                        global_labels.push(format!("{}_{}", name, idx));
                    } else {
                        global_labels.push(name.clone());
                    }
                }
            }
        }
        global_labels.sort();
        global_labels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::random::MockRandomSelector;

    fn create_test_label(name: &str, fn_name: &str) -> LabelInfo {
        LabelInfo {
            name: name.to_string(),
            scope: LabelScope::Global,
            attributes: HashMap::new(),
            fn_name: fn_name.to_string(),
            parent: None,
        }
    }

    #[test]
    fn test_label_table_register_and_find() {
        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = LabelTable::new(selector);

        table.register(create_test_label("greeting", "greeting_1"));

        let result = table.find_label("greeting", &HashMap::new());
        assert_eq!(result.unwrap(), "greeting_1");
    }

    #[test]
    fn test_label_table_not_found() {
        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = LabelTable::new(selector);

        let result = table.find_label("nonexistent", &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_label_table_multiple_labels() {
        let selector = Box::new(MockRandomSelector::new(vec![1, 0]));
        let mut table = LabelTable::new(selector);

        table.register(create_test_label("greeting", "greeting_1"));
        table.register(create_test_label("greeting", "greeting_2"));

        // First call should select index 1 (greeting_2)
        let result1 = table.find_label("greeting", &HashMap::new());
        assert_eq!(result1.unwrap(), "greeting_2");

        // Second call should select index 0 (greeting_1)
        let result2 = table.find_label("greeting", &HashMap::new());
        assert_eq!(result2.unwrap(), "greeting_1");
    }

    #[test]
    fn test_label_table_with_attributes() {
        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = LabelTable::new(selector);

        let mut label1 = create_test_label("greeting", "greeting_morning");
        label1
            .attributes
            .insert("time".to_string(), "morning".to_string());

        let mut label2 = create_test_label("greeting", "greeting_evening");
        label2
            .attributes
            .insert("time".to_string(), "evening".to_string());

        table.register(label1);
        table.register(label2);

        let mut filters = HashMap::new();
        filters.insert("time".to_string(), "morning".to_string());

        let result = table.find_label("greeting", &filters);
        assert_eq!(result.unwrap(), "greeting_morning");
    }

    #[test]
    fn test_label_table_history() {
        let selector = Box::new(MockRandomSelector::new(vec![0, 1]));
        let mut table = LabelTable::new(selector);

        table.register(create_test_label("test", "test_1"));
        table.register(create_test_label("test", "test_2"));

        table.find_label("test", &HashMap::new()).unwrap();
        table.find_label("test", &HashMap::new()).unwrap();

        let history = table.get_history("test").unwrap();
        assert_eq!(history, &vec![0, 1]);
    }

    #[test]
    fn test_label_table_has_label() {
        let selector = Box::new(MockRandomSelector::new(vec![0]));
        let mut table = LabelTable::new(selector);

        table.register(create_test_label("test", "test_1"));

        assert!(table.has_label("test"));
        assert!(!table.has_label("nonexistent"));
    }
}
