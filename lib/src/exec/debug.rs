use std::collections::BTreeMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::parse::Span;

/// Struct to store original labels of shuffled addresses
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DebugInfo {
    /// Original labels of instructions
    pub prog: BTreeMap<usize, String>,
    /// Original labels of memory entries
    pub mem: BTreeMap<usize, String>,
    /// Portions of source recognised as instructions
    pub inst_spans: Vec<Span>,
}
