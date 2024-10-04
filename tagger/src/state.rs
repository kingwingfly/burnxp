use std::{
    fmt,
    sync::{
        atomic::{AtomicUsize, Ordering},
        LazyLock,
    },
};

#[cfg(feature = "cmper")]
pub(crate) static PROCESS_WITH_COMPLEXITY: LazyLock<ProcessWithComplexity> =
    LazyLock::new(ProcessWithComplexity::default);
pub(crate) static PROCESS: LazyLock<Process> = LazyLock::new(Process::default);

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub(crate) enum CurrentScreen {
    #[default]
    Main,
    Popup(u8),
    Finished,
    Exiting,
}

#[cfg(feature = "cmper")]
#[derive(Default)]
pub(crate) struct ProcessWithComplexity {
    pub finished: AtomicUsize,
    pub total: AtomicUsize,
}

#[cfg(feature = "cmper")]
impl fmt::Display for ProcessWithComplexity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let finished = self.finished.load(Ordering::Relaxed);
        let total = self.total.load(Ordering::Relaxed);
        let complexity = complexity(finished, total);
        write!(f, "m/n={}/{} | ∑logi={}", finished, total, complexity)
    }
}

#[cfg(feature = "cmper")]
fn complexity(finished: usize, total: usize) -> usize {
    if total < 3 {
        return total.max(1) - 1;
    }
    let mut res = 0;
    for i in finished.max(1)..total {
        res += (i as f64).log2() as usize + 1;
    }
    res
}

#[derive(Default)]
pub(crate) struct Process {
    pub finished: AtomicUsize,
    pub total: AtomicUsize,
}

impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let finished = self.finished.load(Ordering::Relaxed);
        let total = self.total.load(Ordering::Relaxed);
        write!(f, "{}/{} ", finished, total)
    }
}
