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
    map: HashMap<OrdPaths, HashMap<PathBuf, CompareResult>>,
}

impl Matrix {
    pub(crate) fn insert(&mut self, p1: OrdPaths, p2: OrdPaths, v: CompareResult) {
        let line2 = self.map.entry(p2).or_default();
        line2.insert(p1[0].clone(), v.reverse());
        let line1 = self.map.entry(p1).or_default();
        let paths = line1
            .iter()
            .filter(|(_, v)| **v == CompareResult::ZERO)
            .map(|(k, _)| k.clone())
            .collect::<Vec<_>>();
        line1.insert(p2[0].clone(), v);
        for p in paths {
            let p1 = OrdPaths::new([p]);
            self.insert(p1, p2, v);
            // Safety: p1 comes from p, which is already in the map.
            // Thus p1 will never be inserted (i.e. only be used as query key), and is safe to be dropped.
            unsafe {
                p1.drop();
            }
        }
    }

    /// Remove the line and column of the given path.
    /// The memory of it will be freed.
    #[allow(unused)]
    pub(crate) fn remove(&mut self, p: OrdPaths) {
        self.map.remove(&p);
        for line in self.map.values_mut() {
            line.remove(&p[0]);
        }
        unsafe {
            p.drop();
        }
    }

    pub(crate) fn get(&self, p1: &OrdPaths, p2: &OrdPaths) -> Option<CompareResult> {
        self.map.get(p1)?.get(&p2[0]).cloned()
    }

    pub(crate) fn get_paths(&self, p: &PathBuf) -> Option<OrdPaths> {
        self.map.keys().find(|paths| &paths[0] == p).cloned()
    }
}
