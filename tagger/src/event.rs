use crate::sort::{CompareResult, OrdPaths};
use crossbeam::channel::{bounded, Receiver, Sender};
use std::sync::LazyLock;

pub(crate) static CMPDISPATCH: LazyLock<CmpDispatcher> = LazyLock::new(|| {
    let (req_tx, req_rx) = bounded(0);
    let (resp_tx, resp_rx) = bounded(0);
    CmpDispatcher {
        req_tx,
        req_rx,
        resp_rx,
        resp_tx,
    }
});

pub(crate) type ComparePair = [OrdPaths; 2];

pub(crate) enum Event {
    Compare(ComparePair),
    Finished,
}

unsafe impl Send for Event {}
unsafe impl Sync for Event {}

pub(crate) struct CmpDispatcher {
    pub req_tx: Sender<Event>,
    pub req_rx: Receiver<Event>,
    pub resp_tx: Sender<CompareResult>,
    pub resp_rx: Receiver<CompareResult>,
}
