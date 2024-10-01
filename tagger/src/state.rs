use std::{
    fmt,
    sync::{
        atomic::{AtomicUsize, Ordering},
        LazyLock,
    },
};

pub(crate) static TAGGER_PROCESS: LazyLock<TaggerProcess> = LazyLock::new(TaggerProcess::default);
pub(crate) static PICKER_PROCESS: LazyLock<PickerProcess> = LazyLock::new(PickerProcess::default);

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub(crate) enum CurrentScreen {
    #[default]
    Main,
    Popup,
    Finished,
    Exiting,
}

#[derive(Default)]
pub(crate) struct TaggerProcess {
    pub finished: AtomicUsize,
    pub total: AtomicUsize,
}

impl fmt::Display for TaggerProcess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let finished = self.finished.load(Ordering::Relaxed);
        let total = self.total.load(Ordering::Relaxed);
        let complexity = complexity(finished, total);
        write!(f, "m/n={}/{} | ∑logi={}", finished, total, complexity)
    }
}

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
pub(crate) struct PickerProcess {
    pub finished: AtomicUsize,
    pub total: AtomicUsize,
}

impl fmt::Display for PickerProcess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let finished = self.finished.load(Ordering::Relaxed);
        let total = self.total.load(Ordering::Relaxed);
        write!(f, "{}/{} ", finished, total)
    }
}
