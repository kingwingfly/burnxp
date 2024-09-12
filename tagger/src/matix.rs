use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

use crate::sort::CompareResult;

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Matrix<K>
where
    K: Eq + Hash + Clone,
{
    pub inner: HashMap<K, HashMap<K, CompareResult>>,
}

impl<K> Matrix<K>
where
    K: Eq + Hash + Clone,
{
    pub(crate) fn insert(&mut self, k1: K, k2: K, v: CompareResult) {
        if k1 == k2 {
            return;
        }
        let line2 = self.inner.entry(k2.clone()).or_default();
        line2.insert(k1.clone(), v.reverse());
        let line1 = self.inner.entry(k1.clone()).or_default();
        line1.insert(k2.clone(), v);
        let keys = line1
            .iter()
            .filter(|(_, v)| **v == CompareResult::Same)
            .map(|(k, _)| k.clone())
            .collect::<Vec<_>>();
        for k in keys {
            self.insert(k, k2.clone(), v);
        }
    }

    pub(crate) fn get(&self, k1: &K, k2: &K) -> Option<&CompareResult> {
        if k1 == k2 {
            return Some(&CompareResult::Same);
        }
        self.inner.get(k1)?.get(k2)
    }
}
