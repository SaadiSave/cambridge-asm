// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{RtError, RtResult};
use std::{
    collections::btree_map::{BTreeMap, Iter},
    fmt::Debug,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Struct providing random-access memory (RAM)
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct Memory(BTreeMap<usize, usize>);

impl Memory {
    pub fn new(mem: BTreeMap<usize, usize>) -> Self {
        Self(mem)
    }

    pub fn iter(&self) -> Iter<usize, usize> {
        self.0.iter()
    }

    pub fn get(&self, addr: &usize) -> RtResult<&usize> {
        self.0.get(addr).ok_or(RtError::InvalidAddr(*addr))
    }

    pub fn get_mut(&mut self, addr: &usize) -> RtResult<&mut usize> {
        self.0.get_mut(addr).ok_or(RtError::InvalidAddr(*addr))
    }

    pub fn inner(&self) -> &BTreeMap<usize, usize> {
        &self.0
    }
}

impl<'a> IntoIterator for &'a Memory {
    type IntoIter = std::collections::btree_map::Iter<'a, usize, usize>;
    type Item = (&'a usize, &'a usize);
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> From<T> for Memory
where
    T: Into<BTreeMap<usize, usize>>,
{
    fn from(x: T) -> Self {
        Self(x.into())
    }
}
