use crate::components::{Component, Footer, Images, Quit, Title};
use crate::state::CurrentScreen;
use crate::terminal::AutoDropTerminal;
use crate::utils::centered_rect;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

#[derive(Default)]
pub struct App {
    _imgs: Vec<String>,
    current_screen: CurrentScreen,
}

impl App {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = AutoDropTerminal::new()?;
        loop {
            terminal.draw(|f| {
                self.render(f, f.size()).ok();
            })?;
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    // Skip events that are not KeyEventKind::Press
                    continue;
                }
                match self.current_screen {
                    CurrentScreen::Main => match key.code {
                        KeyCode::Char('q') => {
                            self.current_screen = CurrentScreen::Exiting;
                        }
                        KeyCode::Left => {}
                        KeyCode::Right => {}
                        _ => {}
                    },
                    CurrentScreen::Exiting => match key.code {
                        KeyCode::Char('y') => break Ok(()),
                        _ => self.current_screen = CurrentScreen::Main,
                    },
                }
            }
        }
    }
}

impl Component for App {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        if CurrentScreen::Exiting == self.current_screen {
            let area = centered_rect(60, 25, f.size());
            Quit.render(f, area)?;
            return Ok(());
        }
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(area);
        Title {
            title: "Tagger".to_string(),
        }
        .render(f, chunks[0])?;
        Images::new(&["images/test.jpg", "images/test.jpg"])?.render(f, chunks[1])?;
        Footer {
            current_screen: self.current_screen,
        }
        .render(f, chunks[2])?;
        Ok(())
    }
}
