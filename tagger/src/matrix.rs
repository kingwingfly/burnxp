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
    pub(crate) fn insert(&mut self, p1: OrdPaths, p2: OrdPaths, v: CompareResult) {
        // To avoid cloning K leading to double free, do not use entry.
        let line2 = self.map.entry(p2).or_default();
        line2.insert(p1, v.reverse());
        let line1 = self.map.entry(p1).or_default();
        let paths = line1
            .iter()
            .filter(|(_, v)| **v == CompareResult::ZERO)
            .map(|(k, _)| *k)
            .collect::<Vec<_>>();
        line1.insert(p2, v);
        for p in paths {
            self.insert(p, p2, v);
        }
    }

    pub(crate) fn get(&self, p1: &OrdPaths, p2: &OrdPaths) -> Option<CompareResult> {
        self.map.get(p1)?.get(p2).cloned()
    }

    pub(crate) fn get_paths(&self, p: &PathBuf) -> Option<OrdPaths> {
        self.map.keys().find(|paths| &paths[0] == p).cloned()
    }
}
