use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::sort::{CompareResult, OrdPaths};

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
    pub(crate) fn insert(&mut self, k1: &OrdPaths, k2: &OrdPaths, v: CompareResult) {
        // To avoid cloning K leading to double free, do not use entry.
        let line2 = self.map.entry(k2.clone()).or_default();
        line2.insert(k1.clone(), v.reverse());
        let line1 = self.map.entry(k1.clone()).or_default();
        let keys = line1
            .iter()
            .filter(|(_, v)| **v == CompareResult::ZERO)
            .map(|(k, _)| k.clone())
            .collect::<Vec<_>>();
        line1.insert(k2.clone(), v);
        for k in keys {
            self.insert(&k, k2, v);
        }
    }

    pub(crate) fn get(&self, k1: &OrdPaths, k2: &OrdPaths) -> Option<CompareResult> {
        self.map.get(k1)?.get(k2).cloned()
    }
}
