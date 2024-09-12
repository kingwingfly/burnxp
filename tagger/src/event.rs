use crate::sort::CompareResult;
use crossbeam::channel::{bounded, Receiver, Sender};
use std::{path::PathBuf, sync::LazyLock};

pub(crate) static CMPDISPATCHER: LazyLock<CmpDispatcher> = LazyLock::new(|| {
    let (req_tx, req_rx) = bounded(0);
    let (resp_tx, resp_rx) = bounded(0);
    CmpDispatcher {
        req_tx,
        req_rx,
        resp_rx,
        resp_tx,
    }
});

pub(crate) type ComparePair = [PathBuf; 2];

pub(crate) enum Event {
    Compare(ComparePair),
    Finished,
}

pub(crate) struct CmpDispatcher {
    pub req_tx: Sender<Event>,
    pub req_rx: Receiver<Event>,
    pub resp_tx: Sender<CompareResult>,
    pub resp_rx: Receiver<CompareResult>,
}
