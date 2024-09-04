use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, Terminal};
use std::{
    io::Stderr,
    ops::{Deref, DerefMut},
};

pub(crate) struct AutoDropTerminal {
    terminal: Terminal<CrosstermBackend<Stderr>>,
}

impl AutoDropTerminal {
    pub(crate) fn new() -> Result<Self> {
        // setup terminal
        enable_raw_mode()?;
        let mut stderr = std::io::stderr();
        execute!(stderr, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stderr);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

impl Deref for AutoDropTerminal {
    type Target = Terminal<CrosstermBackend<Stderr>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for AutoDropTerminal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for AutoDropTerminal {
    fn drop(&mut self) {
        disable_raw_mode().ok();
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen,).ok();
    }
}
