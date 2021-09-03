use super::{PasmError, PasmResult};
use std::{
    collections::btree_map::{BTreeMap, Iter},
    fmt::{Debug, Formatter, Result as FmtResult},
};

#[derive(Debug)]
#[repr(transparent)]
pub struct Memory<K: Ord, V>(BTreeMap<K, V>);

impl<K: Ord, V> Memory<K, V> {
    pub fn new(data: BTreeMap<K, V>) -> Memory<K, V> {
        Memory(data)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> Iter<K, V> {
        self.0.iter()
    }
}

impl<K: Ord + Debug, V: Clone> Memory<K, V> {
    pub fn get(&self, loc: &K) -> Result<V, PasmError> {
        let x = self
            .0
            .get(loc)
            .ok_or_else(|| PasmError::InvalidMemoryLoc(format!("{:?}", loc)))?;
        Ok(x.clone())
    }

    pub fn write(&mut self, loc: &K, dat: V) -> PasmResult {
        let x = self
            .0
            .get_mut(loc)
            .ok_or_else(|| PasmError::InvalidMemoryLoc(format!("{:?}", loc)))?;
        *x = dat;

        Ok(())
    }
}

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

impl Debug for MemEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(a) = self.address {
            f.write_fmt(format_args!("{}, addr: {}", self.literal, a))
        } else {
            f.write_fmt(format_args!("{}", self.literal))
        }
    }
}

impl Memory<usize, MemEntry> {
    pub fn get(&self, loc: &usize) -> Result<usize, PasmError> {
        let x = self
            .0
            .get(loc)
            .ok_or_else(|| PasmError::InvalidMemoryLoc(format!("{:?}", loc)))?;
        Ok(x.literal)
    }

    pub fn get_address(&self, loc: &usize) -> Result<usize, PasmError> {
        let x = self
            .0
            .get(loc)
            .ok_or_else(|| PasmError::InvalidMemoryLoc(format!("{:?}", loc)))?;
        Ok(x.as_address())
    }

    pub fn write(&mut self, loc: &usize, dat: usize) -> PasmResult {
        let x = self
            .0
            .get_mut(loc)
            .ok_or_else(|| PasmError::InvalidMemoryLoc(format!("{:?}", loc)))?;

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
