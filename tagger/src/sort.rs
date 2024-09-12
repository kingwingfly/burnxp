use crate::event::{Event, CMPDISPATCHER};
use crate::matix::Reflexivity;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct OrdPath {
    pub path: PathBuf,
}

impl OrdPath {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl PartialOrd for OrdPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrdPath {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.path == other.path {
            return Ordering::Equal;
        }
        CMPDISPATCHER
            .req_tx
            .send(Event::Compare([self.path.clone(), other.path.clone()]))
            .unwrap();
        match CMPDISPATCHER.resp_rx.recv() {
            Ok(ord) => ord.into(),
            Err(_) => Ordering::Equal,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
    fn reverse(&self) -> Self {
        unsafe { std::mem::transmute(-(*self as i8)) }
    }
    fn zero() -> Self {
        CompareResult::Same
    }
}

impl From<CompareResult> for Ordering {
    fn from(value: CompareResult) -> Self {
        (value as i8).cmp(&0)
    }
}
