use super::Render;
use crate::state::CurrentScreen;
use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(crate) struct Footer {
    pub(crate) current_screen: CurrentScreen,
    pub(crate) process: (usize, usize),
}

impl Render for Footer {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        Navigation {
            current_screen: self.current_screen,
            process: self.process,
        }
        .render(f, chunks[0])?;
        Hint {
            current_screen: self.current_screen,
        }
        .render(f, chunks[1])?;
        Ok(())
    }
}

struct Navigation {
    pub(crate) current_screen: CurrentScreen,
    pub(crate) process: (usize, usize),
}

struct Hint {
    pub(crate) current_screen: CurrentScreen,
}

impl Render for Navigation {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let current_navigation_text = vec![
            match self.current_screen {
                CurrentScreen::Main => {
                    Span::styled("Which is better?", Style::default().fg(Color::Green))
                }

                CurrentScreen::Finished => {
                    Span::styled("Finished", Style::default().fg(Color::LightGreen))
                }

                CurrentScreen::Exiting => {
                    Span::styled("Exiting", Style::default().fg(Color::LightRed))
                }
            },
            Span::styled(" | ", Style::default().fg(Color::White)),
            match self.current_screen {
                CurrentScreen::Main => Span::styled(
                    format!("{}/{} O(nlogn)", self.process.0, self.process.1),
                    Style::default().fg(Color::DarkGray),
                ),
                CurrentScreen::Finished => Span::styled(
                    "The result has been saved",
                    Style::default().fg(Color::DarkGray),
                ),
                CurrentScreen::Exiting => {
                    Span::styled("Exiting", Style::default().fg(Color::LightRed))
                }
            },
        ];
        let mode_footer = Paragraph::new(Line::from(current_navigation_text))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(mode_footer, area);
        Ok(())
    }
}

impl Render for Hint {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let hint = {
            match self.current_screen {
                CurrentScreen::Main => Span::styled(
                    "Left(<-) Equal(=) Right(->) quit(q)",
                    Style::default().fg(Color::Red),
                ),
                CurrentScreen::Finished => {
                    Span::styled("Press (q) to exit", Style::default().fg(Color::LightGreen))
                }
                CurrentScreen::Exiting => Span::styled(
                    "Are you sure you want to exit? (y/n)",
                    Style::default().fg(Color::LightRed),
                ),
            }
        };
        let key_notes_footer = Paragraph::new(hint).block(Block::default().borders(Borders::ALL));
        f.render_widget(key_notes_footer, area);
        Ok(())
    }
}
