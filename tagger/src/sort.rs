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
            Ok(ord) => ord,
            Err(_) => {
                thread::park();
                Ordering::Equal
            }
        }
    }
}
