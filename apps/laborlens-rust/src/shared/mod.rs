//! Shared kernel for identifiers and artifact references whose meaning must
//! stay stable across bounded contexts.
//!
//! Keep this module small. Context-specific rules belong in the context domain
//! modules instead of becoming shared convenience code.

use serde::{Deserialize, Serialize};
use std::fmt;

pub mod db;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RunId(String);

impl RunId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RunId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
