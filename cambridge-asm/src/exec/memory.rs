// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{PasmError, PasmResult};
use std::{
    collections::btree_map::{BTreeMap, Iter},
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "bincode")]
use bincode::{Decode, Encode};

/// Struct representing a single block of RAM
#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bincode", derive(Encode, Decode))]
pub struct MemEntry {
    pub literal: usize,
    pub address: Option<usize>,
}

impl MemEntry {
    pub fn new(val: usize) -> Self {
        Self {
            literal: val,
            address: None,
        }
    }

    pub fn as_address(&self) -> Option<usize> {
        self.address
    }
}

impl From<usize> for MemEntry {
    fn from(x: usize) -> Self {
        MemEntry::new(x)
    }
}

impl Display for MemEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(a) = self.address {
            f.write_fmt(format_args!("{{ {}, addr: {a} }}", self.literal))
        } else {
            f.write_fmt(format_args!("{}", self.literal))
        }
    }
}

/// Struct providing random-access memory (RAM)
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bincode", derive(Encode, Decode))]
#[repr(transparent)]
pub struct Memory(BTreeMap<usize, MemEntry>);

impl Memory {
    pub fn new(mem: BTreeMap<usize, MemEntry>) -> Self {
        Self(mem)
    }

    pub fn iter(&self) -> Iter<usize, MemEntry> {
        self.0.iter()
    }

    pub fn get(&self, addr: &usize) -> Result<usize, PasmError> {
        let x = self.0.get(addr).ok_or(PasmError::InvalidMemoryLoc(*addr))?;
        Ok(x.literal)
    }

    pub fn get_address(&self, addr: &usize) -> PasmResult<usize> {
        self.0
            .get(addr)
            .ok_or(PasmError::InvalidMemoryLoc(*addr))?
            .as_address()
            .ok_or(PasmError::InvalidIndirectAddress(*addr))
    }

    pub fn write(&mut self, addr: &usize, dat: usize) -> PasmResult {
        let x = self
            .0
            .get_mut(addr)
            .ok_or(PasmError::InvalidMemoryLoc(*addr))?;

        if x.literal <= dat {
            let offset = dat - x.literal;
            x.literal = dat;
            if let Some(a) = x.address {
                x.address = Some(a + offset);
            };
        } else {
            let offset = x.literal - dat;
            x.literal = dat;
            if let Some(a) = x.address {
                x.address = Some(a - offset);
            }
        }

        Ok(())
    }
}

impl<T> From<T> for Memory
where
    T: Into<BTreeMap<usize, MemEntry>>,
{
    fn from(x: T) -> Self {
        Self(x.into())
    }
}
