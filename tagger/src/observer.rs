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
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
struct Cache {
    tags: HashMap<String, i64>,
    tagged: HashMap<PathBuf, Vec<String>>,
}

#[derive(Debug)]
pub struct Observer {
    data: Cache,
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
                    _ => unreachable!(),
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
        let mut all_tags = self.data.tags.keys().cloned().collect::<Vec<_>>();
        all_tags.sort();
        let mut data = vec![0; all_tags.len()];
        for (_, tags) in self.data.tagged.iter() {
            for (i, tag) in all_tags.iter().enumerate() {
                if tags.contains(tag) {
                    data[i] += 1;
                }
            }
        }
        let data = all_tags.into_iter().zip(data).collect();
        Histogram { data }.render(area, buf);
    }
}
