use std::{
    fmt,
    sync::{
        atomic::{AtomicUsize, Ordering},
        LazyLock,
    },
};

pub(crate) static PROCESS: LazyLock<Process> = LazyLock::new(Process::default);

#[derive(Default, PartialEq, Clone, Copy)]
pub(crate) enum CurrentScreen {
    #[default]
    Main,
    Finished,
    Exiting,
}

#[derive(Default)]
pub(crate) struct Process {
    pub finished: AtomicUsize,
    pub total: AtomicUsize,
    pub complixity: AtomicUsize,
}

impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fisnihed = self.finished.load(Ordering::Relaxed);
        let total = self.total.load(Ordering::Relaxed);
        let complexity = (total - fisnihed) * (fisnihed as f64).log2() as usize;
        write!(f, "n/t={}/{} | (t-n)log(n)={}", fisnihed, total, complexity)
    }
}
