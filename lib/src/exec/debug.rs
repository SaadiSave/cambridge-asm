use std::collections::BTreeMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "bincode")]
use bincode::{Decode, Encode};

use crate::parse::Span;

/// Struct to store original labels of shuffled addresses
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bincode", derive(Encode, Decode))]
pub struct DebugInfo {
    /// Orginal labels of instructions
    pub prog: BTreeMap<usize, String>,
    /// Orignal labels of memory entries
    pub mem: BTreeMap<usize, String>,
    /// Portions of source recognised as instructions
    pub inst_spans: Vec<Span>,
}
