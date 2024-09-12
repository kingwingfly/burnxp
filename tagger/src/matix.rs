use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

pub(crate) trait Reflexivity {
    fn reverse(&self) -> Self;
    fn zero() -> Self;
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Matrix<K, V>
where
    K: Eq + Hash + Clone,
    V: Reflexivity + PartialEq + Copy,
{
    pub inner: HashMap<K, HashMap<K, V>>,
}

impl<K, V> Matrix<K, V>
where
    K: Eq + Hash + Clone,
    V: Reflexivity + PartialEq + Copy + Clone,
{
    pub(crate) fn insert(&mut self, k1: K, k2: K, v: V) {
        if k1 == k2 {
            return;
        }
        let line2 = self.inner.entry(k2.clone()).or_default();
        line2.insert(k1.clone(), v.reverse());
        let line1 = self.inner.entry(k1.clone()).or_default();
        line1.insert(k2.clone(), v);
        let keys = line1
            .iter()
            .filter(|(_, v)| **v == V::zero())
            .map(|(k, _)| k.clone())
            .collect::<Vec<_>>();
        for k in keys {
            self.insert(k, k2.clone(), v);
        }
    }

    pub(crate) fn get(&self, k1: &K, k2: &K) -> Option<V> {
        if k1 == k2 {
            return Some(V::zero());
        }
        self.inner.get(k1)?.get(k2).cloned()
    }
}
