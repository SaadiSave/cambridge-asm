use super::{PasmError, PasmResult};
use std::collections::btree_map::Iter;
use std::fmt::Display;
use std::{
    collections::btree_map::BTreeMap,
    fmt::{Debug, Formatter, Result as FmtResult},
};

#[derive(Debug, Default, PartialEq)]
pub struct MemEntry {
    pub literal: usize,
    pub address: Option<usize>,
}

impl MemEntry {
    pub fn new(val: usize) -> MemEntry {
        MemEntry {
            literal: val,
            address: None,
        }
    }

    pub fn as_address(&self) -> usize {
        self.address.unwrap()
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

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct Memory(BTreeMap<usize, MemEntry>);

impl Memory {
    pub fn new(mem: BTreeMap<usize, MemEntry>) -> Self {
        Self(mem)
    }

    pub fn iter(&self) -> Iter<usize, MemEntry> {
        self.0.iter()
    }

    pub fn get(&self, loc: &usize) -> Result<usize, PasmError> {
        let x = self
            .0
            .get(loc)
            .ok_or_else(|| PasmError::InvalidMemoryLoc(format!("{loc:?}")))?;
        Ok(x.literal)
    }

    pub fn get_address(&self, loc: &usize) -> Result<usize, PasmError> {
        let x = self
            .0
            .get(loc)
            .ok_or_else(|| PasmError::InvalidMemoryLoc(format!("{loc:?}")))?;
        Ok(x.as_address())
    }

    pub fn write(&mut self, loc: &usize, dat: usize) -> PasmResult {
        let x = self
            .0
            .get_mut(loc)
            .ok_or_else(|| PasmError::InvalidMemoryLoc(format!("{loc:?}")))?;

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
