use std::collections::BTreeMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "bincode")]
use bincode::{Decode, Encode};

/// Struct to store original labels of shuffled addresses
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bincode", derive(Encode, Decode))]
pub struct DebugInfo {
    pub prog: BTreeMap<usize, String>,
    pub mem: BTreeMap<usize, String>,
}
