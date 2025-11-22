//! Common infrastructure for ECS components
//!
//! This module provides domain-agnostic utilities that can be reused across
//! different ECS subsystems, such as generic hierarchical propagation functions.

pub mod tree_system;
pub use tree_system::*;
