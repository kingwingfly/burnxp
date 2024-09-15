use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

use crate::ordpaths::{CompareResult, OrdPaths};

pub(crate) trait Reflexivity {
    /// Return the zero value of the current instance.
    /// self + Self::ZERO == self
    const ZERO: Self;
    /// Reverse the value of the current instance.
    /// self + self.reverse() == Self::zero()
    fn reverse(&self) -> Self;
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Matrix {
    map: HashMap<OrdPaths, HashMap<OrdPaths, CompareResult>>,
}

impl Matrix {
    pub(crate) fn insert(&mut self, k1: OrdPaths, k2: OrdPaths, v: CompareResult) {
        // To avoid cloning K leading to double free, do not use entry.
        let line2 = self.map.entry(k2).or_default();
        line2.insert(k1, v.reverse());
        let line1 = self.map.entry(k1).or_default();
        let keys = line1
            .iter()
            .filter(|(_, v)| **v == CompareResult::ZERO)
            .map(|(k, _)| *k)
            .collect::<Vec<_>>();
        line1.insert(k2, v);
        for k in keys {
            self.insert(k, k2, v);
        }
    }

    pub(crate) fn get(&self, k1: &OrdPaths, k2: &OrdPaths) -> Option<CompareResult> {
        self.map.get(k1)?.get(k2).cloned()
    }

    pub(crate) fn get_key(&self, k: &PathBuf) -> Option<OrdPaths> {
        self.map.keys().find(|key| &key[0] == k).cloned()
    }
}
