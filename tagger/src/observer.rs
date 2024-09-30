use crate::{
    components::{Histogram, Quit},
    state::CurrentScreen,
    terminal::AutoDropTerminal,
    utils::{centered_rect, json_from},
};
use anyhow::Result;
use crossterm::event::{self, Event as TermEvent, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Widget, WidgetRef},
};
use std::{path::PathBuf, time::Duration};

type Set = Vec<(Score, Vec<PathBuf>)>;
type Score = i64;

#[derive(Debug)]
pub struct Observer {
    data: Set,
    current_screen: CurrentScreen,
}

impl Observer {
    pub fn new(path: PathBuf) -> Result<Self> {
        let data = json_from(&path)?;
        Ok(Self {
            data,
            current_screen: CurrentScreen::Main,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = AutoDropTerminal::new()?;
        loop {
            terminal.draw(|f| {
                f.render_widget(&*self, f.area());
            })?;
            while let TermEvent::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    // Skip events that are not KeyEventKind::Press
                    continue;
                }
                if event::poll(Duration::from_millis(50))? {
                    continue;
                }
                match self.current_screen {
                    CurrentScreen::Main => match key.code {
                        KeyCode::Char('q') => {
                            self.current_screen = CurrentScreen::Exiting;
                        }
                        _ => continue,
                    },
                    CurrentScreen::Exiting => match key.code {
                        KeyCode::Char('y') => {
                            return Ok(());
                        }
                        _ => self.current_screen = CurrentScreen::Main,
                    },
                    CurrentScreen::Finished => unreachable!(),
                }
                break;
            }
        }
    }
}

impl WidgetRef for Observer {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if CurrentScreen::Exiting == self.current_screen {
            let area = centered_rect(60, 25, area);
            Quit.render(area, buf);
            return;
        }
        let data = self
            .data
            .iter()
            .map(|(score, paths)| (*score, paths.len()))
            .collect();
        Histogram { data }.render(area, buf);
    }
}
