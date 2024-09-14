use crate::event::{Event, CMPDISPATCH};
use crate::matrix::Reflexivity;
use rand::seq::SliceRandom as _;
use serde::{Deserialize, Serialize};
use std::cell::UnsafeCell;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Index;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub(crate) struct OrdPaths {
    paths: Rc<UnsafeCell<Vec<PathBuf>>>,
}

impl OrdPaths {
    pub(crate) fn new(paths: impl IntoIterator<Item = impl AsRef<Path>>) -> Self {
        let paths: Vec<PathBuf> = paths.into_iter().map(|p| p.as_ref().to_owned()).collect();
        let paths = Rc::new(UnsafeCell::new(paths));
        Self { paths }
    }
}

impl Serialize for OrdPaths {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let path = unsafe { &*self.paths.get() };
        path.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for OrdPaths {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let paths = Vec::<PathBuf>::deserialize(deserializer)?;
        Ok(Self::new(paths))
    }
}

unsafe impl Send for OrdPaths {}
unsafe impl Sync for OrdPaths {}

impl Default for OrdPaths {
    fn default() -> Self {
        Self {
            paths: Rc::new(UnsafeCell::new(Vec::new())),
        }
    }
}

impl OrdPaths {
    pub(crate) fn random_one(&self) -> Option<&PathBuf> {
        let mut rng = rand::thread_rng();
        unsafe { &*self.paths.get() }.choose(&mut rng)
    }

    pub(crate) fn is_empty(&self) -> bool {
        unsafe { &*self.paths.get() }.is_empty()
    }

    pub(crate) fn extend(&self, other: &Self) {
        let this = unsafe { &mut *self.paths.get() };
        let key = other[0].clone();
        let iter = unsafe { &mut *self.paths.get() }.drain(..);
        this.extend(iter);
        let other = unsafe { &mut *self.paths.get() };
        other.push(key);
    }
}

impl Index<usize> for OrdPaths {
    type Output = PathBuf;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.paths.get() }.index(index)
    }
}

impl PartialEq for OrdPaths {
    fn eq(&self, other: &Self) -> bool {
        if self.is_empty() || other.is_empty() {
            return true;
        }
        self[0] == other[0]
    }
}

impl Eq for OrdPaths {}

impl PartialOrd for OrdPaths {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrdPaths {
    /// If the comparison result is `Same`,
    /// the BTreeSet will replace the old one with the new one.
    /// So, merge the two paths into one to avoid data loss.
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }
        CMPDISPATCH
            .req_tx
            .send(Event::Compare([self.clone(), other.clone()]))
            .unwrap();
        match CMPDISPATCH.resp_rx.recv() {
            Ok(ord) => ord.into(),
            Err(_) => unreachable!(),
        }
    }
}

impl Hash for OrdPaths {
    /// The head path represents the whole.
    fn hash<H: Hasher>(&self, state: &mut H) {
        let paths = &self[0];
        paths.hash(state);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i8)]
pub(crate) enum CompareResult {
    MuchBetter = 2,
    Better = 1,
    #[default]
    Same = 0,
    Worse = -1,
    MuchWorse = -2,
}

impl Reflexivity for CompareResult {
    const ZERO: Self = CompareResult::Same;

    fn reverse(&self) -> Self {
        unsafe { std::mem::transmute(-(*self as i8)) }
    }
}

impl From<CompareResult> for Ordering {
    fn from(value: CompareResult) -> Self {
        (value as i8).cmp(&0)
    }
}
