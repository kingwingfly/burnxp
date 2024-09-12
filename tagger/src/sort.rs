use crate::event::{Event, CMPDISPATCHER};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::PathBuf;
use std::thread;

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
            Err(_) => {
                thread::park();
                Ordering::Equal
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i8)]
pub(crate) enum CompareResult {
    MuchBetter = 2,
    Better = 1,
    Same = 0,
    Worse = -1,
    MuchWorse = -2,
}

impl CompareResult {
    pub(crate) fn reverse(&self) -> Self {
        match self {
            CompareResult::MuchBetter => CompareResult::MuchWorse,
            CompareResult::Better => CompareResult::Worse,
            CompareResult::Same => CompareResult::Same,
            CompareResult::Worse => CompareResult::Better,
            CompareResult::MuchWorse => CompareResult::MuchBetter,
        }
    }
}

impl From<CompareResult> for Ordering {
    fn from(value: CompareResult) -> Self {
        match value {
            CompareResult::MuchBetter => Ordering::Greater,
            CompareResult::Better => Ordering::Greater,
            CompareResult::Same => Ordering::Equal,
            CompareResult::Worse => Ordering::Less,
            CompareResult::MuchWorse => Ordering::Less,
        }
    }
}
