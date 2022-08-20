//! This crate contains mocks and utilities for testing Abacus agents.

#![forbid(unsafe_code)]
#![cfg_attr(test, warn(missing_docs))]
#![warn(unused_extern_crates)]
#![forbid(where_clauses_object_safety)]

/// Mock contracts
pub mod mocks;

/// Testing utilities
pub mod test_utils;
